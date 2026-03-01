import { sealBoxPayload, openBoxPayload, generateEphemeralKeyPair, toBase64, fromBase64, DEFAULT_CHUNK_SIZE, EncryptionError, IntegrityError, KeyMismatchError, computeSas, bufferToHex, hashFile, isValidWireErrorCode } from '@the9ines/bolt-core';
import { WebRTCError, ConnectionError, SignalingError, TransferError } from '../../types/webrtc-errors.js';
import { getLocalOnlyRTCConfig } from '../../lib/platform-utils.js';
import { verifyPinnedIdentity } from '../identity/pin-store.js';
import type { PinPersistence } from '../identity/pin-store.js';
import type { SignalingProvider, SignalMessage } from '../signaling/index.js';
import { ENABLE_TRANSFER_METRICS, TransferMetricsCollector, summarizeTransfer } from './transferMetrics.js';

// ─── Types ──────────────────────────────────────────────────────────────────

export interface TransferStats {
  speed: number;
  averageSpeed: number;
  estimatedTimeRemaining: number;
  retryCount: number;
  maxRetries: number;
  startTime: number;
  pauseDuration: number;
  lastPausedAt?: number;
}

export interface TransferProgress {
  filename: string;
  currentChunk: number;
  totalChunks: number;
  loaded: number;
  total: number;
  status?: 'transferring' | 'paused' | 'canceled_by_sender' | 'canceled_by_receiver' | 'error' | 'completed';
  stats?: TransferStats;
}

interface FileChunkMessage {
  type: 'file-chunk';
  filename: string;
  chunk?: string;
  chunkIndex?: number;
  totalChunks?: number;
  fileSize?: number;
  transferId?: string;
  fileHash?: string;
  cancelled?: boolean;
  cancelledBy?: 'sender' | 'receiver';
  paused?: boolean;
  resumed?: boolean;
}

/** Profile Envelope v1 wire format — encrypts inner messages over DataChannel. */
interface ProfileEnvelopeV1 {
  type: 'profile-envelope';
  version: 1;
  encoding: 'base64';
  payload: string;
}

/** Receiver-side state for a guarded transfer (transferId present). */
interface ActiveTransfer {
  transferId: string;
  filename: string;
  totalChunks: number;
  fileSize: number;
  buffer: (Blob | null)[];
  receivedSet: Set<number>;
  remoteIdentityKey: string;
  expectedHash?: string;
}

/** Generate a spec-compliant transfer_id (bytes16 → hex, 32 chars). */
function generateTransferId(): string {
  const bytes = new Uint8Array(16);
  crypto.getRandomValues(bytes);
  return bufferToHex(bytes.buffer);
}

export type VerificationState = 'unverified' | 'verified' | 'legacy';

export interface VerificationInfo {
  state: VerificationState;
  sasCode: string | null;
}

export interface WebRTCServiceOptions {
  /** Local identity public key to send in encrypted HELLO. */
  identityPublicKey?: Uint8Array;
  /** Pin store for TOFU peer identity verification. */
  pinStore?: PinPersistence;
  /** Callback fired when verification state changes (after HELLO or on legacy timeout). */
  onVerificationState?: (info: VerificationInfo) => void;
}

const HELLO_TIMEOUT_MS = 5000;

// ─── WebRTCService ──────────────────────────────────────────────────────────

class WebRTCService {
  // Connection
  private pc: RTCPeerConnection | null = null;
  private dc: RTCDataChannel | null = null;
  private signaling: SignalingProvider;

  // Encryption — generated per session in connect() / handleOffer()
  private keyPair: { publicKey: Uint8Array; secretKey: Uint8Array } | null = null;
  private remotePublicKey: Uint8Array | null = null;

  // Peer codes
  private remotePeerCode: string = '';

  // ICE
  private pendingCandidates: RTCIceCandidateInit[] = [];

  // Transfer state
  private transferPaused = false;
  private transferCancelled = false;
  private receiveBuffers: Map<string, (Blob | null)[]> = new Map();
  private guardedTransfers: Map<string, ActiveTransfer> = new Map();
  private sendTransferIds: Map<string, string> = new Map();
  private recvTransferIds: Map<string, string> = new Map();

  // Stats
  private transferStartTime = 0;
  private pauseDuration = 0;
  private lastPausedAt: number | null = null;

  // Metrics (S2B instrumentation, gated by ENABLE_TRANSFER_METRICS)
  private metricsCollector: TransferMetricsCollector | null = null;
  private metricsFirstProgressRecorded = false;

  // Callbacks
  private onProgressCallback?: (progress: TransferProgress) => void;
  private connectionStateHandler?: (state: RTCPeerConnectionState) => void;

  // Connect promise resolution (for caller side)
  private connectResolve: (() => void) | null = null;
  private connectReject: ((err: Error) => void) | null = null;
  private connectTimeout: ReturnType<typeof setTimeout> | null = null;

  // HELLO / TOFU state
  private options: WebRTCServiceOptions;
  private helloComplete = false;
  private sessionLegacy = false;
  private sessionState: 'pre_hello' | 'post_hello' | 'closed' = 'pre_hello';
  private helloTimeout: ReturnType<typeof setTimeout> | null = null;
  private helloResolve: (() => void) | null = null;
  private helloProcessing = false; // SA12: reentrancy guard
  private sessionGeneration = 0; // SA14: stale timeout guard

  // SA6: signaling listener unsubscribe handle
  private signalUnsub?: () => void;

  // N1: backpressure cancel hook — settled by disconnect() to prevent hang
  private backpressureReject?: (err: Error) => void;

  // N10: completion timer — cleared by disconnect() to prevent stale event
  private completionTimeout: ReturnType<typeof setTimeout> | null = null;

  // SAS verification state
  private remoteIdentityKey: Uint8Array | null = null;
  private verificationInfo: VerificationInfo = { state: 'legacy', sasCode: null };

  // Capabilities negotiation
  private localCapabilities: string[] = ['bolt.file-hash', 'bolt.profile-envelope-v1'];
  private remoteCapabilities: string[] = [];
  private negotiatedCapabilities: string[] = [];

  constructor(
    signaling: SignalingProvider,
    private localPeerCode: string,
    private onReceiveFile: (file: Blob, filename: string) => void,
    private onError: (error: WebRTCError) => void,
    onProgress?: (progress: TransferProgress) => void,
    options?: WebRTCServiceOptions,
  ) {
    console.log('[INIT] WebRTCService with peer code:', localPeerCode);
    this.onProgressCallback = onProgress;
    this.options = options ?? {};
    this.signaling = signaling;
    this.signalUnsub = this.signaling.onSignal((signal) => this.handleSignal(signal));
  }

  // ─── Signaling ──────────────────────────────────────────────────────────

  private async sendSignal(type: SignalMessage['type'], data: any, to: string) {
    console.log('[SIGNALING] Sending', type, 'to', to);
    await this.signaling.sendSignal(type, data, to);
  }

