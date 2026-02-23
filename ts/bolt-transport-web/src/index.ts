// ─── @the9ines/bolt-transport-web ────────────────────────────────────────────
// Public API surface. Every exported symbol is listed here.
// This file is the ONLY supported entry point for consumers.

// ── Services ─────────────────────────────────────────────────────────────────

// Signaling
export type { SignalMessage, DiscoveredDevice, SignalingProvider } from './services/signaling/SignalingProvider.js';
export { WebSocketSignaling } from './services/signaling/WebSocketSignaling.js';
export { DualSignaling } from './services/signaling/DualSignaling.js';
export { detectDeviceType, getDeviceName } from './services/signaling/device-detect.js';

// WebRTC
export { default as WebRTCService } from './services/webrtc/WebRTCService.js';
export type { TransferProgress, TransferStats, WebRTCServiceOptions, VerificationInfo, VerificationState } from './services/webrtc/WebRTCService.js';

// Identity & TOFU
export { IndexedDBIdentityStore, MemoryIdentityStore, getOrCreateIdentity } from './services/identity/identity-store.js';
export type { IdentityPersistence } from './services/identity/identity-store.js';
export { IndexedDBPinStore, MemoryPinStore, verifyPinnedIdentity } from './services/identity/pin-store.js';
export type { PinPersistence, PinRecord, PinVerifyResult } from './services/identity/pin-store.js';

// ── Components ───────────────────────────────────────────────────────────────
export { createDeviceDiscovery } from './components/device-discovery.js';
export { createFileUpload, setWebrtcRef } from './components/file-upload.js';
export { createTransferProgress } from './components/transfer-progress.js';
export { createConnectionStatus } from './components/connection-status.js';
export { createVerificationStatus } from './components/verification-status.js';
export type { VerificationStatusOptions } from './components/verification-status.js';

// ── State ────────────────────────────────────────────────────────────────────
export { store } from './state/store.js';
export type { AppState, ConnectionRequest } from './state/store.js';

// ── UI ───────────────────────────────────────────────────────────────────────
export { icons } from './ui/icons.js';
export { showToast } from './ui/toast.js';

// ── Lib ──────────────────────────────────────────────────────────────────────
export { escapeHTML } from './lib/sanitize.js';
export {
  detectDevice,
  getDeviceName as getPlatformDeviceName,
  getMaxChunkSize,
  getPlatformICEServers,
  getLocalOnlyRTCConfig,
  isPrivateIP,
  isLocalCandidate,
} from './lib/platform-utils.js';

// ── Types ────────────────────────────────────────────────────────────────────
export { SignalingError } from './types/webrtc-errors.js';
export { WebRTCError, ConnectionError, TransferError, EncryptionError } from './types/webrtc-errors.js';
