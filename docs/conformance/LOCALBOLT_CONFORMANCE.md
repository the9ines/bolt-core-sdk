# LocalBolt Conformance Mapping

Maps implemented security behaviors to Bolt Protocol spec sections and test evidence.

## Methodology

Each behavior was traced from its normative MUST/SHOULD requirement in PROTOCOL.md (Bolt Core v1) and LOCALBOLT_PROFILE.md (LocalBolt Profile v1) to the corresponding implementation test(s). Spec section numbers, exact normative language, test file paths, and individual test case names are recorded. Status is CONFORMANT when all normative requirements for the behavior are exercised by at least one passing test.

---

## Conformance Matrix

### 1. HELLO Gating (S4)

- **Spec Reference:** PROTOCOL.md Section 3 "Handshake Completion Rule" -- "Before handshake completion, a peer MUST accept only: `PING` and `PONG` (plaintext), encrypted envelopes containing `HELLO`, encrypted envelopes containing `ERROR`. All other messages MUST be rejected with `ENVELOPE(ERROR(INVALID_STATE))`."
- **Spec Reference:** PROTOCOL.md Section 10 "Error Taxonomy" -- "`INVALID_STATE`: Message received in wrong state (e.g. before handshake)"
- **Spec Reference:** PROTOCOL.md Appendix C item 1 -- "Handshake gating: reject `FILE_OFFER` before HELLO completion -> `ERROR(INVALID_STATE)`"
- **Profile Reference:** LOCALBOLT_PROFILE.md Section 6 "HELLO Exchange" -- "Receivers MUST reject any non-HELLO message before handshake completion with `ERROR(INVALID_STATE)` and SHOULD close the connection (fail-closed)"
- **Test File:** `ts/bolt-transport-web/src/__tests__/webrtcservice-handshake-gating.test.ts`
- **Test Cases:**
  - `allows HELLO message before helloComplete without INVALID_STATE`
  - `rejects file-chunk data before helloComplete with INVALID_STATE + disconnect`
  - `rejects file-chunk cancel control message before helloComplete`
  - `rejects file-chunk pause control message before helloComplete`
  - `after helloComplete, file-chunk messages route normally`
  - `sendFile still blocks until helloComplete (send-side gating preserved)`
- **Status:** CONFORMANT

---

### 2. TOFU Fail-Closed (S1)

- **Spec Reference:** PROTOCOL.md Section 2 "TOFU Flow" -- "Key mismatch (pinned key differs from received key): implementation MUST send `ENVELOPE(ERROR(KEY_MISMATCH))` and close the session; implementation MUST present a clear warning to the user"
- **Spec Reference:** PROTOCOL.md Section 13 "Conformance" -- "MUST: send `ERROR(KEY_MISMATCH)` and close session on key mismatch"; "MUST: verify remote identity matches pinned key when previously seen"
- **Spec Reference:** PROTOCOL.md Appendix C item 2 -- "Key mismatch: pinned key differs -> `ERROR(KEY_MISMATCH)` then close"
- **Profile Reference:** LOCALBOLT_PROFILE.md Section 6 "HELLO Exchange" -- HELLO carries `identityKey` (persistent X25519 public key) used for TOFU verification after decryption.
- **Test File:** `ts/bolt-transport-web/src/__tests__/pin-store.test.ts`
- **Test Cases:**
  - `pins on first contact and returns outcome "pinned"`
  - `returns outcome "verified" when pinned key matches`
  - `throws KeyMismatchError when pinned key differs`
  - `KeyMismatchError contains expected and received keys`
- **Test File:** `ts/bolt-transport-web/src/__tests__/hello.test.ts`
- **Test Cases:**
  - `pins identity on first HELLO from a peer`
  - `verifies pinned identity on subsequent HELLO`
  - `rejects mismatched identity (fail-closed)`
- **Status:** CONFORMANT

---

### 3. SAS Golden Vector (S2)

- **Spec Reference:** PROTOCOL.md Section 3 "SAS Verification" -- "`SAS_input = SHA-256( sort32(identity_A, identity_B) || sort32(ephemeral_A, ephemeral_B) )`; Display first 6 hex chars uppercase; SAS MUST be computed over raw 32-byte keys, not encoded representations"
- **Spec Reference:** PROTOCOL.md Section 14 "Constants" -- "`SAS_LENGTH`: 6 hex characters (uppercase); `SAS_ENTROPY`: 24 bits"
- **Spec Reference:** PROTOCOL.md Appendix C item 5 -- "SAS vectors: raw identity keys + envelope-header ephemeral keys -> expected 6 hex chars"
- **Profile Reference:** N/A (SAS computation is Core-level, not Profile-specific)
- **Test File:** `ts/bolt-core/__tests__/sas.test.ts`
- **Test Cases:**
  - `returns a 6-character uppercase hex string`
  - `is deterministic for same inputs`
  - `is symmetric (A,B == B,A)`
  - `produces different SAS for different keys`
  - `rejects keys with wrong length`
  - `golden vector -- fixed keys produce known SAS "65434F"`
  - `cross-side equality -- swapping A/B roles matches golden vector`
  - `sensitivity -- changing one byte in any input changes SAS`
