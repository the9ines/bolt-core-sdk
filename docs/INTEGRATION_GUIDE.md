# Bolt Ecosystem Integration Guide

External adopters: how to build a Bolt-conformant application.

**Audience:** Developers building apps outside the core Bolt ecosystem who need to
interoperate with LocalBolt, bolt-daemon, or other Bolt-protocol peers.

**Normative sources:**
- Protocol semantics: `bolt-protocol/PROTOCOL.md` (wire formats, state machines, conformance)
- Transport requirements: `docs/TRANSPORT_CONTRACT.md`
- Authority model: `docs/SDK_AUTHORITY.md`

---

## 1. What Bolt Canonical Authority Is

### 1.1 Peer Connection Lifecycle

PROTOCOL.md §9 defines the canonical peer connection state machine:

```
IDLE → REQUEST_SENT → APPROVED → TRANSPORT_CONNECTING → HANDSHAKING → CONNECTED → DISCONNECTED
```

Transitions:
- Enter `TRANSPORT_CONNECTING` when approval is granted
- Enter `HANDSHAKING` when the peer channel opens
- Exit `HANDSHAKING` only when mutual HELLO and TOFU verification succeed
- Enter `CONNECTED` after successful handshake completion

The Rust implementation of the handshake lifecycle lives in
`rust/bolt-core/src/session.rs` (`SessionState`: PreHello → PostHello → Closed).
This is the committed executable authority for session state management, including
`HelloState` (exactly-once HELLO guard) and `SessionContext` (post-handshake state
container with capability negotiation).

### 1.2 Transfer Lifecycle

PROTOCOL.md §9 defines the canonical transfer state machine:

```
IDLE → OFFERED → ACCEPTED → TRANSFERRING ↔ PAUSED → COMPLETED
                                  |                       |
                                ERROR ←──────────── CANCELLED
```

Eight states. Three cancel reasons: `BySender`, `ByReceiver`, `Rejected`.

The Rust implementation lives in `rust/bolt-transfer-core/src/state.rs`
(`TransferState` enum — 8 variants matching §9 exactly). The `SendSession` and
`ReceiveSession` types in `bolt-transfer-core` enforce legal transitions at
compile time.

### 1.3 Verification and SAS

PROTOCOL.md §3 defines SAS (Short Authentication String) verification:

```
SAS_input = SHA-256( sort32(identity_A, identity_B) || sort32(ephemeral_A, ephemeral_B) )
SAS = uppercase(hex(SAS_input[0..3]))
```

Properties:
- 6 characters, uppercase hex (`0-9A-F`), 24 bits entropy
- Symmetric: both peers compute the same SAS regardless of role
- Binds identity keys AND ephemeral keys

PROTOCOL.md §13 conformance requirements:
- MUST support persistent device identity (TOFU)
- MUST send `ERROR(KEY_MISMATCH)` and close session on key mismatch
- MUST complete handshake before sending transfer messages

The handshake completion gate (HELLO exchange + TOFU verification) is the
mechanism that prevents transfers before peer authentication. There is no
separate "transfer gating" API — transfers are only possible after `CONNECTED`
state, which requires completed handshake.

### 1.4 Parity Model

**Canonical truth = Rust crates + golden vectors + PROTOCOL.md.**

The Rust crates (`rust/bolt-core/`, `rust/bolt-btr/`, `rust/bolt-transfer-core/`)
are the reference implementation. Any other implementation is an adapter and
MUST produce identical wire-format outputs for identical inputs, verified by
the golden test vectors in `rust/bolt-core/test-vectors/`.

Vector suites (all committed):

| Suite | Location | Purpose |
|-------|----------|---------|
| Box payload | `rust/bolt-core/test-vectors/core/box-payload.vectors.json` | seal/open correctness |
| Framing | `rust/bolt-core/test-vectors/core/framing.vectors.json` | nonce‖ciphertext layout |
| SAS | `rust/bolt-core/test-vectors/core/sas.vectors.json` | SAS computation |
| HELLO-open | `rust/bolt-core/test-vectors/core/web-hello-open.vectors.json` | HELLO envelope |
| Envelope-open | `rust/bolt-core/test-vectors/core/envelope-open.vectors.json` | Generic envelope |
| BTR (10 files) | `rust/bolt-core/test-vectors/btr/*.vectors.json` | BTR ratchet + key derivation |

