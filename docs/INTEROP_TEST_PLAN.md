# Bolt Protocol Interop Test Plan

Defines the interoperability test matrix, scenarios, and success criteria for cross-transport Bolt implementations.

## 1. Transport Combinations

### Matrix

| Peer A | Peer B | Status | Priority |
|--------|--------|--------|----------|
| Browser WebRTC | Browser WebRTC | Production (tested daily) | — |
| Browser WebRTC | libdatachannel | Planned | P0 |
| Browser WebRTC | webrtc-rs | Future | P1 |
| libdatachannel | libdatachannel | Planned | P0 |
| libdatachannel | webrtc-rs | Future | P2 |
| webrtc-rs | webrtc-rs | Future | P2 |

### Per-combination test scope

Each combination must pass:
1. Deterministic vector verification (offline, no transport needed)
2. Live handshake + transfer scenario
3. Corruption rejection scenario

## 2. Deterministic Vector Verification

**No transport required.** This tests protocol-layer correctness only.

### Procedure

For each implementation under test:

1. Load `__tests__/vectors/box-payload.vectors.json`.
2. For each entry in `vectors`:
   - Call `openBoxPayload(sealed_base64, sender_public_key, receiver_secret_key)`.
   - Verify plaintext matches `plaintext_hex`.
3. For each entry in `corrupt_vectors`:
   - Call `openBoxPayload(...)` with the specified keys.
   - Verify decryption fails with the expected error.
4. Load `__tests__/vectors/framing.vectors.json`.
5. For each entry in `vectors`:
   - Base64-decode `sealed_base64`.
   - Verify first 24 bytes match `expected_nonce_hex`.
   - Verify remaining bytes length equals `expected_ciphertext_length`.

### Success criteria

- All valid vectors decrypt to expected plaintext.
- All corrupt vectors are rejected.
- All framing assertions pass.

### Cross-language vector test

Run `scripts/print-test-vectors.mjs` and pipe the output to the Rust/C++ test harness for comparison. The canonical JSON format is the interchange format.

## 3. Live Interop Scenario

### Deterministic message exchange

```
Step  Direction     Message Type        Payload
────  ─────────     ────────────        ───────
1     A → B         HELLO               identity_key_A (encrypted)
2     B → A         HELLO               identity_key_B (encrypted)
3     (both)        SAS verify          display + user confirmation
4     A → B         file-chunk          chunk_1 (1024 bytes, pattern: 0x00..0xFF repeating)
5     A → B         file-chunk          chunk_2 (1024 bytes, pattern: 0xFF..0x00 repeating)
6     A → B         file-chunk          chunk_3 (512 bytes, pattern: 0xAA repeating)
7     A → B         file-chunk          corrupted_chunk (valid sealed payload with flipped last byte)
8     B             (verify)            decrypt chunk_1..3, reject corrupted_chunk
9     A → B         close               graceful close signal
10    B → A         close               graceful close acknowledgment
```

### Transcript logging format

Each implementation MUST log interop test events in this format:

```
[BOLT_INTEROP] <timestamp_iso8601> <direction> <step> <event> <detail>
```

Fields:

| Field | Format | Example |
|-------|--------|---------|
| timestamp | ISO 8601 with milliseconds | `2026-02-21T13:00:00.000Z` |
| direction | `SEND` or `RECV` or `LOCAL` | `SEND` |
| step | Step number from scenario | `4` |
| event | Event type | `SEALED_PAYLOAD` |
| detail | JSON object with relevant data | `{"length":1064,"chunk_index":0}` |

Event types:

| Event | When |
|-------|------|
| `HELLO_SENT` | After sealing and sending HELLO |
| `HELLO_RECEIVED` | After receiving and opening HELLO |
| `SAS_COMPUTED` | After SAS computation |
| `SAS_VERIFIED` | After user/automated SAS confirmation |
| `SEALED_PAYLOAD` | After sealing a chunk for send |
| `OPENED_PAYLOAD` | After successfully opening a received chunk |
| `DECRYPT_REJECTED` | After a decryption failure (expected or unexpected) |
| `CLOSE_SENT` | After sending close signal |
| `CLOSE_RECEIVED` | After receiving close signal |
| `ICE_CONNECTED` | Transport-level ICE connection established |
| `DTLS_COMPLETE` | DTLS handshake finished |
| `DATACHANNEL_OPEN` | DataChannel is open |

