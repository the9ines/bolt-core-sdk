# Bolt Session/Transfer State Contract v1

## Status

**Frozen (v1).** Primary ecosystem interoperability authority.
Frozen 2026-04-07. No breaking changes without a major version bump (v2).

Stable surface: 5 session phases, 9 session transitions, 5 transfer phases,
10 transfer transitions, 3 verification states, policies P1–P3, invariants
INV-1 through INV-5. Items listed under "Deferred to v2" are explicitly
not part of the frozen contract.

## Layer Relationship

This contract is the **primary authority for ecosystem adopters**.
It defines how products expose and manage session, transfer, and
verification state for cross-product interoperability.

**Relationship to PROTOCOL.md §9:**
`PROTOCOL.md` defines wire-level state machines (7 peer-connection states,
8 transfer states) for protocol conformance. This contract is a
product-level simplification: fewer states, direction-aware naming,
explicit verification gating. The two are complementary layers, not
competing authorities.

| Scope | Authority | Location |
|-------|-----------|----------|
| Ecosystem/adopter interop | **This contract** | `docs/SESSION_CONTRACT.md` + `rust/bolt-app-core/contracts/` |
| Wire-level protocol detail | `bolt-protocol/PROTOCOL.md` §9, §13 | bolt-protocol repo |
| Crypto primitives / vectors | `rust/bolt-core/` + `test-vectors/` | bolt-core-sdk |

Products MUST conform to this contract for ecosystem interoperability.
Products that implement wire-level protocol details directly MUST ALSO
satisfy `PROTOCOL.md` §13 conformance requirements.

### Product-to-Protocol State Mapping

| Contract phase | PROTOCOL.md §9 equivalent(s) |
|----------------|------------------------------|
| `idle` | IDLE, DISCONNECTED |
| `requesting` | REQUEST_SENT |
| `incoming_request` | CONNECTION_REQUEST received (§5) |
| `connecting` | APPROVED, TRANSPORT_CONNECTING, HANDSHAKING |
| `connected` | CONNECTED |

| Contract transfer phase | PROTOCOL.md §9 equivalent(s) |
|-------------------------|------------------------------|
| `idle` | Idle |
| `sending` | Offered, Accepted, Transferring, Paused (sender) |
| `receiving` | Offered, Accepted, Transferring (receiver) |
| `complete` | Completed |
| `failed` | Cancelled, Error |

## Purpose

Primary ecosystem interoperability contract for session lifecycle, transfer
lifecycle, and verification state across all Bolt Protocol products.

Both `localbolt-app` (native) and `localbolt-v3` (web) MUST conform.
Products own their state implementations; this contract defines the shapes,
transitions, and invariants they must satisfy.

## Scope

- Session phases and legal transitions
- Transfer phases and legal transitions
- Verification states and transfer gating policy
- Required and recommended invariants

## Session Phases

| Phase | Meaning |
|-------|---------|
| `idle` | No session activity. Initial state, or after reset. |
| `requesting` | Outbound connection request sent, awaiting peer response. |
| `incoming_request` | Inbound connection request received, awaiting local decision. |
| `connecting` | Request accepted, transport handshake in progress. |
| `connected` | Session active, transfer-ready (subject to verification gating). |

### Session Transitions

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

9 legal transitions. All other session phase transitions are ILLEGAL.

## Transfer Phases

| Phase | Meaning |
|-------|---------|
| `idle` | No transfer in progress. |
| `sending` | Outbound transfer in progress. |
| `receiving` | Inbound transfer in progress. |
| `complete` | Transfer finished successfully. |
| `failed` | Transfer failed. |

### Transfer Transitions

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

10 unique legal transition pairs (`complete→idle` and `failed→idle` each
subsume both user-dismiss and session-disconnect triggers).
All other transfer phase transitions are ILLEGAL.

## Verification States

| State | Meaning | Transfer allowed? |
|-------|---------|-------------------|
| `unverified` | SAS received, pending user confirmation. | NO |
| `verified` | User confirmed SAS match. | YES |
| `legacy` | Peer lacks identity support (no SAS). | YES |

## Policies

- **P1 — Transfer gating:** Transfer allowed only when `session_phase == connected`
  AND `verification_state` is `verified` or `legacy`.
- **P2 — Single session:** Only one active session per product instance.
- **P3 — Canonical reset:** The only path from `connected` to `idle` is through the
  product's `resetSession()` equivalent, which MUST also reset transfer phase to `idle`.

## Invariants

| ID | Description | Level |
|----|-------------|-------|
| INV-1 | Session disconnect MUST reset transfer phase to `idle`. | REQUIRED |
| INV-2 | Transfer MUST be gated by policy P1. | REQUIRED |
| INV-3 | Only one active session per product instance. | REQUIRED |
| INV-4 | Generation counter SHOULD increment on every session reset. | RECOMMENDED |
| INV-5 | Stale callbacks (generation mismatch) SHOULD be rejected. | RECOMMENDED |

## Machine-Readable Artifacts

- Schema: `rust/bolt-app-core/contracts/session_contract.schema.json`
- Example: `rust/bolt-app-core/contracts/session_contract.v1.json`
- Rust types: `rust/bolt-app-core/src/contracts/session_contract.rs`

## Authority Model (v1)

The contract has two enforcement layers with distinct responsibilities:

**JSON Schema** guarantees structural integrity:
- Exactly the 5 canonical session phases, each present exactly once.
- Exactly the 5 canonical transfer phases, each present exactly once.
- Exactly the 3 canonical verification states, each present exactly once.
- All transition `from`/`to` values reference valid canonical phase IDs.
- All required fields present, no extra fields.

The schema does NOT enumerate the exact legal transition set.

**Rust validators** are the executable authority for legal transitions:
- `is_valid_session_transition(from, to)` — 9 legal pairs, 16 rejected.
- `is_valid_transfer_transition(from, to)` — 10 legal pairs, 15 rejected.
- `is_transfer_allowed(connected, verification)` — policy P1.
- All validators are exhaustively tested (full N×N matrix coverage).

The JSON instance (`session_contract.v1.json`) documents the intended transitions
for human readers. The Rust code enforces them. If the JSON and Rust disagree,
Rust is authoritative.

## Implementation Guidance

### What conformance means

- Product MUST model all 5 session phases. Mapping from product-specific names
  is acceptable (e.g., native `pairingPending` maps to canonical `incoming_request`).
- Product MUST enforce all REQUIRED invariants.
- Product SHOULD enforce RECOMMENDED invariants.
- Product MAY add product-specific transient states (e.g., `disconnected(reason)`
  as a local presentation state, `connectingPhase` animation timers) that do not
  appear in the contract.

### What is explicitly out of scope (product-specific)

- Event source (IPC polling, WebRTC callbacks, signaling)
- UI animation states (connecting phase timers, pulsing indicators)
- Device name resolution
- Signaling and discovery mechanism
- Transport selection (WS, WT, WebRTC, QUIC)
- File picker / drop zone UX
- Daemon lifecycle management
- Disconnect reason display (product-specific presentation)

## Deferred to v2

- `disconnecting` session phase (transient, not externally observable)
- `mismatch` verification state (policy-level today, not a canonical type)
- `paused`, `canceled_by_sender`, `canceled_by_receiver` transfer phases
- Generation counter promoted from RECOMMENDED to REQUIRED
- CI-enforced cross-product parity testing
