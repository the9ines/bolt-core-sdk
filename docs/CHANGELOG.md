# Changelog

All notable changes to bolt-core-sdk are documented here. Newest first.

## [sdk-v0.2.1-peer-code-bias-fix] - 2026-02-24

Phase 8E: Peer Code Modulo Bias Elimination.

Replaces biased `byte % 31` peer code generation with rejection sampling.
The old algorithm overrepresented alphabet indices 0–7 by 12.5% because
256 is not divisible by 31. The fix discards bytes >= 248
(`floor(256/31)*31`) and uses `byte % 31` only for survivors.

### Changed (bolt-core)
- `generateSecurePeerCode()` and `generateLongPeerCode()` now use
  rejection sampling via internal `fillUnbiased()` helper.
- No public API change. Alphabet, output length, and long format
  (XXXX-XXXX) are preserved.
- Version bumped from `0.2.0` to `0.2.1`.

### Tests
- New: 3 rejection sampling invariant tests (MAX=248 for N=31, stress
  tests for both generators).
- bolt-core: 76 tests (was 73, +3 rejection sampling tests).

## [transport-web-v0.4.2-strict-handshake-gating] - 2026-02-23

Phase 8D: Strict Handshake Gating (S4 audit item close).

Makes receive-side handshake gating fail-closed. Any non-HELLO message
received before `helloComplete` now triggers `[INVALID_STATE]` error,
sends plaintext error control message, and calls `disconnect()`.

### Changed (bolt-transport-web)
- `handleMessage()` now rejects all non-HELLO messages before
  `helloComplete` with `ConnectionError`, error control message
  (`{ type: "error", code: "INVALID_STATE" }`), and `disconnect()`.
- Previously: non-HELLO messages were silently ignored (control messages
  like `paused`/`cancelled` could mutate transfer state pre-handshake).
- Version bumped from `0.4.1` to `0.4.2`.

### Changed (LOCALBOLT_PROFILE.md)
- HELLO Exchange section: added normative MUST/SHOULD for pre-handshake
  rejection and note on plaintext error format.

### Tests
- New: `webrtcservice-handshake-gating.test.ts` (6 tests).
- Updated: `webrtcservice-lifecycle.test.ts` (added `helloComplete = true`
  to control message routing test for Phase 8D compatibility).
- bolt-transport-web: 80 tests (was 74, +6 handshake gating tests).

## [transport-web-v0.4.1-lifecycle-tests] - 2026-02-23

Phase 8B.1: WebRTCService Test Harness + Lifecycle Coverage.

Adds 12 lifecycle and failure-path tests for WebRTCService transfer
logic. No production code changes. Tests cover sender flow (HELLO wait,
chunk ordering, dc-not-open), receiver flow (legacy + guarded path
reconstruction, out-of-order delivery), control messages (cancel, pause,
resume, remote cancel), state cleanup (disconnect), error path
(decryption failure), bounds rejection sanity, and handleMessage routing.

### Added (bolt-transport-web)
- `webrtcservice-lifecycle.test.ts` — 12 tests covering:
  - `sendFile` blocks until `helloComplete` is resolved.
  - `sendFile` emits chunk messages in order with correct fields.
  - `cancelTransfer` sends cancel control message and clears state.
  - `pauseTransfer` / `resumeTransfer` send control messages.
  - Legacy receiver path reconstructs file and calls `onReceiveFile`.
  - Guarded receiver path reconstructs out-of-order chunks (2,0,1).
  - `handleRemoteCancel` clears all receiver state maps.
  - `disconnect` clears all transfer state.
  - Decryption failure triggers error progress and clears guarded transfer.
  - Invalid chunk fields rejected without crash (bounds sanity).
  - `handleMessage` routes pause/resume/cancel control messages.
  - `sendFile` throws when data channel is not open.

### Changed (bolt-transport-web)
- Version bumped from `0.4.0` to `0.4.1` (tests only, no API change).

### Tests
- bolt-transport-web: 74 tests (was 62, +12 lifecycle tests).

## [transport-web-v0.4.0-replay-protection] - 2026-02-23

Phase 8A: Replay Protection + Chunk Bounds (S3 audit item).

Adds protocol-level replay defenses to the file transfer path. Sender
generates a `transferId` (bytes16, hex) per transfer; receiver tracks
state per `transferId` with bounds checks, dedup, and sender-identity
binding. Legacy peers (no `transferId`) still work via unguarded path
with deprecation warning.

