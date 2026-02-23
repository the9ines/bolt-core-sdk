import { sealBoxPayload, openBoxPayload, generateEphemeralKeyPair, toBase64, fromBase64, DEFAULT_CHUNK_SIZE, EncryptionError, KeyMismatchError, computeSas } from '@the9ines/bolt-core';
import { WebRTCError, ConnectionError, SignalingError, TransferError } from '../../types/webrtc-errors.js';
import { getLocalOnlyRTCConfig } from '../../lib/platform-utils.js';
import { verifyPinnedIdentity } from '../identity/pin-store.js';
import type { PinPersistence } from '../identity/pin-store.js';
import type { SignalingProvider, SignalMessage } from '../signaling/index.js';

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
  cancelled?: boolean;
  cancelledBy?: 'sender' | 'receiver';
  paused?: boolean;
  resumed?: boolean;
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

  // Stats
  private transferStartTime = 0;
  private pauseDuration = 0;
  private lastPausedAt: number | null = null;

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
  private helloTimeout: ReturnType<typeof setTimeout> | null = null;
  private helloResolve: (() => void) | null = null;

  // SAS verification state
  private remoteIdentityKey: Uint8Array | null = null;
  private verificationInfo: VerificationInfo = { state: 'legacy', sasCode: null };

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
    this.signaling.onSignal((signal) => this.handleSignal(signal));
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

  // ─── HELLO Protocol ──────────────────────────────────────────────────────

  private initiateHello() {
    if (!this.options.identityPublicKey || !this.keyPair || !this.remotePublicKey) {
      // No identity configured — this node operates in legacy mode
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
    });
    const plaintext = new TextEncoder().encode(hello);
    const encrypted = sealBoxPayload(plaintext, this.remotePublicKey, this.keyPair.secretKey);
    this.dc!.send(JSON.stringify({ type: 'hello', payload: encrypted }));
    console.log('[HELLO] Sent encrypted HELLO');

    // Start timeout — if remote doesn't send HELLO, treat as legacy
    this.helloTimeout = setTimeout(() => {
      if (!this.helloComplete) {
        console.warn('[TOFU_LEGACY_PEER] No HELLO received within timeout, treating as legacy peer');
        this.helloComplete = true;
        this.sessionLegacy = true;
        this.verificationInfo = { state: 'legacy', sasCode: null };
        this.options.onVerificationState?.(this.verificationInfo);
        if (this.helloResolve) {
          this.helloResolve();
          this.helloResolve = null;
        }
      }
    }, HELLO_TIMEOUT_MS);
  }

  private async processHello(msg: { type: 'hello'; payload: string }) {
    if (!this.keyPair || !this.remotePublicKey) {
      console.warn('[HELLO] Cannot decrypt — no ephemeral keys');
      return;
    }

    const decrypted = openBoxPayload(msg.payload, this.remotePublicKey, this.keyPair.secretKey);
    const hello = JSON.parse(new TextDecoder().decode(decrypted));

    if (hello.type !== 'hello' || hello.version !== 1 || !hello.identityPublicKey) {
      console.warn('[HELLO] Invalid HELLO format, ignoring');
      return;
    }

    const remoteIdentityKey = fromBase64(hello.identityPublicKey);
    this.remoteIdentityKey = remoteIdentityKey;
    console.log('[HELLO] Received identity from peer', this.remotePeerCode);

    // TOFU verification — determines verification state
    let verificationState: VerificationState = 'unverified';

    if (this.options.pinStore) {
      const result = await verifyPinnedIdentity(
        this.options.pinStore,
        this.remotePeerCode,
        remoteIdentityKey,
      );
      // KeyMismatchError propagates to caller (handleMessage catch → abort session)
      if (result.outcome === 'pinned') {
        console.log('[TOFU] First contact — pinned identity for', this.remotePeerCode);
        verificationState = 'unverified';
      } else {
        console.log('[TOFU] Identity verified for', this.remotePeerCode);
        verificationState = result.verified ? 'verified' : 'unverified';
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

    // HELLO complete
    if (this.helloTimeout) {
      clearTimeout(this.helloTimeout);
      this.helloTimeout = null;
    }
    this.helloComplete = true;
    this.sessionLegacy = false;
    if (this.helloResolve) {
      this.helloResolve();
      this.helloResolve = null;
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

    if (this.dc) {
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
    this.remotePublicKey = null;
    this.remotePeerCode = '';
    this.pendingCandidates = [];
    this.transferPaused = false;
    this.transferCancelled = false;
    this.receiveBuffers.clear();

    // Clear HELLO / TOFU / SAS state
    if (this.helloTimeout) {
      clearTimeout(this.helloTimeout);
      this.helloTimeout = null;
    }
    this.helloComplete = false;
    this.sessionLegacy = false;
    this.helloResolve = null;
    this.remoteIdentityKey = null;
    this.verificationInfo = { state: 'legacy', sasCode: null };
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
    console.log(`[TRANSFER] Sending ${file.name} (${file.size} bytes, ${totalChunks} chunks)`);
    this.transferCancelled = false;
    this.transferPaused = false;
    this.transferStartTime = Date.now();
    this.pauseDuration = 0;
    this.lastPausedAt = null;

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
        };

        // Backpressure — wait for buffer to drain
        if (this.dc!.bufferedAmount > this.dc!.bufferedAmountLowThreshold) {
          await new Promise<void>(resolve => {
            this.dc!.onbufferedamountlow = () => {
              this.dc!.onbufferedamountlow = null;
              resolve();
            };
          });
        }

        if (this.transferCancelled) throw new TransferError('Transfer cancelled by user');

        this.dc!.send(JSON.stringify(msg));
        this.emitProgress(file.name, i + 1, totalChunks, end, file.size, 'transferring');
      }

      console.log(`[TRANSFER] All chunks sent for ${file.name}`);
      // Emit completion after a brief delay so UI can process final progress
      setTimeout(() => {
        this.emitProgress(file.name, totalChunks, totalChunks, file.size, file.size, 'completed');
      }, 50);
    } catch (error) {
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

      // Route HELLO messages (encrypted identity exchange)
      if (msg.type === 'hello' && msg.payload) {
        this.processHello(msg).catch((error) => {
          if (error instanceof KeyMismatchError) {
            console.error('[TOFU] IDENTITY MISMATCH — aborting session:', error.message);
            this.onError(new ConnectionError('Identity key mismatch (TOFU violation)', error));
            this.disconnect();
          } else {
            console.error('[HELLO] Failed to process:', error);
          }
        });
        return;
      }

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
    } catch (error) {
      console.error('[RECV] Error processing message:', error);
    }
  }

  private handleRemoteCancel(msg: FileChunkMessage) {
    const status = msg.cancelledBy === 'receiver' ? 'canceled_by_receiver' : 'canceled_by_sender';
    this.receiveBuffers.delete(msg.filename);
    this.transferCancelled = true;
    this.emitProgress(msg.filename, 0, 0, 0, 0, status);
  }

  private processChunk(msg: FileChunkMessage) {
    if (!msg.chunk || typeof msg.chunkIndex !== 'number' || !msg.totalChunks || !msg.fileSize) return;
    if (!this.remotePublicKey) return;

    const { filename, chunk, chunkIndex, totalChunks, fileSize } = msg;

    // Initialize buffer on first chunk
    if (!this.receiveBuffers.has(filename)) {
      console.log(`[TRANSFER] Receiving ${filename} (${fileSize} bytes, ${totalChunks} chunks)`);
      this.receiveBuffers.set(filename, new Array(totalChunks).fill(null));
      this.transferStartTime = Date.now();
      this.pauseDuration = 0;
    }

    try {
      if (!this.keyPair) throw new EncryptionError('No ephemeral key pair');
      const decrypted = openBoxPayload(chunk, this.remotePublicKey, this.keyPair.secretKey);
      const buffer = this.receiveBuffers.get(filename)!;
      buffer[chunkIndex] = new Blob([decrypted as BlobPart]);

      const received = buffer.filter(Boolean).length;
      this.emitProgress(filename, received, totalChunks, received * (fileSize / totalChunks), fileSize, 'transferring');

      // Check completion
      if (received === totalChunks) {
        console.log(`[TRANSFER] Completed receiving ${filename}`);
        const file = new Blob(buffer as Blob[]);
        this.receiveBuffers.delete(filename);
        this.emitProgress(filename, totalChunks, totalChunks, fileSize, fileSize, 'completed');
        this.onReceiveFile(file, filename);
      }
    } catch (error) {
      console.error(`[TRANSFER] Error processing chunk ${chunkIndex} of ${filename}:`, error);
      this.receiveBuffers.delete(filename);
      this.emitProgress(filename, 0, totalChunks, 0, fileSize, 'error');
      this.onError(error instanceof WebRTCError ? error : new TransferError('Chunk processing failed', error));
    }
  }

  // ─── Transfer Control ──────────────────────────────────────────────────

  pauseTransfer(filename: string) {
    this.transferPaused = true;
    this.lastPausedAt = Date.now();
    this.sendControlMessage(filename, { paused: true });
    this.emitProgress(filename, 0, 0, 0, 0, 'paused');
  }

  resumeTransfer(filename: string) {
    if (this.lastPausedAt) {
      this.pauseDuration += Date.now() - this.lastPausedAt;
      this.lastPausedAt = null;
    }
    this.transferPaused = false;
    this.sendControlMessage(filename, { resumed: true });
    this.emitProgress(filename, 0, 0, 0, 0, 'transferring');
  }

  cancelTransfer(filename: string, isReceiver: boolean = false) {
    this.transferCancelled = true;
    this.sendControlMessage(filename, { cancelled: true, cancelledBy: isReceiver ? 'receiver' : 'sender' });
    this.receiveBuffers.delete(filename);
    const status = isReceiver ? 'canceled_by_receiver' : 'canceled_by_sender';
    this.emitProgress(filename, 0, 0, 0, 0, status);
  }

  private sendControlMessage(filename: string, fields: Partial<FileChunkMessage>) {
    if (!this.dc || this.dc.readyState !== 'open') return;
    this.dc.send(JSON.stringify({ type: 'file-chunk', filename, ...fields }));
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

  /** Mark the current peer as verified. Persists to pin store. */
  async markPeerVerified(): Promise<void> {
    if (!this.options.pinStore || !this.remotePeerCode) return;
    await this.options.pinStore.markVerified(this.remotePeerCode);
    this.verificationInfo = { ...this.verificationInfo, state: 'verified' };
    this.options.onVerificationState?.(this.verificationInfo);
  }
}

export default WebRTCService;
