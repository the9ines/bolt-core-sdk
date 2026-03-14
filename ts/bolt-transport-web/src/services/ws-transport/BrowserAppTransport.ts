/**
 * BrowserAppTransport -- orchestrator for browser-to-daemon transport.
 *
 * Strategy: WS primary -> WebRTC fallback.
 *
 * 1. Attempts direct WebSocket connection to the daemon.
 * 2. On WS failure (refused, timeout, TLS error), falls back to WebRTC
 *    via existing DualSignaling + WebRTCService path.
 *
 * The active transport is transparent to the caller after connect().
 */

import { WsDataTransport } from './WsDataTransport.js';
import type { WsDataTransportOptions } from './WsDataTransport.js';
import type WebRTCService from '../webrtc/WebRTCService.js';
import type { TransferProgress, VerificationInfo } from '../webrtc/types.js';

// ─── Options ────────────────────────────────────────────────────────────────

export interface BrowserAppTransportOptions {
  /** Daemon WebSocket URL, e.g. "ws://localhost:9100" */
  daemonUrl: string;
  /** WS connection timeout in ms. Default: 5000 */
  wsConnectTimeout?: number;

  // ── WebRTC fallback options ──
  /** Signaling server URL for WebRTC fallback. */
  signalingUrl?: string;
  /** Remote peer code for WebRTC fallback. */
  peerCode?: string;

  // ── Shared options ──
  /** Ed25519 identity keypair. */
  identity?: { publicKey: Uint8Array; secretKey: Uint8Array };
  /** Ed25519 identity public key. */
  identityPublicKey?: Uint8Array;
  /** Verification state callback. */
  onVerification?: (info: VerificationInfo) => void;
  /** Fired when a complete file is received. */
  onReceiveFile?: (file: Blob, metadata: { filename: string }) => void;
  /** Transfer progress callback. */
  onProgress?: (progress: TransferProgress) => void;
  /** Fired with the active transport mode ('ws' | 'webrtc'). */
  onTransportMode?: (mode: 'ws' | 'webrtc') => void;
  /** Error callback. */
  onError?: (error: Error) => void;
  /** Enable BTR capability. Default: false. */
  btrEnabled?: boolean;

  /**
   * Factory for creating a WebRTC fallback service.
   * Injected to avoid hard dependency on signaling setup.
   * If not provided, WebRTC fallback is unavailable and connect()
   * will throw when WS fails.
   */
  createWebRTCFallback?: () => Promise<WebRTCService>;
}

// ─── BrowserAppTransport ────────────────────────────────────────────────────

export class BrowserAppTransport {
  private wsTransport: WsDataTransport | null = null;
  private webrtcService: WebRTCService | null = null;
  private transportMode: 'ws' | 'webrtc' | null = null;
  private readonly options: BrowserAppTransportOptions;

  constructor(options: BrowserAppTransportOptions) {
    this.options = options;
  }

  /** Current transport mode, or null if not connected. */
  get mode(): 'ws' | 'webrtc' | null {
    return this.transportMode;
  }

  /**
   * Connect using WS primary with WebRTC fallback.
   * Throws if both transports fail.
   */
  async connect(): Promise<void> {
    // 1. Try WS primary
    const wsOk = await this.tryWsConnect();
    if (wsOk) {
      this.transportMode = 'ws';
      this.options.onTransportMode?.('ws');
      console.log('[WS_TRANSPORT] Using WebSocket transport');
      return;
    }

    // 2. WS failed -> automatic WebRTC fallback
    console.log('[TRANSPORT_FALLBACK] WS failed, falling back to WebRTC');
    await this.connectWebRTC();
    this.transportMode = 'webrtc';
    this.options.onTransportMode?.('webrtc');
    console.log('[TRANSPORT_FALLBACK] Using WebRTC transport');
  }

  /**
   * Attempt WS connection with timeout.
   * Returns true on success, false on failure.
   */
  private async tryWsConnect(): Promise<boolean> {
    try {
      this.wsTransport = new WsDataTransport({
        daemonUrl: this.options.daemonUrl,
        connectTimeout: this.options.wsConnectTimeout ?? 5000,
        identity: this.options.identity,
        identityPublicKey: this.options.identityPublicKey,
        onVerification: this.options.onVerification,
        onReceiveFile: this.options.onReceiveFile,
        onProgress: this.options.onProgress,
        onTransportMode: this.options.onTransportMode,
        onError: this.options.onError,
        btrEnabled: this.options.btrEnabled,
        onDisconnect: () => {
          console.log('[WS_TRANSPORT] Disconnected from daemon');
        },
      });

      const ok = await this.wsTransport.connect();
      if (!ok) {
        this.wsTransport = null;
        return false;
      }
      return true;
    } catch {
      this.wsTransport = null;
      return false;
    }
  }

  /**
   * Connect via WebRTC using the injected factory.
   * Throws if no factory provided or connection fails.
   */
  private async connectWebRTC(): Promise<void> {
    if (!this.options.createWebRTCFallback) {
      throw new Error('[TRANSPORT_FALLBACK] WebRTC fallback not configured — no createWebRTCFallback factory');
    }

    this.webrtcService = await this.options.createWebRTCFallback();

    if (this.options.peerCode) {
      await this.webrtcService.connect(this.options.peerCode);
    }
  }

  /** Send a file via the active transport. */
  async sendFile(file: File): Promise<void> {
    if (this.transportMode === 'ws' && this.wsTransport) {
      return this.wsTransport.sendFile(file);
    }
    if (this.transportMode === 'webrtc' && this.webrtcService) {
      return this.webrtcService.sendFile(file);
    }
    throw new Error('No active transport — call connect() first');
  }

  /** Disconnect the active transport. */
  disconnect(): void {
    if (this.wsTransport) {
      this.wsTransport.disconnect();
      this.wsTransport = null;
    }
    if (this.webrtcService) {
      this.webrtcService.disconnect();
      this.webrtcService = null;
    }
    this.transportMode = null;
  }
}