  private async handleSignal(signal: SignalMessage) {
    if (signal.to !== this.localPeerCode) return;
    // Only handle WebRTC signal types — ignore custom approval types
    if (signal.type !== 'offer' && signal.type !== 'answer' && signal.type !== 'ice-candidate') return;

    console.log('[SIGNALING] Received', signal.type, 'from', signal.from);
    if (signal.from) this.remotePeerCode = signal.from;

    try {
      switch (signal.type) {
        case 'offer':
          await this.handleOffer(signal);
          break;
        case 'answer':
          await this.handleAnswer(signal);
          break;
        case 'ice-candidate':
          await this.handleIceCandidate(signal);
          break;
      }
    } catch (error) {
      console.error('[SIGNAL] Error handling', signal.type, ':', error);
      // SA5: deterministic cleanup before surfacing error — prevents pc/dc leak on throw
      this.disconnect();
      this.onError(error instanceof WebRTCError ? error : new ConnectionError('Signal handling failed', error));
    }
  }

  private async handleOffer(signal: SignalMessage) {
    this.keyPair = generateEphemeralKeyPair();
    console.log('[SIGNALING] Processing offer from', signal.from);
    this.remotePublicKey = fromBase64(signal.data.publicKey);

    const pc = this.createPeerConnection();
    await pc.setRemoteDescription(new RTCSessionDescription(signal.data.offer));
    console.log('[SIGNALING] Remote description set (offer)');

    const answer = await pc.createAnswer();
    await pc.setLocalDescription(answer);
    console.log('[SIGNALING] Local description set (answer)');

    await this.sendSignal('answer', {
      answer,
      publicKey: toBase64(this.keyPair!.publicKey),
      peerCode: this.localPeerCode,
    }, signal.from);

    await this.flushPendingCandidates();
  }

  private async handleAnswer(signal: SignalMessage) {
    console.log('[SIGNALING] Processing answer from', signal.from);
    this.remotePublicKey = fromBase64(signal.data.publicKey);

    if (!this.pc) throw new ConnectionError('No peer connection for answer');
    console.log('[SIGNALING] Signaling state:', this.pc.signalingState);
    if (this.pc.signalingState !== 'have-local-offer') {
      throw new ConnectionError('Received answer in invalid state: ' + this.pc.signalingState);
    }

    await this.pc.setRemoteDescription(new RTCSessionDescription(signal.data.answer));
    console.log('[SIGNALING] Remote description set (answer)');
    await this.flushPendingCandidates();
  }

  private async handleIceCandidate(signal: SignalMessage) {
    const candidate = new RTCIceCandidate(signal.data);
    console.log('[ICE] Remote candidate:', candidate.type, candidate.address, candidate.protocol);

    if (!this.pc || !this.pc.remoteDescription) {
      console.log('[ICE] Queuing candidate (no PC or remote desc)');
      this.pendingCandidates.push(signal.data);
      return;
    }

    try {
      await this.pc.addIceCandidate(candidate);
      console.log('[ICE] Added remote candidate');
    } catch (err) {
      console.warn('[ICE] Failed to add candidate, queuing:', err);
      this.pendingCandidates.push(signal.data);
    }
  }

  private async flushPendingCandidates() {
    if (!this.pc) return;
    console.log('[ICE] Flushing', this.pendingCandidates.length, 'pending candidates');
    while (this.pendingCandidates.length > 0) {
      const c = this.pendingCandidates.shift()!;
      try {
        await this.pc.addIceCandidate(new RTCIceCandidate(c));
      } catch (error) {
        console.warn('[ICE] Failed to add pending candidate:', error);
      }
    }
  }

  // ─── Connection ─────────────────────────────────────────────────────────

  private createPeerConnection(): RTCPeerConnection {
    if (this.pc) {
      console.log('[WEBRTC] Closing existing peer connection');
      // Remove handlers before closing to avoid spurious error callbacks
      this.pc.onconnectionstatechange = null;
      this.pc.onicecandidate = null;
      this.pc.ondatachannel = null;
      this.pc.oniceconnectionstatechange = null;
      this.pc.close();
      this.pc = null;
    }

    const config = getLocalOnlyRTCConfig();
    console.log('[WEBRTC] Creating peer connection with config:', JSON.stringify(config));
    this.pc = new RTCPeerConnection(config);

    // Filter ICE candidates — block relay to enforce same-network policy proactively
    this.pc.onicecandidate = (event) => {
      if (event.candidate) {
        if (event.candidate.type === 'relay') {
          console.log('[ICE] Blocking relay candidate (same-network policy)');
          return;
        }
        console.log('[ICE] Local candidate:', event.candidate.type, event.candidate.address, event.candidate.protocol);
        this.sendSignal('ice-candidate', event.candidate.toJSON(), this.remotePeerCode)
          .catch(err => console.error('[ICE] Failed to send candidate:', err));
      } else {
        console.log('[ICE] Candidate gathering complete');
      }
    };

    // ICE connection state (more granular than connection state)
    this.pc.oniceconnectionstatechange = () => {
      console.log('[ICE] Connection state:', this.pc?.iceConnectionState);
    };

    // Connection state
    this.pc.onconnectionstatechange = () => {
      const state = this.pc?.connectionState;
      console.log('[WEBRTC] Connection state:', state);
      if (!state) return;

      if (this.connectionStateHandler) this.connectionStateHandler(state);

      if (state === 'connected') {
        console.log('[WEBRTC] Connection established!');
        this.enforceSameNetworkPolicy();
        // Resolve connect() promise if waiting
        if (this.connectResolve) {
          if (this.connectTimeout) clearTimeout(this.connectTimeout);
          this.connectResolve();
          this.connectResolve = null;
          this.connectReject = null;
          this.connectTimeout = null;
        }
      } else if (state === 'failed' || state === 'closed' || state === 'disconnected') {
        // Reject connect() promise if waiting
        if (this.connectReject && (state === 'failed' || state === 'closed')) {
          if (this.connectTimeout) clearTimeout(this.connectTimeout);
          this.connectReject(new ConnectionError('Connection ' + state));
          this.connectResolve = null;
          this.connectReject = null;
          this.connectTimeout = null;
        }
        if (state === 'failed' || state === 'closed') {
          this.onError(new ConnectionError('WebRTC connection ' + state));
        }
      }
    };

    // Incoming data channel (receiver side)
    this.pc.ondatachannel = (event) => {
      console.log('[DC] Received data channel from remote peer');
      this.setupDataChannel(event.channel);
    };

    return this.pc;
  }

  private setupDataChannel(channel: RTCDataChannel) {
    this.dc = channel;
    this.dc.binaryType = 'arraybuffer';
    this.sessionState = 'pre_hello';
    this.dc.onmessage = (event) => this.handleMessage(event);
    this.dc.onopen = () => {
      console.log('[DC] Data channel open');
      this.initiateHello();
    };
    this.dc.onclose = () => console.log('[DC] Data channel closed');
    this.dc.onerror = (e) => console.error('[DC] Error:', e);
  }