### Success criteria for live scenario

1. Both peers compute identical SAS.
2. Chunks 1-3 decrypt to expected plaintext on peer B.
3. Corrupted chunk is rejected with `EncryptionError`.
4. Graceful close completes on both sides.
5. No unhandled errors in either peer's transcript log.

## 4. Failure Taxonomy

### ICE failure

| Symptom | Likely cause | Debugging |
|---------|-------------|-----------|
| No candidates gathered | Firewall, no network interface | Check `onicecandidate` events |
| Candidates gathered but no pair selected | NAT traversal failure, no common address | Check ICE candidate types (host/srflx/relay) |
| Connection timeout | Asymmetric firewall, TURN unavailable | Check ICE connection state transitions |

**Protocol impact**: None. ICE failures occur before any Bolt messages are exchanged.

### DTLS failure

| Symptom | Likely cause | Debugging |
|---------|-------------|-----------|
| DTLS handshake timeout | Incompatible cipher suites | Check DTLS version and cipher negotiation |
| DTLS alert | Certificate validation mismatch | Check DTLS fingerprint in SDP |

**Protocol impact**: None. DTLS is transport-layer. Bolt does not depend on DTLS for security.

### Decrypt failure

| Symptom | Likely cause | Debugging |
|---------|-------------|-----------|
| `EncryptionError('Decryption failed')` on valid payload | Key mismatch (wrong ephemeral keypair) | Log public keys on both sides, verify they match |
| `EncryptionError('Decryption failed')` on first message | Byte ordering issue in nonce||ciphertext split | Verify wire format: first 24 bytes = nonce |
| All payloads fail | Base64 encoding mismatch (URL-safe vs standard) | Verify standard base64 with padding |

### Framing error

| Symptom | Likely cause | Debugging |
|---------|-------------|-----------|
| Partial message received | Transport not in reliable/ordered mode | Check DataChannel configuration |
| Messages coalesced | Application-layer framing bug | Verify one sealed payload = one transport message |
| Payload too large | Chunk size exceeds transport max message size | Reduce chunk size or verify SCTP config |

## 5. Manual Test Procedure (Runnable Today)

This procedure tests Browser WebRTC ↔ Browser WebRTC using existing product deployments.

### Prerequisites

- Two browser windows (same machine or same LAN).
- localbolt or localbolt-v3 running in both windows.
- Browser DevTools open (Console tab).

### Steps

1. **Window A**: Generate a peer code. Note it.
2. **Window B**: Enter the peer code from Window A. Connect.
3. **Verify SAS**: Both windows display the same 6-character hex code. Confirm.
4. **Transfer small file**: From Window A, select a small text file (< 1 KB). Send.
5. **Verify receipt**: Window B receives the file. Open it and verify contents match.
6. **Transfer larger file**: From Window A, select a file > 64 KB (forces multiple chunks). Send.
7. **Verify receipt**: Window B receives the file. Compare SHA-256 hash of original and received.
8. **Verify cleanup**: Close the connection. Verify no errors in console.

### Expected console observations

- `[WebRTC]` connection state transitions: `new` → `connecting` → `connected`.
- DataChannel open event.
- No `EncryptionError` entries.
- File hash match (if `bolt.file-hash` capability is negotiated).

### What this does NOT test

- Cross-transport interop (requires libdatachannel or webrtc-rs peer).
- Corrupted payload handling (requires injecting bad data, not available in standard UI).
- LAN-only ICE filtering (requires network inspection, not console observation).

## 6. Automated Interop Test (Future)

When Rust SDK is available:

1. TypeScript peer runs in Node.js using `@the9ines/bolt-core` + a headless WebRTC library.
2. Rust peer runs using bolt-core Rust SDK + libdatachannel (or webrtc-rs).
3. Both connect via a local bolt-rendezvous instance.
4. Execute the deterministic message exchange scenario (§3).
5. Compare transcript logs for consistency.
6. Report pass/fail per matrix entry.

CI integration: Run as a separate workflow triggered on SDK releases. Gate on all P0 matrix entries passing.
