# Bolt Ecosystem Integration Guide

External adopters: how to build a Bolt-conformant application.

**Audience:** Developers building apps outside the core Bolt ecosystem who need to
interoperate with LocalBolt, bolt-daemon, or other Bolt-protocol peers.

**Normative sources:**
- Protocol semantics: `bolt-protocol/PROTOCOL.md` + `LOCALBOLT_PROFILE.md`
- Session/transfer contract: `docs/SESSION_CONTRACT.md` + `rust/bolt-app-core/contracts/session_contract.v1.json`
- Transport requirements: `docs/TRANSPORT_CONTRACT.md`
- Authority model: `docs/SDK_AUTHORITY.md`

---

## 1. What Bolt Canonical Authority Is

### 1.1 Session Contract

The session contract defines five phases and nine legal transitions:

```
idle ──────────────► requesting         (user selects peer)
idle ──────────────► incoming_request   (signal received from remote)
requesting ────────► connecting         (remote accepted)
requesting ────────► idle               (remote declined or timeout)
incoming_request ──► connecting         (local user accepted)
incoming_request ──► idle               (local user declined)
connecting ────────► connected          (handshake complete)
connecting ────────► idle               (handshake failed)
connected ─────────► idle               (disconnect, error, or remote close)
```

All other session phase transitions are **illegal**. Adopters MUST reject them.

### 1.2 Transfer Contract

Five transfer phases and twelve legal transitions:

```
idle ──────► sending    (user initiates send)
idle ──────► receiving  (remote initiates send)
sending ───► complete   (all chunks acknowledged)
sending ───► failed     (error or session lost)
receiving ─► complete   (all chunks received and saved)
receiving ─► failed     (error or session lost)
complete ──► idle       (user dismisses)
failed ────► idle       (user dismisses)
sending ───► idle       (session disconnect — mandatory cleanup)
receiving ─► idle       (session disconnect — mandatory cleanup)
complete ──► idle       (session disconnect — mandatory cleanup)
failed ────► idle       (session disconnect — mandatory cleanup)
```

The last four transitions (session disconnect → idle) are mandatory cleanup paths —
they apply regardless of the current transfer state when the session ends.

### 1.3 Verification and Trust States

| State | Meaning | Transfer allowed? |
|-------|---------|-------------------|
| `unverified` | SAS received, pending user confirmation | NO |
| `verified` | User confirmed SAS match | YES |
| `legacy` | Peer lacks identity support (no SAS) | YES |

Transfer is only permitted when `session_phase == connected` AND
`verification_state` is `verified` or `legacy`.

The SAS (Short Authentication String) is a 6-character uppercase hex string computed
over both peers' identity and ephemeral public keys:

```
SAS_input = SHA-256( sort32(identity_A, identity_B) || sort32(ephemeral_A, ephemeral_B) )
SAS = uppercase(hex(SAS_input[0..3]))
```

Both peers compute identical SAS regardless of role. See `PROTOCOL.md §3`.

### 1.4 Parity Model

**Canonical truth = Rust crates + golden vectors + contract documents.**

The Rust crates (`rust/bolt-core/`, `rust/bolt-btr/`, `rust/bolt-transfer-core/`)
are the reference implementation. Any other implementation (TypeScript, Swift, Kotlin)
is an adapter and MUST produce identical wire-format outputs for identical inputs,
verified by the golden test vectors in `rust/bolt-core/test-vectors/`.

Vector suites:

| Suite | Location | Purpose |
|-------|----------|---------|
| Box payload | `test-vectors/core/box-payload.vectors.json` | seal/open correctness |
| Framing | `test-vectors/core/framing.vectors.json` | nonce||ciphertext layout |
| SAS | `test-vectors/core/sas.vectors.json` | SAS computation |
| HELLO-open | `test-vectors/core/hello-open.vectors.json` | HELLO envelope |
| Envelope-open | `test-vectors/core/envelope-open.vectors.json` | Generic envelope |
| BTR | `test-vectors/btr/*.vectors.json` | BTR ratchet + key derivation |

