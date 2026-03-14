// ─── WS Transport (PM-RC-02) ────────────────────────────────────────────────
// WebSocket-direct primary transport for browser-to-daemon communication,
// with automatic WebRTC fallback.

export { WsDataTransport } from './WsDataTransport.js';
export type { WsDataTransportOptions, DataTransport } from './WsDataTransport.js';
export { BrowserAppTransport } from './BrowserAppTransport.js';
export type { BrowserAppTransportOptions } from './BrowserAppTransport.js';
