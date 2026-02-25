# Bolt Ecosystem Audit Tracker

**Last updated:** 2026-02-24
**Scope:** All repos under the9ines/bolt-ecosystem
**Authority:** bolt-core-sdk (this repo)

---

## SECURITY

| ID | Finding | Severity | Status | Evidence |
|----|---------|----------|--------|----------|
| S1 | TOFU identity pinning missing | HIGH | **DONE** | `bolt-core` identity primitives + `bolt-transport-web` pin store. Phase 7A: `sdk-v0.2.0-identity-primitives`, `transport-web-v0.2.0-hello-tofu-foundation`. 21 tests. |
| S2 | SAS verification not surfaced | HIGH | **DONE** | `bolt-core` canonical `computeSas()` + `bolt-transport-web` verification UI. Phase 7B: `transport-web-v0.3.0-sas-verification`. 15 tests + 3 golden vectors. |
| S3 | No replay protection / chunk dedup | HIGH | **DONE** | `transferId` (bytes16), per-transfer dedup, bounds checks, sender-identity binding. Phase 8A: `transport-web-v0.4.0-replay-protection`. 9 tests. Legacy path with `[REPLAY_UNGUARDED]` warning. |
| S4 | Pre-handshake message acceptance | MEDIUM | **DONE** | Fail-closed `INVALID_STATE` + disconnect for any non-HELLO message before `helloComplete`. Phase 8D: `transport-web-v0.4.2-strict-handshake-gating`. 6 tests. |
| S5 | Peer code modulo bias | LOW | **DONE** | Rejection sampling (`byte >= 248` discarded). Phase 8E: `sdk-v0.2.1-peer-code-bias-fix`. 3 tests. |
| S6 | Filename XSS via innerHTML | MEDIUM | **DONE** | `escapeHTML()` on all user-controlled strings. Phase 6B: `transport-web-v0.1.1-security-hardening`. 10 tests. |
| S7 | Ephemeral key reuse across sessions | HIGH | **DONE** | Keys generated per session in `connect()`/`handleOffer()`, zeroed in `disconnect()`. Phase 6B: `transport-web-v0.1.1-security-hardening`. 7 tests. |

---

## INTEROP

| ID | Finding | Severity | Status | Evidence |
|----|---------|----------|--------|----------|
| I1 | Rust/TS constants misaligned | MEDIUM | **DONE** | PEER_CODE_LENGTH 4->6, SAS_LENGTH 4->6, alphabet 36->31 chars. Phase 6A.1: `sdk-v0.1.1-constants-alignment`. Cross-language verification script. |
| I2 | Daemon/Web NaCl interop untested | LOW | **DEFERRED** | bolt-daemon is planned/minimal. No TS consumers exist. Will be addressed when daemon development begins. |
| I3 | Shadow SAS in transport-web | MEDIUM | **DONE** | Removed `getVerificationCode()`. Phase 6A.2: `sdk-v0.1.2-sas-canonical`. Enforcement script `verify-no-shadow-sas.sh`. |
| I4 | Protocol-level bolt-envelope | MEDIUM | **DEFERRED** | Profile Envelope v1 landed (Phase M1, `transport-web-v0.6.0`). Full protocol-level envelope standardization across all transports is a large cross-cutting effort deferred to bolt-protocol specification work. |

---

## QUALITY

| ID | Finding | Severity | Status | Evidence |
|----|---------|----------|--------|----------|
| Q1 | bolt-core test coverage baseline | MEDIUM | **DONE** | 76 tests across 7 files. Golden vectors, export snapshot guard, rejection sampling, identity, SAS, hash, crypto. |
| Q2 | bolt-transport-web test coverage | MEDIUM | **DONE** | 117 tests across 11 files. Hello, TOFU, SAS, replay, handshake gating, lifecycle, capabilities, file-hash, envelope, security. |
| Q3 | localbolt test coverage | MEDIUM | **DONE** | 272 tests across 13 files. Coverage thresholds enforced (80/70/80). |
| Q4 | localbolt-app test coverage | LOW | **DEFERRED** | No test suite. Build-only gate. Will be addressed when app matures past scaffold. |
| Q5 | localbolt-v3 test pipeline | MEDIUM | **DONE** | 4 smoke tests (FAQ + app render). Phase TP: `v3.0.53-test-pipeline`. CI step before build. |
| Q6 | localbolt-v3 coverage thresholds | MEDIUM | **DONE** | `@vitest/coverage-v8`, thresholds 45/5/31/48%. Phase Q6: `v3.0.55-coverage-thresholds`. |

