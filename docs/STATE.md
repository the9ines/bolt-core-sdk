# State

Current state of the bolt-core-sdk repository.

## Current Version

**Tags (main):** `sdk-v0.4.0-dead-exports-cleanup`
**Tags (feature branch):** `sdk-v0.5.0-h2-webrtc-enforcement`, `sdk-v0.5.1`
**Commit (main):** `064bfe0`
**Commit (feature):** `9d8617d` (H3, on `feature/h3-golden-vectors`)
**Branch:** `main` (H2/H3 on `feature/h3-golden-vectors`, not yet merged)
**TS Package:** `@the9ines/bolt-core` v0.4.0 (dead exports removed, 21 public exports)
**TS Package:** `@the9ines/bolt-transport-web` v0.6.0 (Profile Envelope v1)
**Rust Crate:** `bolt-core` v0.4.0 (R0-R3 complete, all 8 modules at full TS parity)

## Authority Model

**Canonical truth = contracts + vectors + Rust crate.**
TypeScript is supported but MUST pass compatibility tests against
canonical vectors. See [SDK_AUTHORITY.md](SDK_AUTHORITY.md).

## Public API Status

The TypeScript public API surface is **frozen** as of v0.1.0 under SemVer.
See [SDK_STABILITY.md](SDK_STABILITY.md) for the stability contract.

## Contents

| Item | Status |
|------|--------|
| Bolt Core v1 spec | Draft (`PROTOCOL.md`) |
| LocalBolt Profile v1 spec | Draft (`LOCALBOLT_PROFILE.md`) |
| TypeScript SDK (`@the9ines/bolt-core`) | Published (v0.4.0) |
| Transport Web (`@the9ines/bolt-transport-web`) | **v0.6.0** (Profile Envelope v1) |
| Rust crate (`bolt-core`) | v0.4.0 — all 8 modules complete, full TS parity (`rust/bolt-core/`) |
| Rust vector generator | Complete (`rust/bolt-core/src/vectors.rs`) |
| SDK Authority Model | Complete (`docs/SDK_AUTHORITY.md`) |
| SDK Stability Contract | Complete (`docs/SDK_STABILITY.md`) |
| Transport Contract | Complete (`docs/TRANSPORT_CONTRACT.md`) |
| Ecosystem Strategy | Complete (`docs/ECOSYSTEM_STRATEGY.md`) |
| Interop Test Plan | Complete (`docs/INTEROP_TEST_PLAN.md`) |
| Golden test vectors (box-payload, framing) | Complete (`ts/bolt-core/__tests__/vectors/`) |
| H3 golden vectors (SAS, HELLO-open, envelope-open) | Complete, on feature branch (`ts/bolt-core/__tests__/vectors/`) |
| H3 vector generator | Complete, on feature branch (`ts/bolt-core/scripts/generate-h3-vectors.mjs`) |
| Vector compatibility gate (Rust) | Complete (`rust/bolt-core/tests/vector_compat.rs`) |
| Vector equivalence gate (Rust) | Complete (`rust/bolt-core/tests/vector_equivalence.rs`) |
| API surface guard (TS) | Complete (`scripts/audit-exports.mjs`) |
| Rust CI workflow | Complete (`.github/workflows/ci-rust.yml`) |
| Transport-web publish workflow | Complete (`.github/workflows/publish-transport-web.yml`) |
| Transport upgrade protocol | Complete (`docs/TRANSPORT_UPGRADE_PROTOCOL.md`) |
| Constants verification script | Complete (`scripts/verify-constants.sh`) |
| Shadow SAS enforcement script | Complete (`scripts/verify-no-shadow-sas.sh`) |
| Root verify scripts (npm) | Complete (`package.json` — `verify:all`) |
| Identity primitives (bolt-core) | Complete (`ts/bolt-core/src/identity.ts`) |
| Identity persistence (transport-web) | Complete (`src/services/identity/identity-store.ts`) |
| TOFU pin store (transport-web) | Complete (`src/services/identity/pin-store.ts`) |
| HELLO protocol (transport-web) | Complete (`WebRTCService` — encrypted identity exchange) |
| SAS verification (transport-web) | Complete (`WebRTCService` — computeSas after TOFU, verification UI) |
| Pin store schema v2 (transport-web) | Complete (`pin-store.ts` — PinRecord with verified flag, lazy migration) |
| Verification status component (transport-web) | Complete (`verification-status.ts` — imperative DOM component) |
| Replay protection (transport-web) | Complete (`WebRTCService` — transferId, dedup, bounds, identity binding) |
| Strict handshake gating (transport-web) | Complete (`WebRTCService` — INVALID_STATE + disconnect for pre-handshake messages) |
| Peer code security model (PROTOCOL.md) | Complete (§2 — routing hint, not auth secret; length policy locked) |
| HELLO capabilities plumbing (transport-web) | Complete (`WebRTCService` — capabilities negotiation, `hasCapability()` accessor) |
| Conformance mapping (docs) | Complete (`docs/conformance/LOCALBOLT_CONFORMANCE.md` — 5 behaviors, 31 tests) |
| File integrity verification (bolt-core) | Complete (`hashFile` widened to Blob, `IntegrityError` added) |
| File integrity verification (transport-web) | Complete (`WebRTCService` — `bolt.file-hash` capability, sender hash, receiver verify, fail-closed) |
| File integrity verification (spec) | Complete (`LOCALBOLT_PROFILE.md` §13 — capability gate, wire format, backward compat) |
| Audit tracker | Complete (`docs/AUDIT_TRACKER.md` — 26 findings, 22 DONE, 3 DEFERRED) |
| Security posture declaration | Complete (`docs/SECURITY_POSTURE.md` — WebRTC DataChannel profile) |
| Rust core plan | Complete (`docs/ecosystem/RUST_CORE_PLAN.md` — crate layout, API, phased milestones) |
| Ecosystem docs index | Complete (`docs/ecosystem/README.md`) |