The Rust conformance harness (`rust/bolt-core/tests/conformance/`) verifies
state machine authority (AC-RC-10), including transfer SM, BTR negotiation,
replay guard, and backpressure.

---

## 2. What Adopters Must Implement

### 2.1 Peer Connection State Machine

Your application MUST model the PROTOCOL.md §9 peer connection states. You do
not need to use the canonical names internally, but your implementation must
represent the same semantic progression and enforce legal transitions:

| Canonical (§9) | Meaning |
|----------------|---------|
| `IDLE` | No session activity |
| `REQUEST_SENT` | Outbound connection request, awaiting response |
| `APPROVED` | Request approved, transport setup starting |
| `TRANSPORT_CONNECTING` | Transport channel being established |
| `HANDSHAKING` | HELLO exchange and TOFU verification in progress |
| `CONNECTED` | Session active — transfers permitted |
| `DISCONNECTED` | Session ended |

The Rust `SessionState` type in `rust/bolt-core/src/session.rs` models the
handshake-level lifecycle (PreHello → PostHello → Closed). Adopters building
on the Rust crates consume this directly.

### 2.2 Transfer State Machine

Model the eight transfer states from PROTOCOL.md §9. The Rust `TransferState`
enum in `rust/bolt-transfer-core/src/state.rs` is the canonical implementation.
`SendSession` and `ReceiveSession` enforce valid transitions.

Conformance requirements from §13:
- MUST use `transfer_id` to identify transfers
- MUST reject duplicate `chunk_index` (scoped per `(transfer_id, chunk_index)`)
- MUST reject `chunk_index >= total_chunks`
- MUST complete handshake before sending transfer messages
- MUST verify `file_hash` after reassembly when `bolt.file-hash` negotiated

### 2.3 Handshake Requirements

PROTOCOL.md §15 defines handshake invariants. Key requirements for adopters:

- **Exactly one HELLO per connection** — enforced by `HelloState` in `session.rs`
- **Ephemeral-first keying** — fresh ephemeral keypair per connection; MUST NOT rotate mid-session
- **Identity binding** — SAS binds identity AND ephemeral keys
- **Handshake gating** — reject protected messages before handshake completion with `ERROR(INVALID_STATE)`
- **Post-handshake envelope** — all non-PING/PONG messages MUST be sent inside encrypted envelopes after HELLO

### 2.4 Transport Requirements

Your transport MUST provide:

- **Ordered delivery** — messages arrive in the order sent
- **Reliable delivery** — no silent drops; failures are reported
- **Message framing** — one sealed payload = one transport message (no fragmentation, no coalescing)

See `docs/TRANSPORT_CONTRACT.md §1` for the full requirement set.

Each sealed payload is a NaCl box output:

```
sealed_payload = base64( nonce(24 bytes) || ciphertext(plaintext_len + 16 bytes) )
```

Base64 encoding is **RFC 4648 standard** (not URL-safe). `+` and `/` are used,
not `-` and `_`. Padding with `=` is required.

Use the transport path that matches your deployment (see §4).

### 2.5 Crypto Primitives

Adopters MUST NOT reimplement the envelope, handshake, or SAS logic.
Use the canonical crates:

**Rust consumers:**
```toml
[dependencies]
bolt-core          = { path = "../bolt-core-sdk/rust/bolt-core" }          # crypto, identity, SAS
bolt-btr           = { path = "../bolt-core-sdk/rust/bolt-btr" }            # BTR ratchet (if needed)
bolt-transfer-core = { path = "../bolt-core-sdk/rust/bolt-transfer-core" }  # transfer SM
```

**Browser consumers:**
The `@the9ines/bolt-core` npm package provides equivalent primitives for
browser environments. It is an adapter that passes the same golden vectors
as the Rust implementation.

See `docs/BOUNDARY_CONTRACT.md` for the full consumer boundary specification
(Rust-direct, WASM, and Tauri IPC patterns).

### 2.6 Signaling

Signaling is handled by `bolt-rendezvous`. It is **untrusted by design** —
it relays opaque signaling payloads and provides presence notifications.
It does not see file contents or encryption keys.

Adopters may:
- Use a hosted `bolt-rendezvous` endpoint
- Run their own `bolt-rendezvous` instance
- Use a different signaling mechanism that delivers equivalent information
  (connection request/accept/decline, peer identification)

The signaling channel is not part of the Bolt security model. All sensitive
data is protected by the protocol's encryption layer regardless of signaling
channel security.