---

## ARCHITECTURE

| ID | Finding | Severity | Status | Evidence |
|----|---------|----------|--------|----------|
| A1 | Dead exports in bolt-core public API | LOW | **DONE** | 7 unused constants removed (28->21 exports). Version 0.3.0->0.4.0. All consumers adopted. Phase A1: `sdk-v0.4.0-dead-exports-cleanup`. Consumers: localbolt `91a0f29`, localbolt-app `90584bf`, localbolt-v3 `14927d7`. |
| A2 | Signaling type duplication across repos | MEDIUM | **DONE** | `bolt-rendezvous-protocol` shared crate extracted. bolt-rendezvous and bolt-daemon both consume canonical types. |
| A3 | localbolt-v3 unmanaged signaling subtree | LOW | **DONE** | ADR-0001 documents native workspace crate decision + drift control policy. Phase A3: `v3.0.56-signaling-adr`. Not a subtree by design. |

---

## MEDIUM-TERM FEATURES (delivered)

| ID | Feature | Status | Evidence |
|----|---------|--------|----------|
| M1 | Profile Envelope v1 | **DONE** | `bolt.envelope` capability, versioned metadata wrapping, mixed-peer backward compat. `sdk-v0.3.0-profile-envelope-v1`, `transport-web-v0.6.0`. 14 tests. |
| M2 | File integrity hash wiring | **DONE** | `bolt.file-hash` capability, SHA-256 sender+receiver, fail-closed on mismatch. `sdk-v0.2.2-file-hash-wiring`, `transport-web-v0.5.0`. 16 tests. |
| M3 | SDK publish + consumer adoption | **DONE** | `@the9ines/bolt-core@0.4.0` and `@the9ines/bolt-transport-web@0.6.0` on GitHub Packages. All 3 consumers pinned on main. |

---

## ADOPTION STATUS

All product repos on main are pinned to current SDK releases:

| Repo | bolt-core | transport-web | Tests | Build |
|------|-----------|---------------|-------|-------|
| localbolt | 0.4.0 | 0.6.0 | 272/272 | pass |
| localbolt-app | 0.4.0 | 0.6.0 | N/A | pass |
| localbolt-v3 | 0.4.0 | 0.6.0 | 4/4 | pass |

---

## SUMMARY

- **Total findings:** 26
- **DONE:** 22
- **DEFERRED:** 3 (I2, I4, Q4)
- **Residual risk:** See `docs/SECURITY_POSTURE.md`

The LocalBolt audit surface is effectively frozen and clean. All actionable items without entering ByteBolt/global scope are closed.

---

## PROTOCOL HARDENING (H-phases)

| ID | Phase | Description | Status | Evidence |
|----|-------|-------------|--------|----------|
| H0 | Protocol enforcement posture | Normative enforcement doc: exactly-once HELLO, envelope-required, fail-closed, error registry, downgrade resistance | **DONE** | `bolt-ecosystem/docs/PROTOCOL_ENFORCEMENT.md`. Not version-controlled (ecosystem root is not a git repo). |
| H1 | Signal server hardening | Trust-boundary hardening in localbolt-v3 signal server | **DONE** | `v3.0.59-signal-hardening` (`ac5110c`). On `feature/h1-signal-hardening`, not merged to main. |
| H2 | WebRTC enforcement compliance | Exactly-once HELLO, envelope-required, fail-closed in WebRTCService | **DONE** | `sdk-v0.5.0-h2-webrtc-enforcement` (`b4ce544`). 21 enforcement tests. On `feature/h3-golden-vectors`, not merged to main. |
| H3 | Cross-implementation golden vectors | SAS, HELLO-open, envelope-open deterministic vectors across TS, Rust SDK, daemon | **DONE** | `sdk-v0.5.1` (`9d8617d`), `daemon-v0.2.5-h3-golden-vectors` (`3751118`). On feature branches, not merged to main. |
| H4 | Daemon unwrap hardening | Error code enforcement in daemon decode paths | NOT STARTED | — |
| H5 | TOFU/SAS wiring in localbolt-v3 | Wire TOFU + SAS into product UI | NOT STARTED | — |
| H6 | CI/coverage enforcement | Golden vector and enforcement tests as CI gates | NOT STARTED | — |