  private async enforceSameNetworkPolicy() {
    if (!this.pc) return;

    try {
      const stats = await this.pc.getStats();
      const reports = Array.from(stats.values());
      const pair = reports.find((r: any) => r.type === 'candidate-pair' && (r.selected || r.nominated));
      if (!pair) {
        console.log('[POLICY] No selected candidate pair found (will check again later)');
        return;
      }

      const local = reports.find((r: any) => r.id === (pair as any).localCandidateId) as any;
      const remote = reports.find((r: any) => r.id === (pair as any).remoteCandidateId) as any;
      if (!local || !remote) return;

      console.log('[POLICY] Selected pair — local:', local.candidateType, local.address, '→ remote:', remote.candidateType, remote.address);

      if (local.candidateType === 'relay' || remote.candidateType === 'relay') {
        throw new ConnectionError('Same-network policy violation: relay candidate selected');
      }

      if (
        (local.candidateType === 'srflx' || remote.candidateType === 'srflx') &&
        local.address && remote.address &&
        local.address !== remote.address
      ) {
        throw new ConnectionError('Same-network policy violation: different public addresses');
      }

      console.log('[POLICY] Same-network check passed');
    } catch (error) {
      if (error instanceof ConnectionError) {
        console.error('[POLICY]', error.message);
        this.onError(error);
        this.disconnect();
      }
    }
  }

  /** Send an error frame and disconnect. Wraps in envelope when negotiated. */
  private sendErrorAndDisconnect(code: string, message: string): void {
    // Outbound guard: only emit canonical wire error codes (PROTOCOL.md §10)
    if (!isValidWireErrorCode(code)) {
      console.error(`[BUG] sendErrorAndDisconnect called with non-canonical code: ${code}`);
      this.disconnect();
      return;
    }
    if (this.dc && this.dc.readyState === 'open') {
      const errorMsg = { type: 'error', code, message };
      if (this.negotiatedEnvelopeV1() && this.helloComplete && this.keyPair && this.remotePublicKey) {
        this.dc.send(JSON.stringify(this.encodeProfileEnvelopeV1(errorMsg)));
      } else {
        this.dc.send(JSON.stringify(errorMsg));
      }
    }
    this.disconnect();
  }

  // ─── HELLO Protocol ──────────────────────────────────────────────────────

  private initiateHello() {
    if (!this.options.identityPublicKey || !this.keyPair || !this.remotePublicKey) {
      // No identity configured — this node operates in legacy mode
      this.sessionState = 'post_hello';
      this.helloComplete = true;
      this.sessionLegacy = true;
      this.verificationInfo = { state: 'legacy', sasCode: null };
      this.options.onVerificationState?.(this.verificationInfo);
      console.log('[HELLO] No identity configured, skipping HELLO');
      return;
    }

    // Send encrypted HELLO over DataChannel
    const hello = JSON.stringify({
      type: 'hello',
      version: 1,
      identityPublicKey: toBase64(this.options.identityPublicKey),
      capabilities: this.localCapabilities,
    });
    const plaintext = new TextEncoder().encode(hello);
    const encrypted = sealBoxPayload(plaintext, this.remotePublicKey, this.keyPair.secretKey);
    this.dc!.send(JSON.stringify({ type: 'hello', payload: encrypted }));
    console.log('[HELLO] Sent encrypted HELLO');

    // Start timeout — fail-closed if remote doesn't complete HELLO (SA10)
    // SA14: capture session generation to detect stale callbacks after disconnect+reconnect
    const gen = this.sessionGeneration;
    this.helloTimeout = setTimeout(() => {
      if (gen !== this.sessionGeneration) return; // stale timeout from previous session
      if (!this.helloComplete) {
        console.error('[HELLO_TIMEOUT] HELLO not completed within timeout — identity required, failing closed');
        const error = new ConnectionError('HELLO handshake timed out while identity is required');
        this.disconnect();
        this.onError(error);
      }
    }, HELLO_TIMEOUT_MS);
  }