---

## 3. What Adopters Do NOT Need to Implement

| What | Why not required |
|------|-----------------|
| Product-specific UI (localbolt UX, bolt-ui shell) | Contract is behavioral; UI is product-specific |
| Same runtime architecture (Tauri, bolt-daemon sidecar, egui, SwiftUI) | Architecture choices are product-specific |
| bolt-daemon itself | Required only for native apps needing the daemon-centered transport path |
| WebRTC transport | Required only for browser↔browser peers |
| Native QUIC/quinn | Required only for native↔native path |
| WebTransport | Required only for HTTPS web↔native path |
| Specific discovery mechanism (mDNS, cloud signaling) | Signaling is pluggable |
| BTR ratchet | Only required for sessions negotiating `bolt.transfer-ratchet-v1` |

You MUST NOT reimplement any crypto primitive (NaCl box, SAS computation, BTR
key derivation) from scratch. Use the canonical crates or `@the9ines/bolt-core`.

---

## 4. Transport Matrix

### Supported Paths

| Endpoint Pair | Transport | Status |
|--------------|-----------|--------|
| native↔native (app↔app) | QUIC via quinn (Rust) | Supported |
| browser↔browser | WebRTC DataChannel | Supported (G1 invariant — immutable) |
| HTTPS web↔native | WebTransport (HTTP/3) | Supported (production) |
| HTTP/localhost web↔native | WebSocket-direct | Supported (dev/LAN only) |

**G1 invariant:** browser↔browser always uses WebRTC. This is not negotiable.

**HTTPS web↔native (WebTransport)** is the production path for browser-to-native
communication over HTTPS origins. Requirements:
- Daemon serves a WebTransport/HTTP3 endpoint with TLS
- Browser has WebTransport API support (Chrome, Edge, Firefox — not Safari)
- Capability negotiated: `bolt.transport-webtransport-v1`

**HTTP/localhost web↔native (WS-direct)** is for development and LAN testing
only. No TLS is required. This path applies when the browser origin is HTTP
(not HTTPS).

### Explicitly Unsupported

| Path | Why |
|------|-----|
| Native WebRTC (app↔app or app↔browser via WebRTC) | Native path uses QUIC; web↔native uses WebTransport. WebRTC is browser↔browser only. |
| HTTPS web → plain ws:// native | Mixed content — browsers block ws:// from HTTPS origins. Impossible. |
| Raw QUIC from browsers | Browsers do not expose raw QUIC sockets; WebTransport is the browser QUIC surface |
| UDP datagrams without reliability layer | Violates transport contract §1 (ordered + reliable required) |
| webrtc-rs | Candidate only; must pass graduation criteria (`docs/ECOSYSTEM_STRATEGY.md §7`) |

---

## 5. Conformance Expectations

### 5.1 Protocol Conformance (PROTOCOL.md §13)

All adopters MUST satisfy the conformance requirements in PROTOCOL.md §13.
Key requirements summarized (refer to §13 for the full list):

- Persistent device identity (TOFU)
- Fail-closed on key mismatch (`ERROR(KEY_MISMATCH)`)
- Fresh ephemeral keys per connection
- Envelope encryption for all protected messages
- Handshake completion before transfer messages
- `transfer_id` scoped replay protection
- File hash verification when `bolt.file-hash` negotiated

### 5.2 Golden Vector Compliance

Any implementation that performs crypto operations MUST pass the vector suites:

1. Open all valid entries in `test-vectors/core/box-payload.vectors.json`
2. Reject all corrupt entries in the same file
3. Pass all framing assertions in `test-vectors/core/framing.vectors.json`
4. Produce correct SAS output for `test-vectors/core/sas.vectors.json` inputs

If your implementation delegates crypto to the canonical Rust crates or
`@the9ines/bolt-core`, vector compliance is handled by the library. Verify your
integration does not introduce mismatches (wrong key selection, wrong base64
variant, etc.).

### 5.3 State Machine Authority

The Rust types are the executable authority for state machines:

| Domain | Rust type | Location |
|--------|-----------|----------|
| Session lifecycle | `SessionState`, `HelloState`, `SessionContext` | `rust/bolt-core/src/session.rs` |
| Transfer lifecycle | `TransferState`, `SendSession`, `ReceiveSession` | `rust/bolt-transfer-core/src/` |
| BTR negotiation | `BtrMode`, `negotiate_btr` | `rust/bolt-btr/src/negotiate.rs` |
| BTR state | `BtrEngine`, `BtrTransferContext` | `rust/bolt-btr/src/state.rs` |
| Replay guard | `ReplayGuard` | `rust/bolt-btr/src/replay.rs` |
| Backpressure | `BackpressureController` | `rust/bolt-transfer-core/src/backpressure.rs` |

