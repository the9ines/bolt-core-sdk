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

// WS Transport (PM-RC-02) — WebSocket-direct primary + WebRTC fallback
export { WsDataTransport, WtDataTransport, BrowserAppTransport } from './services/ws-transport/index.js';
export type { WsDataTransportOptions, WtDataTransportOptions, DataTransport, BrowserAppTransportOptions } from './services/ws-transport/index.js';

// Identity & TOFU
export { IndexedDBIdentityStore, getOrCreateIdentity, zeroizeIdentityKey } from './services/identity/identity-store.js';
export type { IdentityPersistence } from './services/identity/identity-store.js';
export { IndexedDBPinStore } from './services/identity/pin-store.js';
export type { PinPersistence, PinRecord, PinVerifyResult } from './services/identity/pin-store.js';

// ── Components, State, UI ────────────────────────────────────────────────────
// EXTRACTED to @the9ines/localbolt-browser (TS-EXTRACTION Phase 1).
// Product UI components, store, icons, toast, and sanitize no longer
// live in bolt-core-sdk. Import from @the9ines/localbolt-browser instead.

// ── Lib ──────────────────────────────────────────────────────────────────────
export {
  detectDevice,
  getDeviceName as getPlatformDeviceName,
  getMaxChunkSize,
  getPlatformICEServers,
  getLocalOnlyRTCConfig,
  isPrivateIP,
  isLocalCandidate,
} from './lib/platform-utils.js';

// ── Protocol WASM (BR2+BR3) ─────────────────────────────────────────────────
export { initProtocolWasm, getProtocolAuthorityMode } from './services/webrtc/ProtocolWasmLoader.js';
export type { ProtocolAuthorityMode } from './services/webrtc/ProtocolWasmLoader.js';

// ── Transfer Metrics (PF2) ──────────────────────────────────────────────────
export { setTransferMetricsEnabled } from './services/webrtc/transferMetrics.js';

// ── Policy ──────────────────────────────────────────────────────────────────
export { initPolicyAdapter, getPolicyAdapter } from './services/webrtc/PolicyAdapter.js';
export type { PolicyAdapter, ScheduleDecision, StallResult, ProgressResult } from './services/webrtc/PolicyAdapter.js';

// ── Types ────────────────────────────────────────────────────────────────────
export { SignalingError } from './types/webrtc-errors.js';
export { WebRTCError, ConnectionError, TransferError, EncryptionError } from './types/webrtc-errors.js';