- **Status:** CONFORMANT

---

### 4. Replay Protection Dedup/Bounds (S3)

- **Spec Reference:** PROTOCOL.md Section 8 "Replay Protection" -- "Receiver MUST reject duplicate `chunk_index` for the same `transfer_id`"; "Receiver MUST reject `chunk_index >= total_chunks`"
- **Spec Reference:** PROTOCOL.md Section 13 "Conformance" -- "MUST: reject duplicate chunk indices (scoped per `(transfer_id, chunk_index)`)"; "MUST: reject `chunk_index >= total_chunks`"
- **Spec Reference:** PROTOCOL.md Appendix C item 3 -- "Replay: duplicate `(transfer_id, chunk_index)` -> `ERROR(REPLAY_DETECTED)`"
- **Profile Reference:** LOCALBOLT_PROFILE.md Section 12 "Replay Protection" -- "Receiver Guards (Guarded Mode)" table: `chunkIndex` already received -> "Ignore duplicate, log `[REPLAY_DUP]`"; `chunkIndex < 0` or `chunkIndex >= totalChunks` -> "Reject, log `[REPLAY_OOB]`"; Same `transferId` bound to different sender identity key -> "Ignore, log `[REPLAY_XFER_MISMATCH]`"
- **Test File:** `ts/bolt-transport-web/src/__tests__/replay-protection.test.ts`
- **Test Cases (Guarded mode):**
  - `rejects duplicate chunkIndex with [REPLAY_DUP] warning`
  - `rejects out-of-range chunkIndex with [REPLAY_OOB] warning`
  - `rejects same transferId from different sender identity with [REPLAY_XFER_MISMATCH]`
  - `creates new transfer for different transferId (no mismatch log)`
  - `reconstructs file correctly from out-of-order delivery`
  - `resets state after completion and accepts new transferId`
- **Test Cases (Sender):**
  - `sendFile includes transferId (32 hex chars, constant across all chunks)`
- **Status:** CONFORMANT

---

### 5. transferId Optional Legacy Path

- **Spec Reference:** PROTOCOL.md Section 7 "Message Model" -- "`transfer_id`: bytes16, random via CSPRNG" (required field on FILE_CHUNK); PROTOCOL.md Section 8 "Replay Protection" -- "Replay detection is scoped per `(transfer_id, chunk_index)`"
- **Profile Reference:** LOCALBOLT_PROFILE.md Section 12 "Backward Compatibility (Legacy Mode)" -- "`transferId` is OPTIONAL at the wire level for backward compatibility; When absent, receiver operates in legacy mode (no dedup, bounds checks still applied); Legacy mode logs: `[REPLAY_UNGUARDED] chunk received without transferId`; Legacy chunks MUST NOT create or mutate guarded transfer state; Future versions MAY make `transferId` mandatory (fail-closed)"
- **Test File:** `ts/bolt-transport-web/src/__tests__/replay-protection.test.ts`
- **Test Cases (Legacy mode):**
  - `accepts chunks without transferId and completes transfer`
  - `emits [REPLAY_UNGUARDED] deprecation warning per chunk`
- **Test File:** `ts/bolt-transport-web/src/__tests__/webrtcservice-lifecycle.test.ts`
- **Test Cases:**
  - `legacy receiver path (no transferId) reconstructs file and calls onReceiveFile`
- **Status:** CONFORMANT

---

## Summary

| # | Behavior | Spec Sections | Test Count | Status |
|---|----------|---------------|------------|--------|
| 1 | HELLO Gating (S4) | PROTOCOL.md Section 3, Section 10, Appendix C.1; LOCALBOLT_PROFILE.md Section 6 | 6 | CONFORMANT |
| 2 | TOFU Fail-Closed (S1) | PROTOCOL.md Section 2, Section 13, Appendix C.2; LOCALBOLT_PROFILE.md Section 6 | 7 | CONFORMANT |
| 3 | SAS Golden Vector (S2) | PROTOCOL.md Section 3, Section 14, Appendix C.5 | 8 | CONFORMANT |
| 4 | Replay Protection (S3) | PROTOCOL.md Section 8, Section 13, Appendix C.3; LOCALBOLT_PROFILE.md Section 12 | 7 | CONFORMANT |
| 5 | transferId Legacy Path | PROTOCOL.md Section 7, Section 8; LOCALBOLT_PROFILE.md Section 12 | 3 | CONFORMANT |

All 5 mapped behaviors are CONFORMANT with their respective spec requirements. Total: 31 test cases providing evidence.
