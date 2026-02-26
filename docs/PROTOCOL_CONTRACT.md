# Bolt Protocol Contract — SDK Conformance Clarifications

This document provides conformance clarifications and implementation notes for the `@the9ines/bolt-core` SDK. It is **not** a second specification.

**Normative authority**: `PROTOCOL.md` in `bolt-protocol` is the single source of truth for wire formats, byte layouts, and protocol semantics. This document clarifies how the TypeScript SDK implements those requirements and defines the test vector contract for cross-implementation verification.

If this document and `PROTOCOL.md` conflict, `PROTOCOL.md` wins.

Protocol version: **1** (`BOLT_VERSION`).

## 0. Authoritative Specification

- Wire-level formats, byte layouts, message schemas, and handshake rules are normative **only** in `PROTOCOL.md`.
- This document defines SDK implementation invariants, error contract clarifications, and test vector validation rules.
- If any statement in this document conflicts with `PROTOCOL.md`, `PROTOCOL.md` is authoritative and this document must be corrected.
- This document MUST NOT duplicate wire schema tables from `PROTOCOL.md`. Implementation-facing summaries reference the spec without replacing it.

## 1. Box Payload Format

Bolt uses NaCl `crypto_box` (X25519 key agreement + XSalsa20-Poly1305 AEAD).

### Wire format

```
sealed_payload = base64( nonce || ciphertext )
```

| Field | Length | Description |
|-------|--------|-------------|
| `nonce` | 24 bytes | Cryptographically random. Unique per message. |
| `ciphertext` | `plaintext_length + 16` bytes | NaCl box output (encrypted payload + Poly1305 MAC). |

Total raw bytes before base64 encoding: `24 + plaintext_length + 16`.

### Base64 encoding

- **RFC 4648 standard base64** (alphabet `A-Za-z0-9+/`).
- Padding with `=` is **required**.
- **Not base64url.** The `+` and `/` characters are used, not `-` and `_`. If a future transport requires URL-safe encoding, that decision must be made explicitly in `PROTOCOL.md` and will require a version bump.
- No line breaks or whitespace within the encoded string.
- Implementation: `tweetnacl-util.encodeBase64` / `decodeBase64` (TypeScript SDK), which delegates to `Buffer.toString('base64')` in Node.js and `btoa()`/`atob()` in browsers.

### Seal operation

```
nonce      = random_bytes(24)
ciphertext = nacl.box(plaintext, nonce, receiver_public_key, sender_secret_key)
output     = base64(nonce || ciphertext)
```

### Open operation

```
raw        = base64_decode(sealed_payload)
nonce      = raw[0..24]
ciphertext = raw[24..]
plaintext  = nacl.box.open(ciphertext, nonce, sender_public_key, receiver_secret_key)
```

If `nacl.box.open` returns null/failure, the implementation MUST throw `EncryptionError('Decryption failed')`.

## 2. Key Formats

### Ephemeral keypair

| Property | Value |
|----------|-------|
| Algorithm | X25519 (Curve25519 Diffie-Hellman) |
| Public key length | 32 bytes |
| Secret key length | 32 bytes |
| Encoding (wire) | Raw bytes or base64 depending on context |
| Lifecycle | Per-connection. Generated fresh. Discarded on disconnect. |

### Identity keypair

Same algorithm and lengths. Persistent across connections. TOFU-pinned.

### Constants

| Name | Value | Unit |
|------|-------|------|
| `NONCE_LENGTH` | 24 | bytes |
| `PUBLIC_KEY_LENGTH` | 32 | bytes |
| `SECRET_KEY_LENGTH` | 32 | bytes |

These are protocol invariants. They MUST NOT change.

## 3. Chunk Framing

### DEFAULT_CHUNK_SIZE

| Name | Value | Unit |
|------|-------|------|
| `DEFAULT_CHUNK_SIZE` | 16384 | bytes (16 KB) |

**Protocol vs implementation:**

- `DEFAULT_CHUNK_SIZE` is an **implementation default**, not a protocol invariant.
- The protocol requires that plaintext is split into chunks before encryption. Each chunk is sealed individually.
- The chunk size MAY vary between implementations as long as both peers agree (via capability negotiation or profile specification).
- 16 KB is the canonical default chosen for WebRTC DataChannel compatibility (SCTP max message size considerations).

### Ordering and reliability

The protocol assumes the transport delivers chunks **in order** and **reliably**. See `TRANSPORT_CONTRACT.md` for transport requirements.

Chunk sequence:
1. Sender splits plaintext file into chunks of at most `chunk_size` bytes.
2. Each chunk is sealed individually with `sealBoxPayload`.
3. Chunks are sent sequentially over the transport.
4. Receiver opens each chunk with `openBoxPayload` and reassembles.

The last chunk MAY be smaller than `chunk_size`.

## 4. Error Contract

### Error hierarchy

```
BoltError (base)
├── EncryptionError   — crypto failures
├── ConnectionError   — transport/connection failures
└── TransferError     — file transfer failures
```

### Required error conditions

