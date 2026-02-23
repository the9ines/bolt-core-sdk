# State

Current state of the bolt-core-sdk repository.

## Current Version

**Tags:** `transport-web-v0.3.0-sas-verification`
**Commit:** `ec0b878`
**Branch:** `main`
**TS Package:** `@the9ines/bolt-core` v0.2.0
**TS Package:** `@the9ines/bolt-transport-web` v0.3.0
**Rust Crate:** `bolt-core` v0.1.0 (vectors complete, constants aligned)

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
| TypeScript SDK (`@the9ines/bolt-core`) | Published (v0.2.0) |
| Transport Web (`@the9ines/bolt-transport-web`) | **Published (v0.3.0)** |
| Rust crate (`bolt-core`) | Vectors complete (`rust/bolt-core/`) |
| Rust vector generator | Complete (`rust/bolt-core/src/vectors.rs`) |
| SDK Authority Model | Complete (`docs/SDK_AUTHORITY.md`) |
| SDK Stability Contract | Complete (`docs/SDK_STABILITY.md`) |
| Transport Contract | Complete (`docs/TRANSPORT_CONTRACT.md`) |
| Ecosystem Strategy | Complete (`docs/ECOSYSTEM_STRATEGY.md`) |
| Interop Test Plan | Complete (`docs/INTEROP_TEST_PLAN.md`) |
| Golden test vectors | Complete (`ts/bolt-core/__tests__/vectors/`) |
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

## Test Summary

- TypeScript (bolt-core): 73 tests (vitest), 8 test files
- TypeScript (bolt-transport-web): 53 tests (vitest, jsdom)
- Rust: 7 tests (2 unit + 3 vector compat + 2 vector equivalence)
- Golden vector suites: box-payload, framing
- API surface drift detection: `npm run audit-exports`
- Cross-language verification: `npm run verify:all`

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

## Downstream Consumers

| Repo | bolt-core Version | bolt-transport-web Version | Drift Guard | Pin Guard | Single-Install Guard |
|------|-------------------|---------------------------|-------------|-----------|---------------------|
| localbolt | `0.0.5` | `0.1.0` | CI active | CI active | CI active |
| localbolt-app | `0.0.5` | `0.1.0` | CI active | CI active | CI active |
| localbolt-v3 | `0.0.5` | `0.1.0` | CI active | CI active | CI active |
| bolt-daemon | Compatible (Rust) | N/A | N/A | N/A | N/A |
