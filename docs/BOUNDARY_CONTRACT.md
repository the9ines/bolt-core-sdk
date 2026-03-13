# Bolt Core SDK — Consumer Boundary Contract

Defines how downstream products consume the Rust core API across different
boundary types. This document is the AC-RC-06 deliverable (verification
closure mode).

Keywords: RFC 2119 (MUST, MUST NOT, REQUIRED, SHALL, SHOULD, MAY).

## Closure Mode Decision (AC-RC-06)

**Verification closure** — the existing boundary mechanisms are the
canonical contract. No UniFFI, cbindgen, or formal FFI layer is required.

**Rationale:**
1. Native consumers (bolt-daemon) are Rust — direct crate dependency, no FFI.
2. Tauri app communicates with daemon via IPC (sidecar), not linked library.
3. Browser path has wasm-bindgen coverage for policy.
4. No Swift/Kotlin consumers exist (bytebolt-app is stub).
5. Building UniFFI now would be speculative infrastructure with no consumer.

If a non-Rust native consumer materializes (Swift, Kotlin, C++), this
document MUST be revised and a dedicated FFI pass opened.

---

## Boundary Types

Three boundary mechanisms serve the full consumer matrix:

| Type | Mechanism | Consumers | Error Model |
|------|-----------|-----------|-------------|
| **Rust-direct** | Cargo path dependency | bolt-daemon | Native `Result<T, BoltError>` / `Result<T, BtrError>` / `Result<T, TransferError>` |
| **WASM** | wasm-bindgen | localbolt-v3 (browser) | Flat DTOs, no panics, no Result (errors encoded in return struct) |
| **Tauri IPC** | NDJSON over Unix socket / Windows named pipe | localbolt-app | `Result<T, String>` via Tauri commands; events are fire-and-forget |

---

## Boundary 1: Rust-Direct (bolt-daemon)

### Dependency Declaration

```toml
# bolt-daemon/Cargo.toml
bolt-core          = { path = "../bolt-core-sdk/rust/bolt-core" }
bolt-transfer-core = { path = "../bolt-core-sdk/rust/bolt-transfer-core" }
```

### Consumed Surface

| Crate | Imports Used |
|-------|-------------|
| `bolt-core` | `crypto::{seal_box_payload, KeyPair}`, `identity::{generate_identity_keypair, IdentityKeyPair}`, `encoding::to_base64`, `hash::sha256_hex` |
| `bolt-transfer-core` | `SendSession`, `ReceiveSession`, `TransferState` |

### Error Propagation

bolt-daemon handles errors from the SDK natively:
- `BoltError` variants map to daemon log tokens and exit codes.
- `TransferError` variants map to IPC event payloads for the frontend.
- No serialization boundary — errors flow as Rust types.

### Lifecycle Expectations

- Daemon links crates at compile time (static linking).
- No runtime discovery or dynamic loading.
- Crate versions are pinned by `Cargo.lock` in the daemon repo.
- Breaking SDK changes require daemon rebuild and version bump.

---

## Boundary 2: WASM (Browser via bolt-transfer-policy-wasm)

### Binding Mechanism

`wasm-bindgen` with `cdylib` + `rlib` crate type. All exports use
`#[wasm_bindgen]` with explicit `js_name` mappings.

### Exported Functions

| JS Name | Parameters | Returns |
|---------|------------|---------|
| `policyDecide` | 11 scalar/array params (flat) | `WasmScheduleDecision` |
| `policyDetectStall` | 5 scalar params | `WasmStallResult` |
| `policyProgressCadence` | 6 scalar params | `WasmProgressResult` |

### Design Constraints

- **Flat signatures only.** wasm-bindgen does not support nested struct
  parameters. All inputs are scalar or `&[u32]`.
- **No Result returns.** Errors are encoded in the return struct
  (e.g., `should_emit: false` for suppressed progress).
- **No panics.** All inputs are valid by construction (enum discriminants,
  bounded integers).
- **Enum flattening.** Rust enums with data payloads (e.g.,
  `StallClassification::Warning { ms }`) are converted to tag + field
  structs for the WASM boundary.

### Parity Enforcement

6 native parity tests in `bolt-transfer-policy-wasm/src/lib.rs` verify
that WASM boundary outputs match direct native calls for identical inputs.

---

## Boundary 3: Tauri IPC (localbolt-app ↔ bolt-daemon)

### Architecture

localbolt-app does **not** link bolt-core-sdk as a library. The Tauri
shell manages the bolt-daemon as a sidecar process and communicates via
IPC.

