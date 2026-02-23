# State

Current state of the bolt-core-sdk repository.

## Current Version

**Tag:** `transport-web-v0.1.0`
**Commit:** `08ff266`
**Branch:** `main`
**TS Package:** `@the9ines/bolt-core` v0.1.0
**TS Package:** `@the9ines/bolt-transport-web` v0.1.0
**Rust Crate:** `bolt-core` v0.1.0 (vectors complete)

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
| TypeScript SDK (`@the9ines/bolt-core`) | Published (v0.1.0) |
| Transport Web (`@the9ines/bolt-transport-web`) | **Published (v0.1.0)** |
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

## Test Summary

- TypeScript: 66 tests (vitest), 6 test files
- Rust: 6 tests (1 unit + 3 vector compat + 2 vector equivalence)
- Golden vector suites: box-payload, framing
- API surface drift detection: `npm run audit-exports`

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

## Downstream Consumers

| Repo | bolt-core Version | bolt-transport-web Version | Drift Guard | Pin Guard | Single-Install Guard |
|------|-------------------|---------------------------|-------------|-----------|---------------------|
| localbolt | `0.0.5` | `0.1.0` | CI active | CI active | CI active |
| localbolt-app | `0.0.5` | `0.1.0` | CI active | CI active | CI active |
| localbolt-v3 | `0.0.5` | `0.1.0` | CI active | CI active | CI active |
| bolt-daemon | Compatible (Rust) | N/A | N/A | N/A | N/A |