### Added (bolt-transport-web)
- `transferId` field on `FileChunkMessage` (optional, hex, 32 chars).
- `ActiveTransfer` receiver-side state with `receivedSet` for O(1) dedup.
- `generateTransferId()` — `crypto.getRandomValues(16)` → hex via `bufferToHex`.
- `guardedTransfers: Map<string, ActiveTransfer>` keyed by `transferId`.
- `sendTransferIds` / `recvTransferIds` maps for O(1) filename→tid lookup.
- `isValidChunkFields()` bounds validator (shared by guarded + legacy).
- `processChunkGuarded()` — dedup (`[REPLAY_DUP]`), bounds (`[REPLAY_OOB]`),
  sender-identity binding (`[REPLAY_XFER_MISMATCH]`).
- `processChunkLegacy()` — existing logic with bounds checks, `[REPLAY_UNGUARDED]` warning.
- 9 new tests in `replay-protection.test.ts` (guarded: dup, OOB, identity mismatch,
  new transfer, out-of-order, completion reset; legacy: accept, warning; sender: tid format).

### Changed (bolt-transport-web)
- `sendFile()` generates and includes `transferId` in every chunk message.
- `processChunk()` refactored into dispatcher → guarded/legacy paths.
- `cancelTransfer()`, `pauseTransfer()`, `resumeTransfer()` include `transferId` when present.
- `handleRemoteCancel()`, `disconnect()` clean up `guardedTransfers` state.
- Version bumped from `0.3.0` to `0.4.0` (backward-compatible feature addition).

### Changed (LOCALBOLT_PROFILE.md)
- Added section 12: Replay Protection — documents `transferId` semantics,
  receiver guards, dedup policy, and backward compatibility mode.

### Tests
- bolt-transport-web: 62 tests (was 53, +9 replay protection tests).

## [transport-web-v0.3.0-sas-verification] - 2026-02-23

Phase 7B: Surface SAS verification to users.

Surfaces the existing canonical `computeSas()` to users without modifying
the SAS algorithm. Adds pin store schema evolution, SAS computation in
WebRTCService after HELLO, verification status UI component, and golden
vector tests.

### Added (bolt-core)
- 3 golden vector tests (`65434F` from fixed keys) for SAS computation.
- `sas.ts` untouched — canonical implementation unchanged.

### Added (bolt-transport-web)
- `PinRecord` with `verified` boolean, `PinVerifyResult` type,
  `markVerified()` in pin store.
- IndexedDB lazy migration from v1 string format with immediate writeback.
- `WebRTCService`: `computeSas` after TOFU, `VerificationInfo` /
  `VerificationState` types, `onVerificationState` callback,
  `getVerificationInfo()`, `markPeerVerified()`.
- `verification-status.ts` — imperative DOM component
  (verified / unverified / legacy states).
- New public exports: `VerificationInfo`, `VerificationState`, `PinRecord`,
  `PinVerifyResult`, `createVerificationStatus`, `VerificationStatusOptions`.

### Changed
- `@the9ines/bolt-transport-web` version bumped from `0.2.0` to `0.3.0`
  (new public API surface).

### Tests
- bolt-core: 73 tests (was 70, +3 SAS golden vector tests).
- bolt-transport-web: 53 tests (was 38, +15 verification tests).
- `verify-no-shadow-sas`: PASS.
- `verify-constants`: PASS.

**Commit:** `ec0b878`

## [sdk-v0.2.0-identity-primitives / transport-web-v0.2.0-hello-tofu-foundation] - 2026-02-23

Phase 7A: Encrypted HELLO and TOFU identity pinning.

### Added (bolt-core)
- `src/identity.ts` — X25519 identity keypair generation (`generateIdentityKeyPair`)
  and `KeyMismatchError` for TOFU violations. Identity keys are long-lived and
  persisted by the transport layer. They MUST NOT traverse the signaling server.
- `IdentityKeyPair` type, `generateIdentityKeyPair`, `KeyMismatchError` exported
  from package index.
- `__tests__/identity.test.ts` — 4 tests (keypair generation, distinctness,
  non-zero, KeyMismatchError semantics).
- `export-snapshot.json` updated with `generateIdentityKeyPair` and
  `KeyMismatchError`.
- Canonical comment added to `dist/sas.js`.

### Added (bolt-transport-web)
- `src/services/identity/identity-store.ts` — `IdentityPersistence` interface,
  `IndexedDBIdentityStore` (browser), `MemoryIdentityStore` (tests),
  `getOrCreateIdentity()` helper. Manages long-lived local identity keypairs.