| Condition | Error type | Message |
|-----------|-----------|---------|
| `nacl.box.open` returns null | `EncryptionError` | `'Decryption failed'` |
| `nacl.box` returns null | `EncryptionError` | `'Encryption returned null'` |
| Malformed base64 input | Runtime error from base64 decoder | (implementation-defined) |
| Wrong key (decrypt with incorrect keypair) | `EncryptionError` | `'Decryption failed'` |
| Truncated ciphertext | `EncryptionError` | `'Decryption failed'` |
| Modified ciphertext (bit flip) | `EncryptionError` | `'Decryption failed'` |

All NaCl authentication failures (wrong key, truncated, modified) produce the same error. This is by design — NaCl does not distinguish between these cases.

### Invariant violations

If an implementation detects a protocol invariant violation (e.g., nonce length != 24, key length != 32), it SHOULD throw `BoltError` with a descriptive message. The specific behavior is implementation-defined.

## 5. SAS (Short Authentication String)

### Computation

```
sorted_identity  = sort32(identity_key_A, identity_key_B)
sorted_ephemeral = sort32(ephemeral_key_A, ephemeral_key_B)
sas_input        = sorted_identity || sorted_ephemeral
hash             = SHA-256(sas_input)
sas              = uppercase(hex(hash[0..3]))   // first 3 bytes = 6 hex chars
```

`sort32` compares two 32-byte arrays lexicographically and returns `(smaller || larger)`. If equal, concatenation order is arbitrary (identical keys are a degenerate case).

### Properties

| Property | Value |
|----------|-------|
| Length | 6 characters |
| Alphabet | `0-9A-F` (uppercase hex) |
| Entropy | 24 bits |
| Symmetry | Both peers compute the same SAS regardless of role |
| Binding | Binds identity keys AND ephemeral keys |

### Inputs

All four inputs MUST be 32 bytes. Implementations MUST reject inputs of other lengths.

## 6. Peer Codes

### Format

| Type | Length | Alphabet | Entropy |
|------|--------|----------|---------|
| Short | 6 characters | `ABCDEFGHJKMNPQRSTUVWXYZ23456789` (30 chars) | ~29.4 bits |
| Long | 8 characters (displayed as `XXXX-XXXX`) | Same | ~39.2 bits |

The alphabet excludes `0/O`, `1/I/L` to avoid visual ambiguity.

### Operations

- **Generate**: Cryptographically random selection from alphabet.
- **Validate**: Check length (6 or 8 after removing dashes) and alphabet membership.
- **Normalize**: Remove dashes, uppercase.

Peer codes are a product-layer concept for room identification. They are NOT part of the cryptographic protocol.

## 7. Hash Functions

| Function | Algorithm | Output |
|----------|-----------|--------|
| `sha256` | SHA-256 (Web Crypto API) | `ArrayBuffer` (32 bytes) |
| `bufferToHex` | — | Lowercase hex string |
| `hashFile` | SHA-256 | Lowercase hex string of file contents |

SHA-256 is used for SAS computation and file integrity verification. It is NOT used in the encryption path (NaCl handles its own MAC).

## 8. Backward Compatibility Rules

### Pre-1.0 (current: 0.y.z)

| Change type | Version bump | Allowed? |
|-------------|-------------|----------|
| Bug fix, no API change | Patch (z) | Yes |
| New export added | Minor (y) | Yes |
| Export removed or renamed | Minor (y) | Yes, with same-cycle product upgrade |
| Wire format change | Minor (y) | Yes, but requires all implementations to update simultaneously |
| Constant value change (`NONCE_LENGTH`, `PUBLIC_KEY_LENGTH`, `SECRET_KEY_LENGTH`) | **NEVER** | No — these are cryptographic invariants |

### Post-1.0 (future: x.y.z)

| Change type | Version bump |
|-------------|-------------|
| Bug fix | Patch (z) |
| New export (backward-compatible) | Minor (y) |
| Wire format change | Major (x) |
| Export removal | Major (x) |
| Constant value change | **NEVER** |

### Transport-agnostic guarantee

The following are entirely transport-agnostic and MUST NOT reference any specific transport:

- Box payload seal/open
- Key generation
- SAS computation
- Error types
- Constants

Transport-specific behavior (ICE policy, SCTP parameters, DataChannel configuration) belongs in `TRANSPORT_CONTRACT.md` and in product-layer code.

## 9. Test Vectors

Deterministic test vectors are provided in:

- `__tests__/vectors/box-payload.vectors.json` — seal/open verification + corrupt cases
- `__tests__/vectors/framing.vectors.json` — wire format layout verification

These vectors use fixed keypairs and fixed nonces for cross-implementation reproducibility. See `scripts/print-test-vectors.mjs` for generation.

```
╔══════════════════════════════════════════════════════════════════╗
║  TEST FIXTURES ONLY — NEVER USE IN PRODUCTION                  ║
║                                                                ║
║  All keypairs in the vector files are deterministic, publicly   ║
║  known test fixtures. The fixed secret keys (bytes [1..32],     ║
║  [33..64], [65..96]) are NOT valid cryptographic material for   ║
║  real deployments. Using them in production is a fatal security ║
║  error.                                                        ║
╚══════════════════════════════════════════════════════════════════╝
```

### CI enforcement

Committed vectors are verified against the generation script in CI via `npm run check-vectors`. If the committed JSON files do not match regenerated output, CI fails.

### Cross-implementation conformance

Any compliant implementation (Rust SDK, libdatachannel peer, webrtc-rs peer) MUST produce identical sealed output given the same inputs and MUST successfully open all valid vectors.
