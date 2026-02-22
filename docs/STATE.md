# State

Current state of the bolt-core-sdk repository.

## Current Version

**Tag:** `sdk-v0.1.0-stable-api`
**Commit:** `8110bb5`
**Branch:** `main`
**Package:** `@the9ines/bolt-core` v0.1.0

## Public API Status

The public API surface is **frozen** as of v0.1.0 under SemVer.
See [SDK_STABILITY.md](SDK_STABILITY.md) for the stability contract.

## Contents

| Item | Status |
|------|--------|
| Bolt Core v1 spec | Draft (`PROTOCOL.md`) |
| LocalBolt Profile v1 spec | Draft (`LOCALBOLT_PROFILE.md`) |
| TypeScript SDK (`@the9ines/bolt-core`) | Published (v0.1.0) |
| Core Extraction Map | Complete (`docs/CORE_EXTRACTION_MAP.md`) |
| SDK Stability Contract | Complete (`docs/SDK_STABILITY.md`) |
| Transport Contract | Complete (`docs/TRANSPORT_CONTRACT.md`) |
| Ecosystem Strategy | Complete (`docs/ECOSYSTEM_STRATEGY.md`) |
| Interop Test Plan | Complete (`docs/INTEROP_TEST_PLAN.md`) |
| Golden test vectors | Complete (`__tests__/vectors/`) |
| API surface guard | Complete (`scripts/audit-exports.mjs`) |
| Rust SDK implementation | Not started |

## Test Summary

- 66 tests (vitest), 6 test files
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
| Phase 4C | Freeze SDK public API surface | **Complete** |

## Downstream Consumers

| Repo | Pinned SDK Version |
|------|--------------------|
| localbolt | `0.0.5` (upgrade to 0.1.0 pending) |
| localbolt-app | `0.0.5` (upgrade to 0.1.0 pending) |
| localbolt-v3 | `0.0.5` (upgrade to 0.1.0 pending) |
| bolt-daemon | Compatible (Rust, consumes same types) |