- `src/services/identity/pin-store.ts` — `PinPersistence` interface,
  `IndexedDBPinStore` (browser), `MemoryPinStore` (tests),
  `verifyPinnedIdentity()` — TOFU verification (pin on first contact,
  verify on subsequent, fail-closed `KeyMismatchError` on mismatch).
- `src/__tests__/hello.test.ts` — 6 tests covering encrypted HELLO message
  round-trip, wrong-key rejection, identity-not-in-outer-envelope,
  full two-peer HELLO exchange, HELLO+TOFU pin/verify/reject integration.
- `src/__tests__/identity-store.test.ts` — 5 tests for MemoryIdentityStore
  and getOrCreateIdentity.
- `src/__tests__/pin-store.test.ts` — 10 tests for MemoryPinStore and
  verifyPinnedIdentity (pin, verify, reject, KeyMismatchError fields).
- `WebRTCServiceOptions` interface — `identityPublicKey` and `pinStore`
  constructor options.
- `WebRTCService` HELLO protocol: encrypted HELLO sent over DataChannel
  on open, `processHello()` decrypts and runs TOFU verification,
  5s legacy timeout for peers without HELLO support, `waitForHello()`
  gate before file transfer, `isLegacySession()` accessor.
- TOFU violation handling: `KeyMismatchError` triggers `ConnectionError`
  and `disconnect()`.
- Identity & TOFU symbols exported from package index.

### Changed
- `@the9ines/bolt-core` version bumped from `0.1.0` to `0.2.0`.
- `@the9ines/bolt-transport-web` version bumped from `0.1.1` to `0.2.0`.
- `@the9ines/bolt-transport-web` peerDependency on bolt-core raised to `>=0.2.0`.
- bolt-core devDependency linked via `file:../bolt-core` for monorepo development.

### Files Changed
- `ts/bolt-core/src/identity.ts` (new)
- `ts/bolt-core/src/index.ts`
- `ts/bolt-core/dist/identity.js` (new)
- `ts/bolt-core/dist/identity.d.ts` (new)
- `ts/bolt-core/dist/index.js`
- `ts/bolt-core/dist/index.d.ts`
- `ts/bolt-core/dist/sas.js`
- `ts/bolt-core/package.json`
- `ts/bolt-core/scripts/export-snapshot.json`
- `ts/bolt-core/__tests__/identity.test.ts` (new)
- `ts/bolt-transport-web/src/services/identity/identity-store.ts` (new)
- `ts/bolt-transport-web/src/services/identity/pin-store.ts` (new)
- `ts/bolt-transport-web/src/services/webrtc/WebRTCService.ts`
- `ts/bolt-transport-web/src/index.ts`
- `ts/bolt-transport-web/src/__tests__/hello.test.ts` (new)
- `ts/bolt-transport-web/src/__tests__/identity-store.test.ts` (new)
- `ts/bolt-transport-web/src/__tests__/pin-store.test.ts` (new)
- `ts/bolt-transport-web/package.json`
- `ts/bolt-transport-web/package-lock.json`

**Commit:** `192424b`

## [transport-web-v0.1.1-security-hardening] - 2026-02-23

### Fixed
- **S7: Ephemeral key lifecycle** — `WebRTCService` no longer generates keys
  at construction. Keys are generated per session in `connect()` / `handleOffer()`,
  and zeroed + discarded in `disconnect()`. Prevents key reuse across sessions.
- **S6: Filename XSS** — `transfer-progress.ts` and `toast.ts` now pass
  user-controlled strings through `escapeHTML()` before insertion into innerHTML.
  Guards against reflected XSS via crafted filenames.
- Null-guard on `this.keyPair` in `sendFile()` and `receiveChunk()` —
  throws `EncryptionError` if key material is missing.

### Added
- `src/__tests__/security.test.ts` — 17 tests covering S7 ephemeral key
  lifecycle (7 tests) and S6 escapeHTML / showToast XSS safety (10 tests).
  Uses vitest + jsdom.
- `vitest.config.ts` — test configuration (jsdom environment).
- `package.json` test script (`vitest run`), jsdom and vitest devDependencies.
- `tsconfig.build.json` excludes `src/__tests__` from production build.

### Changed
- `@the9ines/bolt-transport-web` version bumped from `0.1.0` to `0.1.1`.

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
