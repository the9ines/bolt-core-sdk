/**
 * Shared types for the WebRTC service decomposition.
 *
 * Types formerly defined in WebRTCService.ts are canonical here;
 * WebRTCService.ts re-exports them to preserve the public API.
 */

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

export interface FileChunkMessage {
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
export interface ProfileEnvelopeV1 {
  type: 'profile-envelope';
  version: 1;
  encoding: 'base64';
  payload: string;
}

/** Receiver-side state for a guarded transfer (transferId present). */
export interface ActiveTransfer {
  transferId: string;
  filename: string;
  totalChunks: number;
  fileSize: number;
  buffer: (Blob | null)[];
  receivedSet: Set<number>;
  remoteIdentityKey: string;
  expectedHash?: string;
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
  pinStore?: import('../identity/pin-store.js').PinPersistence;
  /** Callback fired when verification state changes (after HELLO or on legacy timeout). */
  onVerificationState?: (info: VerificationInfo) => void;
}