  private async processHello(msg: { type: 'hello'; payload: string }) {
    // SA12: synchronous reentrancy guard — must be set before any await
    if (this.helloProcessing) {
      console.warn('[DUPLICATE_HELLO] HELLO received while processing — disconnecting');
      this.sendErrorAndDisconnect('DUPLICATE_HELLO', 'Duplicate HELLO');
      return;
    }
    this.helloProcessing = true;

    // N2: scoped-lock — try/finally guarantees reset on all exits (success, error, unexpected throw)
    try {
      // H2: Fail-closed HELLO processing — all failures send error + disconnect
      if (!this.keyPair || !this.remotePublicKey) {
        console.warn('[HELLO_DECRYPT_FAIL] Cannot decrypt — no ephemeral keys');
        this.sendErrorAndDisconnect('HELLO_DECRYPT_FAIL', 'Cannot decrypt HELLO');
        return;
      }

      let decrypted: Uint8Array;
      try {
        decrypted = openBoxPayload(msg.payload, this.remotePublicKey, this.keyPair.secretKey);
      } catch {
        console.warn('[HELLO_DECRYPT_FAIL] Failed to decrypt HELLO payload');
        this.sendErrorAndDisconnect('HELLO_DECRYPT_FAIL', 'Failed to decrypt HELLO');
        return;
      }

      let hello: any;
      try {
        hello = JSON.parse(new TextDecoder().decode(decrypted));
      } catch {
        console.warn('[HELLO_PARSE_ERROR] Failed to parse HELLO JSON');
        this.sendErrorAndDisconnect('HELLO_PARSE_ERROR', 'Failed to parse HELLO');
        return;
      }

      if (hello.type !== 'hello' || hello.version !== 1 || !hello.identityPublicKey) {
        console.warn('[HELLO_SCHEMA_ERROR] Invalid HELLO format');
        this.sendErrorAndDisconnect('HELLO_SCHEMA_ERROR', 'Invalid HELLO schema');
        return;
      }

      const remoteIdentityKey = fromBase64(hello.identityPublicKey);
      this.remoteIdentityKey = remoteIdentityKey;

      // Capabilities negotiation — missing field treated as empty (backward compat)
      // SA17: reject oversized capabilities array (max 32)
      const rawCaps = Array.isArray(hello.capabilities) ? hello.capabilities : [];
      if (rawCaps.length > 32) {
        console.warn(`[PROTOCOL_VIOLATION] capabilities array length ${rawCaps.length} exceeds max 32 — disconnecting`);
        this.sendErrorAndDisconnect('PROTOCOL_VIOLATION', 'Capabilities array exceeds maximum length');
        return;
      }
      // N8: reject individual capability strings exceeding 64 UTF-8 bytes
      const encoder = new TextEncoder();
      for (const cap of rawCaps) {
        if (encoder.encode(cap).length > 64) {
          console.warn('[PROTOCOL_VIOLATION] capability too long — disconnecting');
          this.sendErrorAndDisconnect('PROTOCOL_VIOLATION', 'capability too long');
          return;
        }
      }
      this.remoteCapabilities = rawCaps;
      const localSet = new Set(this.localCapabilities);
      this.negotiatedCapabilities = this.remoteCapabilities.filter((c: string) => localSet.has(c));
      console.log('[HELLO] Remote capabilities:', this.remoteCapabilities, '→ negotiated:', this.negotiatedCapabilities);

      // N5: Enforce envelope-v1 in identity-configured sessions.
      // If we reach processHello(), identity IS configured. Remote MUST
      // advertise bolt.profile-envelope-v1 — omission is downgrade attack.
      if (!rawCaps.includes('bolt.profile-envelope-v1')) {
        console.warn('[PROTOCOL_VIOLATION] Remote omitted required capability bolt.profile-envelope-v1 — disconnecting');
        this.sendErrorAndDisconnect('PROTOCOL_VIOLATION', 'Missing required capability: bolt.profile-envelope-v1');
        return;
      }

      console.log('[HELLO] Received identity from peer', this.remotePeerCode);

      // TOFU verification — determines verification state
      let verificationState: VerificationState = 'unverified';

      if (this.options.pinStore) {
        try {
          const result = await verifyPinnedIdentity(
            this.options.pinStore,
            this.remotePeerCode,
            remoteIdentityKey,
          );
          if (result.outcome === 'pinned') {
            console.log('[TOFU] First contact — pinned identity for', this.remotePeerCode);
            verificationState = 'unverified';
          } else {
            console.log('[TOFU] Identity verified for', this.remotePeerCode);
            verificationState = result.verified ? 'verified' : 'unverified';
          }
        } catch (error) {
          if (error instanceof KeyMismatchError) {
            console.error('[TOFU] IDENTITY MISMATCH — aborting session:', error.message);
            this.onError(new ConnectionError('Identity key mismatch (TOFU violation)', error));
            this.sendErrorAndDisconnect('KEY_MISMATCH', 'Identity key mismatch');
            return;
          }
          throw error;
        }
      }

      // Compute SAS — only when all 4 keys are available (never in legacy path)
      let sasCode: string | null = null;
      if (this.options.identityPublicKey && this.keyPair && this.remotePublicKey) {
        sasCode = await computeSas(
          this.options.identityPublicKey,
          remoteIdentityKey,
          this.keyPair.publicKey,
          this.remotePublicKey,
        );
        console.log('[SAS] Computed verification code:', sasCode);
      }

      // Emit verification state exactly once per HELLO
      this.verificationInfo = { state: verificationState, sasCode };
      this.options.onVerificationState?.(this.verificationInfo);

      // HELLO complete — transition state
      if (this.helloTimeout) {
        clearTimeout(this.helloTimeout);
        this.helloTimeout = null;
      }
      this.sessionState = 'post_hello';
      this.helloComplete = true;
      this.sessionLegacy = false;
      if (this.helloResolve) {
        this.helloResolve();
        this.helloResolve = null;
      }
    } finally {
      this.helloProcessing = false;
    }
  }

  /**
   * Wait for the HELLO handshake to complete (or legacy timeout).
   * Returns immediately if already complete.
   */
  waitForHello(): Promise<void> {
    if (this.helloComplete) return Promise.resolve();
    return new Promise<void>((resolve) => {
      this.helloResolve = resolve;
    });
  }

  /** Whether the remote peer was identified as legacy (no HELLO). */
  isLegacySession(): boolean {
    return this.sessionLegacy;
  }

  async connect(remotePeerCode: string): Promise<void> {
    this.keyPair = generateEphemeralKeyPair();
    console.log('[WEBRTC] Connecting to peer:', remotePeerCode);
    this.remotePeerCode = remotePeerCode;
    const pc = this.createPeerConnection();

    const dc = pc.createDataChannel('fileTransfer', { ordered: true });
    this.setupDataChannel(dc);

    const offer = await pc.createOffer();
    await pc.setLocalDescription(offer);
    console.log('[SIGNALING] Local description set (offer)');

    await this.sendSignal('offer', {
      offer,
      publicKey: toBase64(this.keyPair!.publicKey),
      peerCode: this.localPeerCode,
    }, remotePeerCode);

    // Wait for connection or timeout using class-level resolve/reject
    // This avoids the handler-wrapping race condition
    const connectionPromise = new Promise<void>((resolve, reject) => {
      this.connectResolve = resolve;
      this.connectReject = reject;
      this.connectTimeout = setTimeout(() => {
        this.connectResolve = null;
        this.connectReject = null;
        this.connectTimeout = null;
        reject(new ConnectionError('Connection timeout (30s)'));
      }, 30000);
    });

    try {
      await connectionPromise;
    } catch (error) {
      this.disconnect();
      throw error instanceof WebRTCError ? error : new ConnectionError('Connection failed', error);
    }
  }

  disconnect() {
    console.log('[WEBRTC] Disconnecting');
    // SA14: increment generation so stale timeout callbacks from this session become no-ops
    this.sessionGeneration++;
    // SA6: unregister signaling listener first to prevent further signal delivery
    this.signalUnsub?.();
    this.signalUnsub = undefined;
    // Zero and discard ephemeral key material
    if (this.keyPair) {
      this.keyPair.secretKey.fill(0);
      this.keyPair = null;
    }
    // Clear any pending connect promise
    if (this.connectTimeout) clearTimeout(this.connectTimeout);
    this.connectResolve = null;
    this.connectReject = null;
    this.connectTimeout = null;

    // N1: settle any pending backpressure wait before closing DC
    if (this.backpressureReject) {
      this.backpressureReject(new TransferError('Transfer aborted: disconnected'));
      this.backpressureReject = undefined;
    }

    // N10: clear pending completion timer to prevent stale event after teardown
    if (this.completionTimeout) {
      clearTimeout(this.completionTimeout);
      this.completionTimeout = null;
    }

    if (this.dc) {
      // SA13 + N1: null handlers before close to prevent post-close event delivery
      this.dc.onmessage = null;
      this.dc.onopen = null;
      this.dc.onclose = null;
      this.dc.onerror = null;
      this.dc.onbufferedamountlow = null;
      try { this.dc.close(); } catch { /* ignore */ }
      this.dc = null;
    }
    if (this.pc) {
      // Remove handlers before closing to avoid spurious callbacks
      this.pc.onconnectionstatechange = null;
      this.pc.onicecandidate = null;
      this.pc.ondatachannel = null;
      this.pc.oniceconnectionstatechange = null;
      try { this.pc.close(); } catch { /* ignore */ }
      this.pc = null;
    }
    if (this.remotePublicKey instanceof Uint8Array) this.remotePublicKey.fill(0);
    this.remotePublicKey = null;
    this.remotePeerCode = '';
    this.pendingCandidates = [];
    this.transferPaused = false;
    this.transferCancelled = false;
    this.receiveBuffers.clear();
    this.guardedTransfers.clear();
    this.sendTransferIds.clear();
    this.recvTransferIds.clear();

    if (this.metricsCollector) {
      this.metricsCollector.reset();
      this.metricsCollector = null;
      this.metricsFirstProgressRecorded = false;
    }

    // Clear HELLO / TOFU / SAS state
    this.sessionState = 'closed';
    if (this.helloTimeout) {
      clearTimeout(this.helloTimeout);
      this.helloTimeout = null;
    }
    this.helloComplete = false;
    this.helloProcessing = false;
    this.sessionLegacy = false;
    this.helloResolve = null;
    if (this.remoteIdentityKey instanceof Uint8Array) this.remoteIdentityKey.fill(0);
    this.remoteIdentityKey = null;
    this.verificationInfo = { state: 'legacy', sasCode: null };

    // Clear capabilities
    this.remoteCapabilities = [];
    this.negotiatedCapabilities = [];
  }

