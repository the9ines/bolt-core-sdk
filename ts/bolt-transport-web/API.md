# @the9ines/bolt-transport-web — Public API

All exports originate from `src/index.ts`. No other import paths are supported.

## Services

### Signaling

| Export | Kind | Source | Purpose |
|--------|------|--------|---------|
| `SignalingProvider` | type | `SignalingProvider.ts` | Abstract signaling transport interface |
| `SignalMessage` | type | `SignalingProvider.ts` | Wire format for signaling messages |
| `DiscoveredDevice` | type | `SignalingProvider.ts` | Peer device descriptor (code, name, type) |
| `WebSocketSignaling` | class | `WebSocketSignaling.ts` | WebSocket-based signaling with auto-reconnect |
| `DualSignaling` | class | `DualSignaling.ts` | Merges local + cloud signaling servers |
| `detectDeviceType` | function | `device-detect.ts` | Detect phone/tablet/laptop/desktop from UA |
| `getDeviceName` | function | `device-detect.ts` | Human-readable device name from UA |

### WebRTC

| Export | Kind | Source | Purpose |
|--------|------|--------|---------|
| `WebRTCService` | class | `WebRTCService.ts` | Encrypted P2P file transfer over WebRTC data channels |
| `TransferProgress` | type | `WebRTCService.ts` | File transfer progress descriptor |
| `TransferStats` | type | `WebRTCService.ts` | Speed, ETA, retry statistics |

## Components

| Export | Kind | Source | Purpose |
|--------|------|--------|---------|
| `createDeviceDiscovery` | function | `device-discovery.ts` | Device list / connection request UI |
| `createFileUpload` | function | `file-upload.ts` | Drag-drop file upload UI with progress |
| `setWebrtcRef` | function | `file-upload.ts` | Bind WebRTCService instance to file upload |
| `createTransferProgress` | function | `transfer-progress.ts` | Transfer progress bar with pause/cancel |
| `createConnectionStatus` | function | `connection-status.ts` | E2E encryption status indicator |

## State

| Export | Kind | Source | Purpose |
|--------|------|--------|---------|
| `store` | instance | `store.ts` | Global reactive state store |
| `AppState` | type | `store.ts` | Shape of global application state |
| `ConnectionRequest` | type | `store.ts` | Incoming connection request descriptor |

## UI

| Export | Kind | Source | Purpose |
|--------|------|--------|---------|
| `icons` | object | `icons.ts` | Inline Lucide SVG icon functions |
| `showToast` | function | `toast.ts` | Ephemeral toast notification |

## Lib

| Export | Kind | Source | Purpose |
|--------|------|--------|---------|
| `escapeHTML` | function | `sanitize.ts` | Escape string for safe innerHTML |
| `detectDevice` | function | `platform-utils.ts` | Full device info (OS, mobile, platform) |
| `getPlatformDeviceName` | function | `platform-utils.ts` | Friendly device name via platform detection |
| `getMaxChunkSize` | function | `platform-utils.ts` | Platform-appropriate WebRTC chunk size |
| `getPlatformICEServers` | function | `platform-utils.ts` | STUN server configuration |
| `getLocalOnlyRTCConfig` | function | `platform-utils.ts` | Local-network RTCConfiguration |
| `isPrivateIP` | function | `platform-utils.ts` | Check if IP is RFC1918/link-local |
| `isLocalCandidate` | function | `platform-utils.ts` | Check if ICE candidate is local-only |

## Types / Errors

| Export | Kind | Source | Purpose |
|--------|------|--------|---------|
| `WebRTCError` | class | `webrtc-errors.ts` | Base error (re-exported from bolt-core) |
| `ConnectionError` | class | `webrtc-errors.ts` | Connection failure error |
| `TransferError` | class | `webrtc-errors.ts` | File transfer error |
| `EncryptionError` | class | `webrtc-errors.ts` | Encryption/decryption error |
| `SignalingError` | class | `webrtc-errors.ts` | Signaling-specific error |
