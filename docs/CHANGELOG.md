# Changelog

All notable changes to bolt-core-sdk are documented here. Newest first.

## [sdk-v0.1.2-sas-canonical] - 2026-02-23

### Removed
- `getVerificationCode()` from `ts/bolt-transport-web/src/services/webrtc/WebRTCService.ts`.
  Shadow SAS implementation using ephemeral-only keys (diverges from spec).
  Zero callers across ecosystem. `computeSas()` in bolt-core is the ONLY
  canonical SAS implementation.

### Added
- `scripts/verify-no-shadow-sas.sh` — enforcement script ensuring no SAS
  logic exists in bolt-transport-web.
- Canonical note in `ts/bolt-core/src/sas.ts` documenting single-source rule.

## [sdk-v0.1.1-constants-alignment] - 2026-02-23

### Fixed
- Rust `PEER_CODE_LENGTH`: 4 → 6 (aligned to TS canonical value).
- Rust `SAS_LENGTH`: 4 → 6 (aligned to TS canonical value).
- Rust `PEER_CODE_ALPHABET`: 36-char ambiguous → 31-char unambiguous
  (removes 0/O, 1/I/L, aligned to TS canonical value).
- `PROTOCOL.md` §14: "(32 chars)" → "(31 chars, unambiguous subset: no 0/O, 1/I/L)".

### Added
- `peer_code_alphabet_length` test asserting 31 chars and no ambiguous characters.
- `scripts/verify-constants.sh` — cross-language (Rust ↔ TS) constants verification.
- Root `package.json` with `verify:constants`, `verify:no-shadow-sas`, `verify:all` scripts.

## [ops-20260222-phase4e] - 2026-02-22

### Added
- `rust/bolt-core/src/vectors.rs` — deterministic golden vector generator
  in Rust. Reproduces all 12 vectors (4 box-payload, 4 corrupt, 4 framing)
  byte-for-byte equivalent to TypeScript output.
- `rust/bolt-core/tests/vector_equivalence.rs` — semantic JSON equivalence
  gate comparing Rust-generated vectors against committed golden vectors.
- `crypto_box` and `base64` dependencies for NaCl box operations in Rust.

## [transport-web-v0.1.0] - 2026-02-22

### Added
- `ts/bolt-transport-web/` — new package `@the9ines/bolt-transport-web`.
  Extracts 17 shared browser source files from localbolt, localbolt-app,
  and localbolt-v3 into a single published npm package.
- Components: `createDeviceDiscovery`, `createFileUpload`,
  `createTransferProgress`, `createConnectionStatus`.
- Services: `WebRTCService`, `WebSocketSignaling`, `DualSignaling`.
- Signaling types: `SignalingProvider`, `SignalMessage`, `DiscoveredDevice`.
- State: `store`, `AppState`, `ConnectionRequest`.
- UI: `icons`, `showToast`.
- Lib: `escapeHTML`, `detectDevice`, `getLocalOnlyRTCConfig`,
  `isPrivateIP`, `isLocalCandidate`.
- Error types: `SignalingError` (new), plus re-exports from bolt-core.
- `ts/bolt-transport-web/API.md` — full exported symbol reference.
- `.github/workflows/publish-transport-web.yml` — CI publish workflow
  for `transport-web-v*` tags with smoke test.

### Fixed
- `.github/workflows/publish-bolt-core.yml` — removed invalid
  `working-directory: /tmp/smoke-test` from smoke test step.

## [ops-20260222-phase4d] - 2026-02-22

### Added
- `rust/bolt-core/` — Rust crate skeleton (canonical SDK declaration).
  Constants matching TS SDK. Scaffold only, no crypto primitives yet.
- `rust/bolt-core/tests/vector_compat.rs` — plumbing gate that parses
  existing golden vectors and validates structure, fields, counts, and
  framing invariants. 4 Rust tests.