  // ─── File Transfer (Send) ──────────────────────────────────────────────

  async sendFile(file: File): Promise<void> {
    // Wait for HELLO handshake before allowing file transfer
    if (!this.helloComplete) {
      await this.waitForHello();
    }

    if (!this.dc || this.dc.readyState !== 'open') {
      throw new TransferError('Data channel not open');
    }
    if (!this.remotePublicKey) {
      throw new EncryptionError('No remote public key');
    }

    const totalChunks = Math.ceil(file.size / DEFAULT_CHUNK_SIZE);
    const transferId = generateTransferId();
    this.sendTransferIds.set(file.name, transferId);

    // Compute file hash if bolt.file-hash was negotiated
    let fileHash: string | undefined;
    if (this.hasCapability('bolt.file-hash')) {
      fileHash = await hashFile(file);
    }

    console.log(`[TRANSFER] Sending ${file.name} (${file.size} bytes, ${totalChunks} chunks, tid=${transferId})`);
    this.transferCancelled = false;
    this.transferPaused = false;
    this.transferStartTime = Date.now();
    this.pauseDuration = 0;
    this.lastPausedAt = null;

    if (ENABLE_TRANSFER_METRICS) {
      this.metricsCollector = new TransferMetricsCollector();
      this.metricsFirstProgressRecorded = false;
      this.metricsCollector.begin(transferId, file.size, DEFAULT_CHUNK_SIZE, totalChunks);
    }

    try {
      for (let i = 0; i < totalChunks; i++) {
        if (this.transferCancelled) throw new TransferError('Transfer cancelled by user');

        while (this.transferPaused) {
          await new Promise(r => setTimeout(r, 100));
          if (this.transferCancelled) throw new TransferError('Transfer cancelled while paused');
        }

        const start = i * DEFAULT_CHUNK_SIZE;
        const end = Math.min(start + DEFAULT_CHUNK_SIZE, file.size);
        const raw = new Uint8Array(await file.slice(start, end).arrayBuffer());

        if (!this.keyPair) throw new EncryptionError('No ephemeral key pair');
        const encrypted = sealBoxPayload(raw, this.remotePublicKey!, this.keyPair.secretKey);

        const msg: FileChunkMessage = {
          type: 'file-chunk',
          filename: file.name,
          chunk: encrypted,
          chunkIndex: i,
          totalChunks,
          fileSize: file.size,
          transferId,
          ...(fileHash && i === 0 ? { fileHash } : {}),
        };

        // Backpressure — wait for buffer to drain (N1: cancelable by disconnect)
        if (this.dc!.bufferedAmount > this.dc!.bufferedAmountLowThreshold) {
          const gen = this.sessionGeneration;
          this.metricsCollector?.enterBufferDrainWait();
          await new Promise<void>((resolve, reject) => {
            this.backpressureReject = reject;
            this.dc!.onbufferedamountlow = () => {
              this.backpressureReject = undefined;
              this.metricsCollector?.exitBufferDrainWait();
              if (this.dc) this.dc.onbufferedamountlow = null;
              if (gen !== this.sessionGeneration) {
                reject(new TransferError('Transfer aborted: session ended'));
                return;
              }
              resolve();
            };
          });
        }

        if (this.transferCancelled) throw new TransferError('Transfer cancelled by user');

        this.metricsCollector?.recordChunkSend(this.dc!.bufferedAmount, i + 1);
        this.dcSendMessage(msg);
        this.emitProgress(file.name, i + 1, totalChunks, end, file.size, 'transferring');
      }

      console.log(`[TRANSFER] All chunks sent for ${file.name}`);
      this.sendTransferIds.delete(file.name);

      if (this.metricsCollector) {
        const metrics = this.metricsCollector.finish();
        this.metricsCollector = null;
        this.metricsFirstProgressRecorded = false;
        if (metrics) console.log('[TRANSFER_METRICS]', JSON.stringify(summarizeTransfer(metrics)));
      }

      // Emit completion after a brief delay so UI can process final progress
      this.completionTimeout = setTimeout(() => {
        this.emitProgress(file.name, totalChunks, totalChunks, file.size, file.size, 'completed');
      }, 50);
    } catch (error) {
      this.sendTransferIds.delete(file.name);

      if (this.metricsCollector) {
        const metrics = this.metricsCollector.finish();
        this.metricsCollector = null;
        this.metricsFirstProgressRecorded = false;
        if (metrics) console.log('[TRANSFER_METRICS]', JSON.stringify(summarizeTransfer(metrics)));
      }

      if (!(error instanceof TransferError && error.message.includes('cancelled'))) {
        this.emitProgress(file.name, 0, totalChunks, 0, file.size, 'error');
      }
      throw error;
    }
  }

  // ─── File Transfer (Receive) ──────────────────────────────────────────

