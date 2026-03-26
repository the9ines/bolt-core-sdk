// ─── @the9ines/bolt-transport-web ────────────────────────────────────────────
// Public API surface. Every exported symbol is listed here.
// This file is the ONLY supported entry point for consumers.

// ── Services ─────────────────────────────────────────────────────────────────

// Signaling, Identity, Components, State, UI — EXTRACTED to @the9ines/localbolt-browser.
// Import signaling, persistence, UI from @the9ines/localbolt-browser instead.

// WebRTC (legacy tribute — will move in Phase 3)
export { default as WebRTCService } from './services/webrtc/WebRTCService.js';
export type { TransferProgress, TransferStats, WebRTCServiceOptions, VerificationInfo, VerificationState } from './services/webrtc/WebRTCService.js';

// WS/WT Transport (forward-path browser↔app adapters)
export { WsDataTransport, WtDataTransport, BrowserAppTransport } from './services/ws-transport/index.js';
export type { WsDataTransportOptions, WtDataTransportOptions, DataTransport, BrowserAppTransportOptions } from './services/ws-transport/index.js';

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