```
┌──────────────┐    Tauri IPC     ┌──────────────┐    NDJSON/socket    ┌──────────────┐
│  React/TS UI │ ◄──────────────► │  Tauri Rust  │ ◄────────────────► │  bolt-daemon │
│  (WebView)   │  invoke + events │  (commands)  │  Unix/named pipe   │  (sidecar)   │
└──────────────┘                  └──────────────┘                    └──────────────┘
                                                                       │
                                                                       ├── bolt-core
                                                                       └── bolt-transfer-core
```

### Tauri Command Surface (Rust → Frontend)

| Command | Direction | Return | Purpose |
|---------|-----------|--------|---------|
| `get_watchdog_state` | TS → Rust | `WatchdogStateResponse` | Poll lifecycle state |
| `get_signal_status` | TS → Rust | Signal health probe | Health check |
| `restart_daemon` | TS → Rust | `String` | Manual restart |
| `send_pairing_decision` | TS → Rust | `String` | Relay user pairing choice |
| `send_transfer_decision` | TS → Rust | `String` | Relay user transfer choice |
| `export_support_bundle` | TS → Rust | `String` | Diagnostic export |

### Tauri Event Surface (Rust → Frontend)

| Event | Payload | Trigger |
|-------|---------|---------|
| `daemon://watchdog-state` | `{ state, retry_count }` | Lifecycle transitions |
| `daemon://status-update` | `{ connected_peers, ui_connected, version }` | Daemon status changes |
| `daemon://pairing-request` | `{ request_id, remote_device_name, sas, ... }` | Incoming pairing |
| `daemon://transfer-request` | `{ request_id, file_name, file_size_bytes, ... }` | Incoming transfer |
| `daemon://bridge-disconnected` | `()` | IPC connection lost |
| `signal://status` | `{ status, consecutive_failures }` | Signal server health |

### NDJSON IPC Protocol (Tauri ↔ Daemon)

Wire format: newline-delimited JSON. Each message ends with `\n`.

```json
{
  "id": "app-0",
  "kind": "decision",
  "type": "pairing.decision",
  "ts_ms": 1710000000000,
  "payload": { "request_id": "...", "decision": "allow_once" }
}
```

Kinds: `event` (daemon → app), `decision` (app → daemon).

Version handshake is mandatory first exchange. See
`bolt-daemon/docs/DAEMON_CONTRACT.md` for full IPC specification.

### Error Model

- Tauri commands return `Result<T, String>` — errors are stringified.
- Events are fire-and-forget (no acknowledgment).
- IPC failures trigger `daemon://bridge-disconnected` event.
- Daemon unreachable → watchdog FSM transitions through `restarting` →
  `degraded` states.

### Lifecycle

1. Tauri app spawns bolt-daemon as sidecar process.
2. Watchdog probes readiness via one-shot IPC handshake.
3. On success, persistent IPC bridge starts event forwarding.
4. On crash, watchdog retries with exponential backoff (max 3).
5. After max retries, enters `degraded` state (manual restart required).

---

## Ad-Hoc / Duplicate Path Audit

| Path | Status | Notes |
|------|--------|-------|
| bolt-daemon envelope.rs | **Active** | Daemon's own envelope codec for DataChannel frames. Uses `bolt-core::crypto` internally. Not a duplicate — it implements the profile-level envelope, not the core envelope. |
| bolt-daemon web_hello.rs | **Active** | HELLO protocol orchestration. Uses `bolt-core::crypto::seal_box_payload`. Lifecycle logic (pre_hello/post_hello/closed) is daemon-owned — migration to shared code is AC-RC-07 scope. |
| bolt-daemon session.rs | **Active** | SessionContext holding negotiated keys. Thin wrapper, not duplicating SDK logic. |
| localbolt-app Tauri crate-type `staticlib`/`cdylib` | **Unused** | Configured in Cargo.toml but no `extern "C"` functions exist. Retained for potential future use. No action needed. |

**Deferred to AC-RC-07:** Session/handshake lifecycle (pre_hello → post_hello → closed, verification state, capability dispatch) currently lives in bolt-daemon's `web_hello.rs` and `session.rs`. Migration to a shared crate is AC-RC-07 scope and MUST NOT be attempted in this pass.

---

## Contract Versioning

This contract is versioned with the SDK. Changes to boundary mechanisms
require:
1. Update this document.
2. Update consumer integration tests.
3. Coordinate version bumps across affected repos.

New boundary types (e.g., UniFFI for Swift/Kotlin) require a dedicated
implementation pass with PM approval, not ad-hoc addition.