  private handleMessage(event: MessageEvent) {
    try {
      const msg = JSON.parse(event.data);

      // ─── HELLO routing (exactly-once) ──────────────────────────
      if (msg.type === 'hello' && msg.payload) {
        if (this.sessionState !== 'pre_hello') {
          console.warn('[DUPLICATE_HELLO] HELLO received after handshake complete — disconnecting');
          this.sendErrorAndDisconnect('DUPLICATE_HELLO', 'Duplicate HELLO');
          return;
        }
        this.processHello(msg).catch((error) => {
          console.error('[HELLO] Unexpected error:', error);
          this.sendErrorAndDisconnect('PROTOCOL_VIOLATION', 'Unexpected HELLO processing error');
        });
        return;
      }

      // ─── Pre-handshake gate ────────────────────────────────────
      if (this.sessionState === 'pre_hello') {
        console.warn('[INVALID_STATE] non-HELLO message before handshake complete — disconnecting');
        this.onError(new ConnectionError('Received message before handshake complete'));
        this.sendErrorAndDisconnect('INVALID_STATE', 'Handshake not complete');
        return;
      }

      // ─── Profile Envelope v1: unwrap if negotiated ─────────────
      if (msg.type === 'profile-envelope') {
        if (!this.negotiatedEnvelopeV1()) {
          console.warn('[ENVELOPE_UNNEGOTIATED] profile-envelope received but not negotiated — disconnecting');
          this.sendErrorAndDisconnect('ENVELOPE_UNNEGOTIATED', 'Profile envelope not negotiated');
          return;
        }
        if (msg.version !== 1 || msg.encoding !== 'base64' || typeof msg.payload !== 'string') {
          console.warn('[ENVELOPE_INVALID] invalid profile-envelope version/encoding — disconnecting');
          this.sendErrorAndDisconnect('ENVELOPE_INVALID', 'Invalid profile envelope format');
          return;
        }
        // Decrypt envelope payload
        let innerBytes: Uint8Array;
        try {
          innerBytes = openBoxPayload(msg.payload, this.remotePublicKey!, this.keyPair!.secretKey);
        } catch {
          console.warn('[ENVELOPE_DECRYPT_FAIL] failed to decrypt profile-envelope — disconnecting');
          this.sendErrorAndDisconnect('ENVELOPE_DECRYPT_FAIL', 'Failed to decrypt profile envelope');
          return;
        }
        // Parse inner JSON
        let inner: any;
        try {
          inner = JSON.parse(new TextDecoder().decode(innerBytes));
        } catch {
          console.warn('[INVALID_MESSAGE] failed to parse inner message JSON — disconnecting');
          this.sendErrorAndDisconnect('INVALID_MESSAGE', 'Invalid inner message');
          return;
        }
        // Handle enveloped error from remote peer
        if (inner.type === 'error') {
          // Validate inbound error code: must be a non-empty string in the canonical registry
          if (!isValidWireErrorCode(inner.code)) {
            console.warn(`[PROTOCOL_VIOLATION] enveloped error with invalid code: ${JSON.stringify(inner.code)} — disconnecting`);
            this.sendErrorAndDisconnect('PROTOCOL_VIOLATION', 'Invalid inbound error code');
            return;
          }
          // Validate message field if present: must be a string
          if (inner.message !== undefined && typeof inner.message !== 'string') {
            console.warn(`[PROTOCOL_VIOLATION] enveloped error with non-string message — disconnecting`);
            this.sendErrorAndDisconnect('PROTOCOL_VIOLATION', 'Invalid inbound error message type');
            return;
          }
          console.warn(`[REMOTE_ERROR] enveloped error: code=${inner.code}, message=${inner.message}`);
          this.onError(new WebRTCError(inner.message || 'Remote error'));
          this.disconnect();
          return;
        }
        // Validate inner message type
        if (inner.type !== 'file-chunk') {
          console.warn(`[UNKNOWN_MESSAGE_TYPE] unknown inner type "${inner.type}" — disconnecting`);
          this.sendErrorAndDisconnect('UNKNOWN_MESSAGE_TYPE', `Unknown message type: ${inner.type}`);
          return;
        }
        this.routeInnerMessage(inner);
        return;
      }

      // ─── Envelope-required enforcement ─────────────────────────
      if (this.negotiatedEnvelopeV1()) {
        console.warn('[ENVELOPE_REQUIRED] plaintext message received in envelope-required session — disconnecting');
        this.sendErrorAndDisconnect('ENVELOPE_REQUIRED', 'Envelope required');
        return;
      }

      // ─── Plaintext error handling (pre-envelope) ────────────────
      if (msg.type === 'error') {
        if (!isValidWireErrorCode(msg.code)) {
          console.warn(`[PROTOCOL_VIOLATION] plaintext error with invalid code: ${JSON.stringify(msg.code)} — disconnecting`);
          this.sendErrorAndDisconnect('PROTOCOL_VIOLATION', 'Invalid inbound error code');
          return;
        }
        if (msg.message !== undefined && typeof msg.message !== 'string') {
          console.warn(`[PROTOCOL_VIOLATION] plaintext error with non-string message — disconnecting`);
          this.sendErrorAndDisconnect('PROTOCOL_VIOLATION', 'Invalid inbound error message type');
          return;
        }
        console.warn(`[REMOTE_ERROR] plaintext error: code=${msg.code}, message=${msg.message}`);
        this.onError(new WebRTCError(msg.message || 'Remote error'));
        this.disconnect();
        return;
      }

      // ─── Envelope-unnegotiated plaintext path ──────────────────
      // SA9: reject unknown type (non-file-chunk) — no silent drops
      if (msg.type !== 'file-chunk') {
        console.warn(`[UNKNOWN_MESSAGE_TYPE] unknown plaintext type "${msg.type}" — disconnecting`);
        this.sendErrorAndDisconnect('UNKNOWN_MESSAGE_TYPE', `Unknown message type: ${msg.type}`);
        return;
      }
      // SA9: reject malformed file-chunk (missing/empty filename) — no silent drops
      if (!msg.filename) {
        console.warn('[INVALID_MESSAGE] file-chunk with missing/empty filename — disconnecting');
        this.sendErrorAndDisconnect('INVALID_MESSAGE', 'file-chunk missing filename');
        return;
      }
      this.routeInnerMessage(msg);
    } catch (error) {
      console.error('[PROTOCOL_VIOLATION] Error processing message:', error);
      this.sendErrorAndDisconnect('PROTOCOL_VIOLATION', 'Protocol violation');
    }
  }

  /** Route a decoded inner message (from envelope or legacy plaintext). */
  private routeInnerMessage(msg: any) {
    if (msg.type !== 'file-chunk' || !msg.filename) return;

    // Control messages
    if (msg.paused) {
      this.transferPaused = true;
      this.emitProgress(msg.filename, 0, 0, 0, 0, 'paused');
      return;
    }
    if (msg.resumed) {
      this.transferPaused = false;
      this.emitProgress(msg.filename, 0, 0, 0, 0, 'transferring');
      return;
    }
    if (msg.cancelled) {
      this.handleRemoteCancel(msg);
      return;
    }

    // Data chunk
    this.processChunk(msg);
  }

  private handleRemoteCancel(msg: FileChunkMessage) {
    const status = msg.cancelledBy === 'receiver' ? 'canceled_by_receiver' : 'canceled_by_sender';
    this.receiveBuffers.delete(msg.filename);
    if (msg.transferId) {
      this.guardedTransfers.delete(msg.transferId);
    }
    // Also clean up by filename lookup
    const recvTid = this.recvTransferIds.get(msg.filename);
    if (recvTid) {
      this.guardedTransfers.delete(recvTid);
      this.recvTransferIds.delete(msg.filename);
    }
    this.transferCancelled = true;
    this.emitProgress(msg.filename, 0, 0, 0, 0, status);
  }

