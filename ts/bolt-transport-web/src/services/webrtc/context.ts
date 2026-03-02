/**
 * ConnectionContext — shared state surface for decomposed WebRTC managers.
 *
 * Created by WebRTCService and passed to HandshakeManager / TransferManager.
 * Provides read/write access to shared primitives without exposing
 * the full WebRTCService instance.
 */
import type { WebRTCServiceOptions, VerificationInfo } from './types.js';

export interface ConnectionContext {
  // ─── Crypto keys (per-session) ──────────────────────────────────
  getKeyPair(): { publicKey: Uint8Array; secretKey: Uint8Array } | null;
  getRemotePublicKey(): Uint8Array | null;

  // ─── DataChannel ────────────────────────────────────────────────
  getDc(): RTCDataChannel | null;

  // ─── Peer codes ─────────────────────────────────────────────────
  getLocalPeerCode(): string;
  getRemotePeerCode(): string;

  // ─── Options ────────────────────────────────────────────────────
  getOptions(): WebRTCServiceOptions;

  // ─── Cross-cutting callbacks ────────────────────────────────────

  /** Send error frame and disconnect (fatal protocol error). */
  onFatalError(code: string, message: string): void;

  /** Surface a WebRTCError to the consumer without disconnecting. */
  onError(error: Error): void;

  /** Tear down the entire connection. */
  disconnect(): void;

  // ─── Handshake state (written by HandshakeManager, read by others) ──
  isHelloComplete(): boolean;
  setHelloComplete(value: boolean): void;
  getSessionState(): 'pre_hello' | 'post_hello' | 'closed';
  setSessionState(value: 'pre_hello' | 'post_hello' | 'closed'): void;
  getSessionGeneration(): number;
  getVerificationInfo(): VerificationInfo;
  setVerificationInfo(info: VerificationInfo): void;
  getRemoteIdentityKey(): Uint8Array | null;
  setRemoteIdentityKey(key: Uint8Array | null): void;

  // ─── Capabilities (written by HandshakeManager, read by TransferManager) ──
  getNegotiatedCapabilities(): string[];
  setNegotiatedCapabilities(caps: string[]): void;
  setRemoteCapabilities(caps: string[]): void;
  hasCapability(name: string): boolean;

  // ─── File receive callback ─────────────────────────────────────
  onReceiveFile(file: Blob, filename: string): void;
}