- `docs/SDK_AUTHORITY.md` — authority model: Rust canonical, TS adapter,
  golden vectors as interop gate. Core rule: canonical truth = contracts +
  vectors + Rust crate.
- `.github/workflows/ci-rust.yml` — Rust CI (fmt, clippy, test). Additive,
  does not affect TS CI.

### Changed
- `docs/SDK_STABILITY.md` — added §6 Authority section.

## [sdk-v0.1.0-stable-api] - 2026-02-22

### Added
- `docs/SDK_STABILITY.md` — public API surface definition, SemVer versioning
  policy, breaking change checklist, transport interface stability, bolt-daemon
  compatibility contract.
- `README.md` — API Stability section linking to SDK_STABILITY.md.

### Changed
- `docs/TRANSPORT_CONTRACT.md` — SDK Stability Alignment cross-reference added.
- `ts/bolt-core/package.json` — version bumped from `0.0.5` to `0.1.0`.
  First stable API freeze.

## [sdk-v0.0.5-phase4b] - 2026-02-22

### Added
- `docs/TRANSPORT_CONTRACT.md` §8 — P2P-First Policy and Relay Optionality.

### Changed
- `docs/ECOSYSTEM_STRATEGY.md` §7 — headless transport lane updated:
  libdatachannel standardized, webrtc-rs optional evaluation, relay future.

## [sdk-v0.0.5-phase2a] - 2026-02-21

### Added
- `scripts/audit-exports.mjs` — API surface guard (drift detection).
- `scripts/smoke.sh` — publish smoke test.
- `docs/SUPPLY_CHAIN.md` — SDK package lifecycle documentation.

### Changed
- CI and release discipline improvements (Phase 2A).

## Phases 2C / 2D (untagged as sdk releases)

### Added
- `PROTOCOL.md` — transport contract, interop test plan, golden test vectors.
- `docs/TRANSPORT_CONTRACT.md` — transport abstraction boundary.
- `docs/INTEROP_TEST_PLAN.md` — interop verification plan.
- `docs/ECOSYSTEM_STRATEGY.md` — ecosystem governance.
- `__tests__/vectors/` — box-payload and framing golden vectors.
- Transport conformance hardening and vector guardrails (Phase 2D).

## [sdk-v0.0.5] - 2026-02-21

### Fixed
- Use default imports for CJS dependencies (tweetnacl, tweetnacl-util).

## [sdk-v0.0.4] - 2026-02-21

### Added
- GitHub Packages publish workflow (`.github/workflows/publish.yml`).

## [sdk-v0.0.3] - 2026-02-21

### Added
- `@the9ines/bolt-core` TypeScript SDK package.
- Core crypto primitives: `generateEphemeralKeyPair`, `sealBoxPayload`,
  `openBoxPayload`.
- Encoding: `toBase64`, `fromBase64`.
- Peer codes: `generateSecurePeerCode`, `generateLongPeerCode`,
  `isValidPeerCode`, `normalizePeerCode`.
- Hashing: `sha256`, `bufferToHex`, `hashFile`.
- SAS: `computeSas`.
- Error types: `BoltError`, `EncryptionError`, `ConnectionError`,
  `TransferError`.
- Constants: `NONCE_LENGTH`, `PUBLIC_KEY_LENGTH`, `SECRET_KEY_LENGTH`,
  `DEFAULT_CHUNK_SIZE`, `PEER_CODE_LENGTH`, `PEER_CODE_ALPHABET`,
  `SAS_LENGTH`, `BOLT_VERSION`.
- 66 tests (vitest).

## [sdk-v0.0.2] - 2026-02-21

### Added
- `docs/CORE_EXTRACTION_MAP.md` — Phase 0 audit of all product repos.

## [sdk-v0.0.1] - 2026-02-19

### Added
- `PROTOCOL.md` — Bolt Core v1 specification (Draft).
- `LOCALBOLT_PROFILE.md` — LocalBolt Profile v1 specification (Draft).
- `README.md` — ecosystem overview.
- `LICENSE` — MIT.
