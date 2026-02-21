# State

Current state of the bolt-core-sdk repository.

## Current Version

**Tag:** `sdk-v0.0.2`
**Commit:** `3801263`
**Branch:** `main`

## Contents

| Item | Status |
|------|--------|
| Bolt Core v1 spec | Draft (PROTOCOL.md) |
| LocalBolt Profile v1 spec | Draft (LOCALBOLT_PROFILE.md) |
| Core Extraction Map | Complete (docs/CORE_EXTRACTION_MAP.md) |
| TypeScript SDK implementation | Not started |
| Rust SDK implementation | Not started |
| Conformance test vectors | Not started |

## Phase Status

| Phase | Description | Status |
|-------|-------------|--------|
| Phase 0 | Audit and Map | **Complete** |
| Phase 1 | SDK Scaffold | Not started |
| Phase 2 | Code Extraction | Not started |
| Phase 3 | Profile Adapters | Not started |
| Phase 4 | Dependency Integration | Not started |
| Phase 5 | Release Discipline | Not started |

## Audit Findings (Phase 0)

- All 3 product repos have identical core implementations
- 1 trivial comment diff in localbolt-app crypto-utils.ts
- Tests exist only in localbolt (10 files)
- No persistent identity / TOFU / encrypted HELLO in any product
- SAS implemented but not displayed in UI
