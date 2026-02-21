# Bolt Transport Contract

Defines the abstraction boundary between the Bolt protocol layer and transport implementations.

## 1. Transport Requirements

A compliant Bolt transport MUST provide:

### Ordered, reliable byte delivery

- Messages arrive in the order they were sent.
- No message is silently dropped. If delivery fails, the transport reports an error.
- This matches TCP semantics. Bolt does NOT handle reordering or retransmission at the protocol layer.

### Connection lifecycle events

The transport MUST expose at minimum:

| Event | Meaning |
|-------|---------|
| `connected` | Transport channel is open and ready for data. |
| `message` | A complete message (byte sequence) has been received. |
| `error` | A transport-level error has occurred. |
| `closed` | The transport channel has been closed (graceful or abrupt). |

### Backpressure

- The transport SHOULD provide backpressure signaling when the send buffer is full.
- Implementations MAY use bufferedAmount monitoring (WebRTC) or equivalent mechanisms.
- The protocol layer does NOT enforce a specific backpressure strategy. Products implement this based on transport capabilities.

### Message framing

- The transport delivers discrete messages, not a byte stream.
- Each sealed payload (output of `sealBoxPayload`) is one transport message.
- The transport MUST NOT fragment a single sealed payload across multiple transport messages.
- The transport MUST NOT coalesce multiple sealed payloads into a single transport message.

### Maximum message size

- The protocol does not define a maximum message size.
- Transport implementations impose practical limits (e.g., SCTP max message size for WebRTC).
- `DEFAULT_CHUNK_SIZE` (16 KB) is chosen to stay well within typical transport limits.
- Products MUST NOT send sealed payloads larger than their transport supports.

## 2. Compliant Implementations

### Browser WebRTC DataChannel

- **Status**: Production. Used by localbolt, localbolt-app (web layer), localbolt-v3.
- **Ordered delivery**: `ordered: true` on DataChannel creation.
- **Reliability**: `reliable` mode (no maxRetransmits, no maxPacketLifeTime).
- **Message framing**: SCTP provides message boundaries natively.
- **Max message size**: Typically 256 KB (browser-dependent). 16 KB chunks are well within this.
- **Backpressure**: Monitor `dataChannel.bufferedAmount` before sending.

### libdatachannel (C/C++ library)

- **Status**: Planned. Target for localbolt-app (native) and bytebolt-app.
- **Ordered delivery**: Configured via DataChannel options.
- **Reliability**: Configured to reliable mode.
- **Message framing**: SCTP message boundaries.
- **Max message size**: Configurable. Must be >= `DEFAULT_CHUNK_SIZE` + box overhead (16 bytes) + nonce (24 bytes).
- **Interop**: Uses the same DTLS/SCTP stack as browser WebRTC. Interoperable with browser peers via standard SDP/ICE signaling.

### webrtc-rs (Rust library)

- **Status**: Candidate. Under evaluation for bolt-daemon and headless deployments.
- **Requirements**: Must satisfy the same ordered/reliable/message-framed contract.
- **Graduation criteria**: See `ECOSYSTEM_STRATEGY.md` § Headless Transport Lane.

## 3. Transport-Agnostic Protocol Messages

All Bolt protocol messages (handshake, payload, control) are transport-agnostic:

- They are defined as byte sequences (sealed payloads).
- They carry no transport-specific metadata.
- The same sealed payload can be sent over any compliant transport.

**Prohibited**: Embedding transport identifiers (e.g., DataChannel labels, SCTP stream IDs) inside sealed payloads.

## 4. LAN-Only Mode

LAN-only operation is enforced at the **signaling and ICE layer**, not by modifying the protocol.

### ICE policy for LAN-only

- **Private IP filtering**: Only gather ICE candidates with private/link-local IP addresses.
  - IPv4: `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`, `169.254.0.0/16`
  - IPv6: `fe80::/10`, `fd00::/8`
- **No TURN**: Do not configure TURN servers. No relay candidates.
- **No STUN to public servers**: Optionally omit public STUN servers to prevent IP leakage.
- **mDNS candidates**: Browser-generated mDNS candidates (e.g., `abc123.local`) are acceptable for LAN discovery.

### Implementation notes

- Browser WebRTC: Use `iceTransportPolicy: 'all'` with filtered ICE servers (no TURN, optionally no STUN). Filter candidates in the `onicecandidate` handler.
- libdatachannel: Configure ICE agent with private-only candidate policy.
- webrtc-rs: Same approach — filter at ICE gathering level.

### Important

LAN-only is a **deployment policy**, not a protocol feature. The Bolt protocol itself is network-agnostic. A product may choose to enforce LAN-only via signaling policy, or it may allow WAN connectivity via TURN. The sealed payloads are identical in both cases.

## 5. Signaling

Signaling (SDP offer/answer exchange, ICE candidate relay) is out of scope for the Bolt Core protocol. It is handled by:

- `bolt-rendezvous` (Rust signaling server, bundled or hosted)
- Product-specific signaling logic

The signaling channel is **untrusted**. All sensitive data (identity keys, file contents) is protected by the Bolt encryption layer, not by signaling channel security.

## 6. Connection Lifecycle

```
[Signaling] → ICE gathering → DTLS handshake → SCTP association → DataChannel open
     ↓
[Bolt Handshake] → HELLO exchange (encrypted) → SAS verification → Ready
     ↓
[Data Transfer] → Sealed chunks → ... → Transfer complete
     ↓
[Close] → DataChannel close → SCTP shutdown → ICE teardown
```

The Bolt handshake (HELLO exchange, SAS verification) occurs AFTER the transport channel is established. The transport is just a pipe — Bolt does not rely on transport-level authentication (DTLS fingerprints are not used for peer identity).