  private isValidChunkFields(msg: FileChunkMessage): boolean {
    const { chunkIndex, totalChunks } = msg;
    if (!Number.isFinite(totalChunks) || !Number.isInteger(totalChunks!) || totalChunks! <= 0) {
      console.warn(`[REPLAY_OOB] invalid totalChunks=${totalChunks} — rejected`);
      return false;
    }
    if (!Number.isFinite(chunkIndex) || !Number.isInteger(chunkIndex!) || chunkIndex! < 0 || chunkIndex! >= totalChunks!) {
      console.warn(`[REPLAY_OOB] chunkIndex=${chunkIndex} out of range [0, ${totalChunks}) — rejected`);
      return false;
    }
    return true;
  }

  private processChunk(msg: FileChunkMessage) {
    if (!msg.chunk || typeof msg.chunkIndex !== 'number' || !msg.totalChunks || !msg.fileSize) return;
    if (!this.remotePublicKey) return;

    // Bounds check applies to BOTH modes
    if (!this.isValidChunkFields(msg)) return;

    if (msg.transferId && this.helloComplete) {
      this.processChunkGuarded(msg).catch((error) => {
        console.error(`[TRANSFER] Unhandled error in guarded path:`, error);
        this.onError(error instanceof WebRTCError ? error : new TransferError('Chunk processing failed', error));
      });
    } else {
      if (msg.transferId && !this.helloComplete) {
        console.warn('[REPLAY_UNGUARDED] transferId present but HELLO incomplete — falling back to legacy');
      } else {
        console.warn('[REPLAY_UNGUARDED] chunk received without transferId — legacy peer');
      }
      this.processChunkLegacy(msg);
    }
  }

  private async processChunkGuarded(msg: FileChunkMessage) {
    const { filename, chunk, chunkIndex, totalChunks, fileSize, transferId } = msg;
    const identityKey = this.remoteIdentityKey ? toBase64(this.remoteIdentityKey) : '';

    // Lookup or create guarded transfer
    let transfer = this.guardedTransfers.get(transferId!);
    if (!transfer) {
      console.log(`[TRANSFER] Receiving ${filename} (${fileSize} bytes, ${totalChunks} chunks, tid=${transferId})`);
      // Store expectedHash from first chunk if bolt.file-hash negotiated
      const expectedHash = (this.hasCapability('bolt.file-hash') && msg.fileHash) ? msg.fileHash : undefined;
      transfer = {
        transferId: transferId!,
        filename: filename,
        totalChunks: totalChunks!,
        fileSize: fileSize!,
        buffer: new Array(totalChunks!).fill(null),
        receivedSet: new Set(),
        remoteIdentityKey: identityKey,
        expectedHash,
      };
      this.guardedTransfers.set(transferId!, transfer);
      this.recvTransferIds.set(filename, transferId!);
      this.transferStartTime = Date.now();
      this.pauseDuration = 0;
    } else if (transfer.remoteIdentityKey !== identityKey) {
      // Same transferId but different sender identity — cross-peer collision
      console.warn(`[REPLAY_XFER_MISMATCH] transferId=${transferId} bound to different sender identity — ignored`);
      return;
    }

    // Dedup check
    if (transfer.receivedSet.has(chunkIndex!)) {
      console.warn(`[REPLAY_DUP] chunkIndex=${chunkIndex} already received for tid=${transferId} — ignored`);
      return;
    }

    try {
      if (!this.keyPair) throw new EncryptionError('No ephemeral key pair');
      const decrypted = openBoxPayload(chunk!, this.remotePublicKey!, this.keyPair.secretKey);
      transfer.buffer[chunkIndex!] = new Blob([decrypted as BlobPart]);
      transfer.receivedSet.add(chunkIndex!);

      const received = transfer.receivedSet.size;
      this.emitProgress(filename, received, totalChunks!, received * (fileSize! / totalChunks!), fileSize!, 'transferring');

      // Check completion
      if (received === totalChunks!) {
        const assembledBlob = new Blob(transfer.buffer as Blob[]);

        // Verify file integrity if expectedHash was provided (bolt.file-hash negotiated)
        if (transfer.expectedHash) {
          try {
            const actual = await hashFile(assembledBlob);
            if (actual !== transfer.expectedHash) {
              console.error(`[INTEGRITY_MISMATCH] expected=${transfer.expectedHash} actual=${actual} (tid=${transferId})`);
              this.guardedTransfers.delete(transferId!);
              this.recvTransferIds.delete(filename);
              this.emitProgress(filename, 0, totalChunks!, 0, fileSize!, 'error');
              this.onError(new IntegrityError('File integrity check failed: hash mismatch'));
              this.dcSendMessage({
                type: 'error',
                code: 'INTEGRITY_FAILED',
                message: 'File integrity check failed: hash mismatch',
              });
              this.disconnect();
              return;
            }
            console.log(`[INTEGRITY_OK] hash verified for ${filename} (tid=${transferId})`);
          } catch (hashError) {
            console.error(`[INTEGRITY_ERROR] failed to compute hash for ${filename}:`, hashError);
            // Fail-closed: treat hash computation failure as integrity failure
            this.guardedTransfers.delete(transferId!);
            this.recvTransferIds.delete(filename);
            this.emitProgress(filename, 0, totalChunks!, 0, fileSize!, 'error');
            this.onError(new IntegrityError('File integrity check failed: hash computation error'));
            this.disconnect();
            return;
          }
        }

        console.log(`[TRANSFER] Completed receiving ${filename} (tid=${transferId})`);
        this.guardedTransfers.delete(transferId!);
        this.recvTransferIds.delete(filename);
        this.emitProgress(filename, totalChunks!, totalChunks!, fileSize!, fileSize!, 'completed');
        this.onReceiveFile(assembledBlob, filename);
      }
    } catch (error) {
      console.error(`[TRANSFER] Error processing chunk ${chunkIndex} of ${filename} (tid=${transferId}):`, error);
      this.guardedTransfers.delete(transferId!);
      this.recvTransferIds.delete(filename);
      this.emitProgress(filename, 0, totalChunks!, 0, fileSize!, 'error');
      this.onError(error instanceof WebRTCError ? error : new TransferError('Chunk processing failed', error));
    }
  }