The **machine-readable contract** for session/transfer states lives in:
- `rust/bolt-app-core/contracts/session_contract.schema.json` (JSON Schema)
- `rust/bolt-app-core/contracts/session_contract.v1.json` (current instance)
- `rust/bolt-app-core/src/contracts/session_contract.rs` (Rust validators — executable authority)

The Rust validators provide the enforcement functions adopters can use directly
or validate against:
- `is_valid_session_transition(from, to)` — 9 legal pairs
- `is_valid_transfer_transition(from, to)` — 10 legal pairs
- `is_transfer_allowed(session_phase, verification_state)` — policy P1

---

## 2. What Adopters Must Implement

### 2.1 Map to Canonical States

Your connector or runtime MUST model all five canonical session phases. You do not
need to use the canonical names internally, but your implementation must represent
the same semantic states:

| Canonical | Meaning | Example internal name |
|-----------|---------|----------------------|
| `idle` | No session | `disconnected`, `ready` |
| `requesting` | Outbound request pending | `connecting_outbound` |
| `incoming_request` | Inbound request pending | `pairing_pending` |
| `connecting` | Handshake in progress | `handshaking` |
| `connected` | Session active | `active`, `paired` |

Similarly for transfer phases (`idle`, `sending`, `receiving`, `complete`, `failed`)
and verification states (`unverified`, `verified`, `legacy`).

Mapping local state names to canonical state IDs is acceptable. The constraint is
behavioral: your state machine must accept exactly the legal transitions and reject
all others.

### 2.2 Enforce Legal Transitions and Invariants

**Required invariants (MUST enforce):**

| ID | Rule |
|----|------|
| INV-1 | Session disconnect MUST reset transfer phase to `idle` |
| INV-2 | Transfer MUST be gated by policy P1 (connected + verified|legacy) |
| INV-3 | Only one active session per product instance at any time |

**Recommended invariants (SHOULD enforce):**

| ID | Rule |
|----|------|
| INV-4 | Increment a generation counter on every session reset |
| INV-5 | Reject stale callbacks whose generation does not match current |

The canonical reset path (P3): the only exit from `connected` to `idle` MUST
also reset transfer phase to `idle`.

### 2.3 Transfer Gating Policy

Gate all transfer initiation (both send and receive) on this check:

```
transfer_allowed = (session_phase == connected) AND
                   (verification_state == verified OR verification_state == legacy)
```