## Test Summary

- TypeScript (bolt-core): 76 tests (vitest), 7 test files
- TypeScript (bolt-transport-web): 117 tests (vitest, jsdom), 11 test files
- Rust: 59 tests (54 unit + 3 vector compat + 2 vector equivalence)
- Golden vector suites: box-payload, framing, H3 (SAS, HELLO-open, envelope-open)
- API surface drift detection: `npm run audit-exports`
- Cross-language verification: `npm run verify:all`
- H2 enforcement tests: 21 (on feature branch)
- H3 vector tests: TS 94 total, Rust 68 total (on feature branch)

## Phase Status

| Phase | Description | Status |
|-------|-------------|--------|
| Phase 0 | Audit and Map | Complete |
| Phase 2A | Release discipline | Complete |
| Phase 2C | Protocol contracts + vectors | Complete |
| Phase 2D | Transport conformance hardening | Complete |
| Phase 4B | P2P-first transport docs | Complete |
| Phase 4C | Freeze SDK public API surface | Complete |
| Phase 4D | Rust canonical SDK declaration | Complete |
| Phase 4E | Rust becomes vector authority | Complete |
| Phase 4F | Extract shared browser transport | **Complete** (published, consumed by all 3 products) |
| Phase 4G | Cross-repo transport verification | **Complete** (parity confirmed, drift guards installed) |
| Phase 4H | Release pipeline hardening | **Complete** (upgrade protocol, pin + single-install CI guards) |
| Phase 6A.1 | Constants alignment (Rust → TS canonical) | **Complete** (PEER_CODE, SAS, alphabet) |
| Phase 6A.2 | SAS canonicalization | **Complete** (shadow SAS removed, enforcement script) |
| Phase 6B | Web transport security hardening | **Complete** (S7 ephemeral key lifecycle, S6 filename XSS, 17 tests) |
| Phase 7A | Encrypted HELLO + TOFU identity pinning | **Complete** (identity primitives, HELLO protocol, pin stores, 21 new tests) |
| Phase 7B | SAS verification surface | **Complete** (pin store schema evolution, SAS after HELLO, verification UI, 15 new tests) |
| Phase 8A | Replay protection + chunk bounds | **Complete** (transferId, dedup, bounds, identity binding, legacy compat, 9 new tests) |
| Phase 8B.1 | WebRTCService lifecycle test coverage | **Complete** (12 tests: sender flow, receiver paths, control msgs, cleanup, error paths) |
| Phase 8D | Strict handshake gating (S4 close) | **Complete** (fail-closed INVALID_STATE + disconnect, 6 new tests) |
| Phase 8E | Peer code modulo bias elimination | **Complete** (rejection sampling, 3 new tests) |
| Phase 9C | Conformance mapping | **Complete** (5 behaviors, 31 test cases, all CONFORMANT) |
| Phase 0 | HELLO capabilities plumbing | **Complete** (capabilities negotiation in encrypted HELLO, 7 new tests) |
| Phase M1 | Profile Envelope v1 | **Complete** (bolt.envelope capability, versioned metadata wrapping, 14 new tests) |
| Phase M2 | File integrity hash wiring | **Complete** (bolt.file-hash capability, SHA-256 verification, fail-closed, 16 new tests) |
| Phase A1 | Export governance + dead-export cleanup | **Complete** (7 unused constants removed from public API, 28→21 exports, v0.3.0→v0.4.0) |
| Phase R0 | Rust core scaffold + CI gates | **Complete** (8 modules, feature-gated vectors, lean default deps, 28 tests) |
| Phase R1 | Rust crypto + encoding parity | **Complete** (base64, seal/open, keygen, 41 tests, golden vector parity) |
| Phase R2 | Rust identity + sas + hash parity | **Complete** (sha256, identity keygen, compute_sas, 52 tests) |
| Phase R3 | Rust peer code generation parity | **Complete** (rejection sampling, all 8 modules complete, 59 tests) |
| Phase H2 | WebRTC enforcement compliance | **Complete** on feature branch (exactly-once HELLO, envelope-required, fail-closed, 21 tests) |
| Phase H3 | Cross-implementation golden vectors | **Complete** on feature branch (SAS, HELLO-open, envelope-open; TS 94 tests, Rust 68 tests) |

## Downstream Consumers

| Repo | bolt-core Version | bolt-transport-web Version | Drift Guard | Pin Guard | Single-Install Guard |
|------|-------------------|---------------------------|-------------|-----------|---------------------|
| localbolt | `0.3.0` | `0.6.0` | CI active | CI active | CI active |
| localbolt-app | `0.3.0` | `0.6.0` | CI active | CI active | CI active |
| localbolt-v3 | `0.3.0` | `0.6.0` | CI active | CI active | CI active |
| bolt-daemon | Compatible (Rust) | N/A | N/A | N/A | N/A |
