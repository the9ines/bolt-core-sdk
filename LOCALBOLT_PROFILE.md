# LocalBolt Profile v1

**Version:** 1.0.0
**Status:** Draft
**Date:** 2026-02-19
**Implements:** Bolt Core v1

---

## 1. Overview

- Profile for: local network file transfer via browser WebRTC
- Implements: Bolt Protocol Core v1
- Discovery scope: local network only (see section 7)

---

## 2. Signaling Transport

- Protocol: WebSocket (JSON messages)
- Endpoint: local LAN signaling server (`ws://<ip>:3001`)
- LocalBolt Profile v1 uses local-only signaling by default
- Signaling is explicitly non-security-critical (see Bolt Core section 12)
- Transport provides: message boundary preservation (WebSocket frames), TLS confidentiality when using `wss://` (defense-in-depth only)

### Cloud Discovery Extension

Cloud signaling (`wss://...`) is a separate optional extension. When enabled, it broadens discovery beyond the local network.

Implementations using cloud discovery:

- MUST clearly indicate to users that discovery scope extends beyond LAN
- MUST use a distinct UI mode or indicator (e.g. "Internet mode" vs "Local mode")
- MUST NOT present cloud-discovered peers as "local" peers

This extension does not change the Bolt Core protocol.

---

## 3. Signaling Wire Format

### Client -> Server

| Message | Format |
|---------|--------|
| Register | `{ "type": "register", "peer_code": "...", "device_name": "...", "device_type": "..." }` |
| Signal | `{ "type": "signal", "to": "<peer_code>", "payload": {...} }` |
| Ping | `{ "type": "ping" }` |

### Server -> Client

| Message | Format |
|---------|--------|
| Peers | `{ "type": "peers", "peers": [...] }` |
| Peer Joined | `{ "type": "peer_joined", "peer": {...} }` |
| Peer Left | `{ "type": "peer_left", "peer_code": "..." }` |
| Signal | `{ "type": "signal", "from": "...", "payload": {...} }` |
| Error | `{ "type": "error", "message": "..." }` |

### Peer Code Validation (server-side)

- Non-empty, max 16 chars, ASCII alphanumeric only

---

## 4. Data Transport

- WebRTC DataChannel
  - Label: `"fileTransfer"`
  - Ordered: `true`
  - Reliable: `true`
  - `binaryType`: `"arraybuffer"`
- RTCConfiguration:
  - `iceCandidatePoolSize`: 10
  - `bundlePolicy`: `"max-bundle"`
  - `rtcpMuxPolicy`: `"require"`
  - `iceTransportPolicy`: `"all"`

---

## 5. ICE Configuration

- STUN servers: `stun:stun.l.google.com:19302`, `stun:stun1.l.google.com:19302`
- TURN servers: none (local-only policy)
- Candidate filtering: `host` and `srflx` only; `relay` candidates BLOCKED

---

## 6. Message Encoding (`json-envelope-v1`)

All protected Bolt messages MUST be transmitted as encrypted envelopes.

Encoding identifier for HELLO negotiation: `json-envelope-v1`

### Envelope Wire Format

```json
{
  "type": "bolt-envelope",
  "senderEphemeralKey": "<base64, 32 bytes>",
  "nonce": "<base64, 24 bytes>",
  "ciphertext": "<base64>"
}
```

### Plaintext Message Serialization

The decrypted payload MUST be UTF-8 JSON representing exactly one canonical Bolt message.

### Byte Fields Encoding (inside decrypted plaintext)

| Field | Encoding |
|-------|----------|
| `transfer_id` (bytes16) | Hex string (32 hex chars) |
| `identity_key` (bytes32) | Base64 string |
| `file_hash` (bytes32) | Hex string (64 hex chars) |
| `payload` (bytes) | Base64 string |

### Example: Decrypted HELLO

```json
{
  "type": "hello",
  "boltVersion": 1,
  "capabilities": ["bolt.file-hash"],
  "encoding": "json-envelope-v1",
  "identityKey": "<base64, 32 bytes>"
}
```

### Example: Decrypted FILE_OFFER

```json
{
  "type": "file-offer",
  "transferId": "<hex, 32 chars>",
  "filename": "example.pdf",
  "size": 688128,
  "totalChunks": 42,
  "chunkSize": 16384,
  "fileHash": "<hex, 64 chars>"
}
```

### Example: Decrypted FILE_CHUNK

```json
{
  "type": "file-chunk",
  "transferId": "<hex, 32 chars>",
  "chunkIndex": 0,
  "totalChunks": 42,
  "payload": "<base64, plaintext chunk>"
}
```

### Example: Decrypted FILE_FINISH

```json
{
  "type": "file-finish",
  "transferId": "<hex, 32 chars>",
  "fileHash": "<hex, 64 chars>"
}
```

### Control Messages (decrypted)

```json
{ "type": "pause", "transferId": "..." }
{ "type": "resume", "transferId": "..." }
{ "type": "cancel", "transferId": "...", "cancelledBy": "sender" }
```

### HELLO Exchange

- The first application message sent over the DataChannel MUST be an encrypted envelope containing HELLO
- Handshake completes only after both peers successfully decrypt HELLO

### Plaintext Messages

Only the following are sent without encryption:

```json
{ "type": "ping" }
{ "type": "pong" }
```

These MUST NOT contain sensitive data.

---

## 7. Local Scope Policy

LocalBolt prefers local network connections. This is a HEURISTIC, not enforcement.

### Mechanisms

- IP-based room grouping: signaling server groups private IPs into shared "local" room
- Relay candidate blocking: no TURN servers configured, relay ICE candidates dropped
- Private IP recognition: RFC 1918, link-local, CGNAT/Tailscale (100.64/10), IPv6 ULA/link-local

### Known Limitations

- VPN clients may appear local when they are remote
- CGNAT devices may appear local when they are not on same mesh
- Multi-homed devices may be in different rooms on different interfaces
- This is product-level policy, not protocol-level security

---

## 8. Resource Limits

| Limit | Value |
|-------|-------|
| Maximum file size per transfer | 10 GB (without explicit user approval) |
| Maximum `total_chunks` per transfer | Derived: `max_file_size / chunk_size` (655360 at 16KB; recompute if chunk size changes) |
| Maximum concurrent transfers per session | 1 |

---

## 9. Reconnection

- Exponential backoff: `delay = min(1000 * 2^attempt, 30000)` ms
- Keepalive: ping every 30s

---

## 10. Key Exchange Binding

The authoritative ephemeral key used for Bolt encryption is the `senderEphemeralKey` field present in each envelope.

Transport-level key carriage (e.g. SDP exchange) MAY be used as an optimization but MUST NOT be required for correctness.

Implementations MAY compare transport-carried keys with observed envelope keys and warn on mismatch.

---

## 11. Platform-Specific Chunk Sizes

| Platform | Chunk Size |
|----------|-----------|
| Default | 16384 (16KB) |
| Mobile | 8192 (8KB) |
| Desktop/Laptop | 16384 (16KB) |
| Steam Deck | 32768 (32KB) |