Your implementation MUST NOT initiate or accept a file transfer unless this
condition holds. This is policy P1 and invariant INV-2.

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
bolt-transfer-core = { path = "../bolt-core-sdk/rust/bolt-transfer-core" }  # transfer SM (optional)
```

**Browser consumers:**
The `@the9ines/bolt-core` npm package (published to npm) provides equivalent
primitives for browser environments. It is an adapter that passes the same
golden vectors as the Rust implementation.

```
import { sealBoxPayload, openBoxPayload, computeSas, generateEphemeralKeyPair } from '@the9ines/bolt-core';
```

See `docs/BOUNDARY_CONTRACT.md` for the full consumer boundary specification
(Rust-direct, WASM, and Tauri IPC patterns).

### 2.6 Signaling Registration

Signaling is handled by `bolt-rendezvous`. It is **untrusted by design** —
it relays opaque signaling payloads (SDP offers/answers, ICE candidates) and
provides presence notifications. It does not see file contents or encryption keys.

Adopters may:
- Use a hosted `bolt-rendezvous` endpoint
- Run their own `bolt-rendezvous` instance (self-hosted)
- Use a different signaling mechanism that delivers the same information (SDP,
  ICE candidates, peer identification)

The signaling channel is not part of the Bolt security model. All sensitive
data is protected by the protocol's encryption layer regardless of signaling
channel security.

---

## 3. What Adopters Do NOT Need to Implement

| What | Why not required |
|------|-----------------|
| Product-specific UI (localbolt UX, bolt-ui shell) | Contract is behavioral; UI is product-specific |
| Same runtime architecture (Tauri, bolt-daemon sidecar, egui) | Architecture choices are product-specific |
| bolt-daemon itself | Required only for native apps needing the daemon-centered transport path |
| WebRTC fallback transport | Required only if you want to interoperate with browser↔browser peers |
| WebTransport + WS + WebRTC three-tier fallback | Required only for the browser↔native path; simpler deployments may use a subset |
| Specific discovery mechanism (mDNS, cloud signaling) | Signaling is pluggable; any mechanism that delivers SDP/ICE satisfies the requirement |
| QUIC/quinn | Required only for native↔native (app↔app) path |
| BTR ratchet | Only required for sessions that negotiate the `bolt.btr-v1` capability |

You MUST NOT reimplement any crypto primitive (NaCl box, SAS computation, BTR
key derivation) from scratch. Use the canonical crates.

---

## 4. Transport Matrix

The supported endpoint-pair → transport mapping. Use the cell matching your
deployment.

| Endpoint Pair | Primary Transport | Fallback 1 | Fallback 2 | Status |
|--------------|-------------------|------------|------------|--------|
| native↔native (app↔app) | QUIC via quinn (Rust) | DataChannel (libdatachannel) | — | Production |
| browser↔browser | WebRTC DataChannel | — | — | Production (G1 invariant) |
| HTTPS web↔native | WebTransport (HTTP/3) | WebSocket-direct | WebRTC | Production |
| HTTP/localhost web↔native | WebSocket-direct | WebRTC | — | Dev/LAN |
| browser↔native (Safari) | WebSocket-direct | WebRTC | — | Safari lacks WebTransport |

**G1 invariant:** browser↔browser always uses WebRTC. This is immutable.

**WebTransport note:** WebTransport (HTTPS web↔native) requires:
- Daemon serving TLS (local CA cert acceptable for dev; CA-signed for production)
- Chrome or other browsers with WebTransport support (not Safari)
- Capability string: `bolt.transport-webtransport-v1`

**WebSocket fallback:** WS direct is the first fallback for browser↔native when
WebTransport is unavailable (Safari, HTTP origin, or daemon not reachable via WT).
WS direct is also the primary path for HTTP/localhost origins (development setups).

### Dev-Only Paths

| Path | Condition | Notes |
|------|-----------|-------|
| WS direct (HTTP origin) | Development/LAN testing only | No TLS; sufficient for localhost dev |

### Explicitly Unsupported

| Path | Why unsupported |
|------|----------------|
| Raw QUIC (quinn) from browsers | Browsers do not expose raw QUIC; use WebTransport instead |
| UDP datagrams without reliability | Violates transport contract §1 (must be ordered + reliable) |
| webrtc-rs as production transport | Candidate only; must pass graduation criteria (see `docs/ECOSYSTEM_STRATEGY.md §7`) |
| libdatachannel as production transport | Planned; not yet validated to graduation criteria |
| WebTransport in Safari | Safari does not support WebTransport; falls to WS/WebRTC |

---

## 5. Conformance Expectations

A Bolt-conformant adopter must satisfy all of the following:

### 5.1 Use Canonical Contracts

- Implement all five session phases and map to canonical IDs as described in §2.1.
- Enforce the nine legal session transitions. Reject all others.
- Enforce the twelve legal transfer transitions.
- Enforce transfer gating policy P1.
- Satisfy required invariants INV-1, INV-2, INV-3.

### 5.2 Pass Golden Vectors (or Equivalent Conformance Tests)

Any implementation that performs crypto operations MUST pass the vector suites:
- Open all valid entries in `test-vectors/core/box-payload.vectors.json`.
- Reject all corrupt entries in the same file.
- Pass all framing assertions in `test-vectors/core/framing.vectors.json`.
- Produce correct SAS output for `test-vectors/core/sas.vectors.json` inputs.

If your implementation delegates crypto to the canonical Rust crates or
`@the9ines/bolt-core`, the library handles vector compliance. You still need
to verify your integration does not introduce mismatches (wrong key selection,
wrong base64 variant, etc.).

### 5.3 Pass Contract Parity Checks

Your state machine MUST be checkable against the Rust validators:
- `is_valid_session_transition(from, to)`
- `is_valid_transfer_transition(from, to)`
- `is_transfer_allowed(session_phase, verification_state)`

These are the M4-PARITY-1 gate validators used by the core products. Document
your mapping (like `docs/conformance/LOCALBOLT_CONFORMANCE.md`) to allow
third-party review.

### 5.4 Required Invariants

All three required invariants (INV-1/2/3) MUST be enforced at runtime, not
just at design time. Tests must exercise session-disconnect → transfer-reset
and transfer-gating paths explicitly.

---

## 6. Minimal Adopter Path

The smallest viable path to become Bolt-conformant, in order:

### Step 1: Add the SDK dependency

**Rust:**
```toml
[dependencies]
bolt-core = { path = "../bolt-core-sdk/rust/bolt-core" }
```

**Browser (npm):**
```
npm install @the9ines/bolt-core
```

### Step 2: Implement the session state machine

Model the five session phases and enforce the nine legal transitions. Use the
Rust validators directly if you are a Rust consumer:

```rust
use bolt_app_core::contracts::session_contract::{
    is_valid_session_transition, is_valid_transfer_transition, is_transfer_allowed,
};