  private processChunkLegacy(msg: FileChunkMessage) {
    const { filename, chunk, chunkIndex, totalChunks, fileSize } = msg;

    // Initialize buffer on first chunk
    if (!this.receiveBuffers.has(filename)) {
      console.log(`[TRANSFER] Receiving ${filename} (${fileSize} bytes, ${totalChunks} chunks) [legacy]`);
      this.receiveBuffers.set(filename, new Array(totalChunks!).fill(null));
      this.transferStartTime = Date.now();
      this.pauseDuration = 0;
    }

    try {
      if (!this.keyPair) throw new EncryptionError('No ephemeral key pair');
      const decrypted = openBoxPayload(chunk!, this.remotePublicKey!, this.keyPair.secretKey);
      const buffer = this.receiveBuffers.get(filename)!;
      buffer[chunkIndex!] = new Blob([decrypted as BlobPart]);

      const received = buffer.filter(Boolean).length;
      this.emitProgress(filename, received, totalChunks!, received * (fileSize! / totalChunks!), fileSize!, 'transferring');

      // Check completion
      if (received === totalChunks!) {
        console.log(`[TRANSFER] Completed receiving ${filename} [legacy]`);
        const file = new Blob(buffer as Blob[]);
        this.receiveBuffers.delete(filename);
        this.emitProgress(filename, totalChunks!, totalChunks!, fileSize!, fileSize!, 'completed');
        this.onReceiveFile(file, filename);
      }
    } catch (error) {
      console.error(`[TRANSFER] Error processing chunk ${chunkIndex} of ${filename}:`, error);
      this.receiveBuffers.delete(filename);
      this.emitProgress(filename, 0, totalChunks!, 0, fileSize!, 'error');
      this.onError(error instanceof WebRTCError ? error : new TransferError('Chunk processing failed', error));
    }
  }

  // ─── Transfer Control ──────────────────────────────────────────────────

  pauseTransfer(filename: string) {
    this.transferPaused = true;
    this.lastPausedAt = Date.now();
    this.metricsCollector?.markPaused();
    const transferId = this.sendTransferIds.get(filename);
    this.sendControlMessage(filename, { paused: true, ...(transferId && { transferId }) });
    this.emitProgress(filename, 0, 0, 0, 0, 'paused');
  }

  resumeTransfer(filename: string) {
    if (this.lastPausedAt) {
      this.pauseDuration += Date.now() - this.lastPausedAt;
      this.lastPausedAt = null;
    }
    this.transferPaused = false;
    this.metricsCollector?.markResumed();
    const transferId = this.sendTransferIds.get(filename);
    this.sendControlMessage(filename, { resumed: true, ...(transferId && { transferId }) });
    this.emitProgress(filename, 0, 0, 0, 0, 'transferring');
  }

  cancelTransfer(filename: string, isReceiver: boolean = false) {
    this.transferCancelled = true;
    const transferId = isReceiver
      ? this.recvTransferIds.get(filename)
      : this.sendTransferIds.get(filename);
    this.sendControlMessage(filename, {
      cancelled: true,
      cancelledBy: isReceiver ? 'receiver' : 'sender',
      ...(transferId && { transferId }),
    });
    this.receiveBuffers.delete(filename);
    if (transferId) {
      this.guardedTransfers.delete(transferId);
      this.recvTransferIds.delete(filename);
      this.sendTransferIds.delete(filename);
    }
    const status = isReceiver ? 'canceled_by_receiver' : 'canceled_by_sender';
    this.emitProgress(filename, 0, 0, 0, 0, status);
  }

  private sendControlMessage(filename: string, fields: Partial<FileChunkMessage>) {
    if (!this.dc || this.dc.readyState !== 'open') return;
    this.dcSendMessage({ type: 'file-chunk', filename, ...fields });
  }

  // ─── Progress ──────────────────────────────────────────────────────────

  private emitProgress(
    filename: string,
    currentChunk: number,
    totalChunks: number,
    loaded: number,
    total: number,
    status: TransferProgress['status']
  ) {
    if (this.metricsCollector && !this.metricsFirstProgressRecorded) {
      this.metricsCollector.recordFirstProgress();
      this.metricsFirstProgressRecorded = true;
    }

    if (!this.onProgressCallback) return;

    const elapsed = Math.max(1, Date.now() - this.transferStartTime - this.pauseDuration);
    const speed = loaded > 0 ? loaded / (elapsed / 1000) : 0;
    const remaining = speed > 0 ? (total - loaded) / speed : 0;

    this.onProgressCallback({
      filename,
      currentChunk,
      totalChunks,
      loaded,
      total,
      status,
      stats: {
        speed,
        averageSpeed: speed,
        estimatedTimeRemaining: remaining,
        retryCount: 0,
        maxRetries: 0,
        startTime: this.transferStartTime,
        pauseDuration: this.pauseDuration,
        lastPausedAt: this.lastPausedAt ?? undefined,
      },
    });
  }

  // ─── Public Accessors ──────────────────────────────────────────────────

  setProgressCallback(callback: (progress: TransferProgress) => void) {
    this.onProgressCallback = callback;
  }

  getRemotePeerCode(): string {
    return this.remotePeerCode;
  }

  setConnectionStateHandler(handler: (state: RTCPeerConnectionState) => void) {
    this.connectionStateHandler = handler;
  }

  /** Get current SAS verification state. */
  getVerificationInfo(): VerificationInfo {
    return this.verificationInfo;
  }

  /** Check whether a capability was successfully negotiated with the remote peer. */
  hasCapability(name: string): boolean {
    return this.negotiatedCapabilities.includes(name);
  }

  // ─── Profile Envelope v1 ─────────────────────────────────────────────

  /** Whether profile-envelope-v1 was mutually negotiated. */
  private negotiatedEnvelopeV1(): boolean {
    return this.hasCapability('bolt.profile-envelope-v1');
  }

  /** Encrypt an inner message into a ProfileEnvelopeV1 wire object. */
  private encodeProfileEnvelopeV1(innerMsg: object): ProfileEnvelopeV1 {
    const innerJson = JSON.stringify(innerMsg);
    const innerBytes = new TextEncoder().encode(innerJson);
    const payload = sealBoxPayload(innerBytes, this.remotePublicKey!, this.keyPair!.secretKey);
    return { type: 'profile-envelope', version: 1, encoding: 'base64', payload };
  }

  // SA18: decodeProfileEnvelopeV1 removed — dead code with silent null return.
  // Inline decryption in handleMessage() is the active path (fail-closed).

  /**
   * Send a message over the DataChannel, wrapping in profile-envelope when negotiated.
   * MUST only be called after helloComplete === true (except for pre-handshake error messages).
   */
  private dcSendMessage(innerMsg: any): void {
    if (!this.dc || this.dc.readyState !== 'open') return;
    if (this.negotiatedEnvelopeV1() && this.helloComplete && this.keyPair && this.remotePublicKey) {
      this.dc.send(JSON.stringify(this.encodeProfileEnvelopeV1(innerMsg)));
    } else {
      this.dc.send(JSON.stringify(innerMsg));
    }
  }

  /** Mark the current peer as verified. Persists to pin store. */
  async markPeerVerified(): Promise<void> {
    if (!this.options.pinStore || !this.remotePeerCode) return;
    await this.options.pinStore.markVerified(this.remotePeerCode);
    this.verificationInfo = { ...this.verificationInfo, state: 'verified' };
    this.options.onVerificationState?.(this.verificationInfo);
  }
}

export default WebRTCService;