The conformance test suite (`rust/bolt-core/tests/conformance/state_machine_authority.rs`)
proves Rust is the canonical source for all protocol state machines.

Adopters not using Rust MUST demonstrate behavioral equivalence with the Rust
types for their implemented state machines.

### 5.4 Document Your Conformance

Create a conformance mapping document (model:
`docs/conformance/LOCALBOLT_CONFORMANCE.md`) that traces each required behavior
from PROTOCOL.md §13 to a test case in your implementation. This is the evidence
required for NONCORE-ADOPTER-1 acceptance.

---

## 6. Minimal Adopter Path

### Step 1: Add the SDK dependency

**Rust:**
```toml
[dependencies]
bolt-core          = { path = "../bolt-core-sdk/rust/bolt-core" }
bolt-transfer-core = { path = "../bolt-core-sdk/rust/bolt-transfer-core" }
```

**Browser (npm):**
```
npm install @the9ines/bolt-core
```

### Step 2: Implement the session lifecycle

Use `SessionState`, `HelloState`, and `SessionContext` from `bolt-core::session`.
Enforce exactly-once HELLO, fail-closed on duplicate, and post-handshake envelope
requirement.

If you are not using Rust directly, implement equivalent state tracking that
enforces the same invariants.

### Step 3: Implement the transfer lifecycle

Use `SendSession` and `ReceiveSession` from `bolt-transfer-core`. These enforce
valid `TransferState` transitions. Ensure:
- `transfer_id` is used for all transfers
- Duplicate `chunk_index` is rejected
- `chunk_index >= total_chunks` is rejected
- Session end resets transfer state

### Step 4: Choose a supported transport

Pick the transport matching your deployment from the matrix in §4.
No other transport paths are supported.

### Step 5: Verify golden vectors

Run the vector suites against your crypto layer:
```bash
cd rust/bolt-core && cargo test
```

If you have a non-Rust crypto implementation, validate against the vector JSON
files in `rust/bolt-core/test-vectors/core/` directly.

### Step 6: Document conformance

Create a conformance mapping document tracing PROTOCOL.md §13 requirements
to test cases. See `docs/conformance/LOCALBOLT_CONFORMANCE.md` for the
expected format and evidence structure.

---

## 7. Reference Documents

All documents listed below are committed and version-controlled.

| Document | Location | Purpose |
|----------|----------|---------|
| Protocol spec | `bolt-protocol/PROTOCOL.md` | Normative wire-level specification (§9 state machines, §13 conformance) |
| LocalBolt profile | `bolt-protocol/LOCALBOLT_PROFILE.md` | Profile-level requirements |
| Transport contract | `docs/TRANSPORT_CONTRACT.md` | Transport requirements (ordered, reliable, message-framed) |
| Authority model | `docs/SDK_AUTHORITY.md` | Canonical source of truth hierarchy |
| API surface (Rust) | `docs/API_SURFACE.md` | Unified Rust crate registry |
| Boundary contract | `docs/BOUNDARY_CONTRACT.md` | Consumer boundary types (Rust-direct, WASM, Tauri IPC) |
| Conformance example | `docs/conformance/LOCALBOLT_CONFORMANCE.md` | Conformance mapping pattern (5 behaviors, 31 tests) |
| Interop test plan | `docs/INTEROP_TEST_PLAN.md` | Vector + live interop procedures |
| Ecosystem strategy | `docs/ECOSYSTEM_STRATEGY.md` | Product boundaries, release discipline, headless transport |
| Protocol contract (SDK) | `docs/PROTOCOL_CONTRACT.md` | SDK conformance clarifications |
| Session types (Rust) | `rust/bolt-core/src/session.rs` | SessionState, HelloState, SessionContext |
| Transfer types (Rust) | `rust/bolt-transfer-core/src/state.rs` | TransferState, CancelReason |
| Golden vectors | `rust/bolt-core/test-vectors/` | 16 vector files (core + BTR) |
| Conformance tests | `rust/bolt-core/tests/conformance/` | State machine authority proof (AC-RC-10) |