// Before any transition:
if !is_valid_session_transition(&current_phase, &next_phase) {
    return Err(/* illegal transition error */);
}
```

Or validate against the JSON contract if you are implementing in another language:
load `session_contract.v1.json` and check your transitions against the
`session_transitions` array.

### Step 3: Implement the transfer state machine

Model the five transfer phases and enforce the twelve legal transitions. Ensure
session disconnect resets transfer phase to `idle` (INV-1).

### Step 4: Enforce transfer gating

Before initiating or accepting any transfer:

```rust
if !is_transfer_allowed(&session_phase, &verification_state) {
    return Err(/* transfer not allowed */);
}
```

### Step 5: Choose a compliant transport

Pick the transport for your deployment (see §4). Verify it satisfies
`TRANSPORT_CONTRACT.md §1`: ordered, reliable, message-framed.

### Step 6: Verify golden vectors

Run the vector suites against your crypto layer. If you use the canonical
crates, run their tests (`cargo test` in `rust/bolt-core/`). If you have your
own crypto layer, validate against the vector JSON files directly.

### Step 7: Document your conformance mapping

Create a conformance mapping document (model: `docs/conformance/LOCALBOLT_CONFORMANCE.md`)
that traces each required behavior to a test case. This is the evidence required
for NONCORE-ADOPTER-1 acceptance.

---

## 7. Reference Documents

| Document | Location | Purpose |
|----------|----------|---------|
| Session/transfer contract | `docs/SESSION_CONTRACT.md` | Canonical state machine spec |
| Machine-readable contract | `rust/bolt-app-core/contracts/` | JSON Schema + instance |
| Rust validators | `rust/bolt-app-core/src/contracts/session_contract.rs` | Executable authority |
| Transport contract | `docs/TRANSPORT_CONTRACT.md` | Transport requirements |
| Authority model | `docs/SDK_AUTHORITY.md` | Canonical source of truth hierarchy |
| Boundary contract | `docs/BOUNDARY_CONTRACT.md` | How to consume SDK across boundary types |
| Conformance example | `docs/conformance/LOCALBOLT_CONFORMANCE.md` | Conformance mapping pattern |
| Interop test plan | `docs/INTEROP_TEST_PLAN.md` | Vector + live interop procedures |
| API surface (Rust) | `docs/API_SURFACE.md` | Unified Rust crate registry |
| Protocol spec | `bolt-protocol/PROTOCOL.md` | Normative wire-level specification |
| LocalBolt profile | `bolt-protocol/LOCALBOLT_PROFILE.md` | Profile-level requirements |
