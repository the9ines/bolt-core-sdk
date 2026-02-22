# State

Current state of the bolt-core-sdk repository.

## Current Version

**Tag:** `ops-20260222-phase4d`
**Commit:** `5c009d1`
**Branch:** `main`
**TS Package:** `@the9ines/bolt-core` v0.1.0
**Rust Crate:** `bolt-core` v0.1.0 (scaffold)

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
| Rust crate (`bolt-core`) | Scaffold (`rust/bolt-core/`) |
| SDK Authority Model | Complete (`docs/SDK_AUTHORITY.md`) |
| SDK Stability Contract | Complete (`docs/SDK_STABILITY.md`) |
| Transport Contract | Complete (`docs/TRANSPORT_CONTRACT.md`) |
| Ecosystem Strategy | Complete (`docs/ECOSYSTEM_STRATEGY.md`) |
| Interop Test Plan | Complete (`docs/INTEROP_TEST_PLAN.md`) |
| Golden test vectors | Complete (`ts/bolt-core/__tests__/vectors/`) |
| Vector compatibility gate (Rust) | Complete (`rust/bolt-core/tests/vector_compat.rs`) |
| API surface guard (TS) | Complete (`scripts/audit-exports.mjs`) |
| Rust CI workflow | Complete (`.github/workflows/ci-rust.yml`) |

## Test Summary

- TypeScript: 66 tests (vitest), 6 test files
- Rust: 4 tests (1 unit + 3 vector compat)
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
| Phase 4D | Rust canonical SDK declaration | **Complete** |

## Downstream Consumers

| Repo | Pinned SDK Version |
|------|--------------------|
| localbolt | `0.0.5` (upgrade to 0.1.0 pending) |
| localbolt-app | `0.0.5` (upgrade to 0.1.0 pending) |
| localbolt-v3 | `0.0.5` (upgrade to 0.1.0 pending) |
| bolt-daemon | Compatible (Rust, consumes same types) |
