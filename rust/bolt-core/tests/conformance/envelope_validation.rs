//! Conformance: Envelope Roundtrip Determinism + MAC Enforcement + Nonce Sanity
//!
//! Invariants under test:
//! - PROTO-01: HELLO MUST be inside an encrypted envelope
//! - PROTO-07: All protected messages MUST be inside an encrypted envelope
//! - SEC-01: Every encrypted envelope MUST use a fresh 24-byte CSPRNG nonce
//! - SEC-02: Nonce MUST NOT be reused with the same ephemeral keypair
//! - SEC-06: MAC MUST be verified before any plaintext processing
//!
//! Uses H3 golden vectors from ts/bolt-core/__tests__/vectors/.
//! Does NOT import bolt-rendezvous-protocol or any cross-repo dependency.

use serde::Deserialize;
use std::path::PathBuf;

// ── Vector schemas ──────────────────────────────────────────────

#[derive(Deserialize)]
struct BoxPayloadVectors {
    sender: KeyEntry,
    receiver: KeyEntry,
    eve: KeyEntry,
    vectors: Vec<ValidVector>,
    corrupt_vectors: Vec<CorruptVector>,
}

#[derive(Deserialize)]
struct KeyEntry {
    #[serde(rename = "publicKey_hex")]
    public_key_hex: String,
    #[serde(rename = "secretKey_hex")]
    secret_key_hex: String,
}

#[derive(Deserialize)]
struct ValidVector {
    id: String,
    #[serde(default)]
    plaintext_hex: String,
    #[allow(dead_code)]
    nonce_hex: String,
    sealed_base64: String,
}

#[derive(Deserialize)]
struct CorruptVector {
    id: String,
    sealed_base64: String,
    #[allow(dead_code)]
    expected_error: String,
    #[serde(default)]
    use_eve_as_sender: bool,
}

#[derive(Deserialize)]
struct HelloVectors {
    cases: Vec<HelloCase>,
}

#[derive(Deserialize)]
struct HelloCase {
    name: String,
    sender_public_hex: String,
    receiver_secret_hex: String,
    sealed_payload_base64: String,
    expected_inner: serde_json::Value,
}

#[derive(Deserialize)]
struct EnvelopeVectors {
    cases: Vec<EnvelopeCase>,
}

#[derive(Deserialize)]
struct EnvelopeCase {
    name: String,
    sender_public_hex: String,
    receiver_secret_hex: String,
    envelope_json: EnvelopeFrame,
    expected_inner: serde_json::Value,
}

#[derive(Deserialize)]
struct EnvelopeFrame {
    payload: String,
}

// ── Helpers ─────────────────────────────────────────────────────

fn vectors_dir() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .join("..")
        .join("..")
        .join("ts")
        .join("bolt-core")
        .join("__tests__")
        .join("vectors")
}

fn hex_to_32(hex: &str) -> [u8; 32] {
    bolt_core::encoding::from_hex(hex)
        .expect("invalid hex")
        .try_into()
        .expect("expected 32 bytes")
}

fn load_box_payload_vectors() -> BoxPayloadVectors {
    let path = vectors_dir().join("box-payload.vectors.json");
    let data = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    serde_json::from_str(&data).expect("box-payload vectors parse failed")
}

fn load_hello_vectors() -> HelloVectors {
    let path = vectors_dir().join("web-hello-open.vectors.json");
    let data = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    serde_json::from_str(&data).expect("hello vectors parse failed")
}

fn load_envelope_vectors() -> EnvelopeVectors {
    let path = vectors_dir().join("envelope-open.vectors.json");
    let data = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    serde_json::from_str(&data).expect("envelope vectors parse failed")
}

// ── Conformance: Envelope Roundtrip Determinism ─────────────────

/// PROTO-07: seal → open roundtrip produces identical plaintext for
/// every H3 box-payload golden vector.
#[test]
fn conformance_box_payload_open_all_vectors() {
    let vecs = load_box_payload_vectors();
    let sender_pk = hex_to_32(&vecs.sender.public_key_hex);
    let receiver_sk = hex_to_32(&vecs.receiver.secret_key_hex);

    for v in &vecs.vectors {
        let plaintext = bolt_core::encoding::from_hex(&v.plaintext_hex).unwrap();
        let result =
            bolt_core::crypto::open_box_payload(&v.sealed_base64, &sender_pk, &receiver_sk)
                .unwrap_or_else(|e| panic!("open failed for vector '{}': {e}", v.id));
        assert_eq!(
            result, plaintext,
            "plaintext mismatch for vector '{}'",
            v.id
        );
    }
}

/// PROTO-01: HELLO vectors decrypt to expected inner JSON.
#[test]
fn conformance_hello_open_all_vectors() {
    let vecs = load_hello_vectors();

    for case in &vecs.cases {
        let sender_pk = hex_to_32(&case.sender_public_hex);
        let receiver_sk = hex_to_32(&case.receiver_secret_hex);

        let decrypted = bolt_core::crypto::open_box_payload(
            &case.sealed_payload_base64,
            &sender_pk,
            &receiver_sk,
        )
        .unwrap_or_else(|e| panic!("HELLO open failed for '{}': {e}", case.name));

        let inner: serde_json::Value = serde_json::from_slice(&decrypted)
            .unwrap_or_else(|e| panic!("HELLO JSON parse failed for '{}': {e}", case.name));

        assert_eq!(
            inner, case.expected_inner,
            "HELLO inner mismatch for '{}'",
            case.name
        );
    }
}

/// PROTO-07: Envelope vectors decrypt to expected inner JSON.
#[test]
fn conformance_envelope_open_all_vectors() {
    let vecs = load_envelope_vectors();

    for case in &vecs.cases {
        let sender_pk = hex_to_32(&case.sender_public_hex);
        let receiver_sk = hex_to_32(&case.receiver_secret_hex);

        let decrypted = bolt_core::crypto::open_box_payload(
            &case.envelope_json.payload,
            &sender_pk,
            &receiver_sk,
        )
        .unwrap_or_else(|e| panic!("envelope open failed for '{}': {e}", case.name));

        let inner: serde_json::Value = serde_json::from_slice(&decrypted)
            .unwrap_or_else(|e| panic!("envelope JSON parse failed for '{}': {e}", case.name));

        assert_eq!(
            inner, case.expected_inner,
            "envelope inner mismatch for '{}'",
            case.name
        );
    }
}

// ── Conformance: MAC Verification Enforcement ───────────────────

/// SEC-06: All corrupt vectors MUST be rejected — no partial open.
#[test]
fn conformance_corrupt_vectors_all_rejected() {
    let vecs = load_box_payload_vectors();
    let sender_pk = hex_to_32(&vecs.sender.public_key_hex);
    let eve_pk = hex_to_32(&vecs.eve.public_key_hex);
    let receiver_sk = hex_to_32(&vecs.receiver.secret_key_hex);

    for cv in &vecs.corrupt_vectors {
        let pk = if cv.use_eve_as_sender {
            &eve_pk
        } else {
            &sender_pk
        };
        let result = bolt_core::crypto::open_box_payload(&cv.sealed_base64, pk, &receiver_sk);
        assert!(
            result.is_err(),
            "corrupt vector '{}' should have been rejected but decrypted successfully",
            cv.id
        );
    }
}

/// SEC-06: Single-bit flip anywhere in ciphertext body MUST cause rejection.
#[test]
fn conformance_mac_rejects_single_bit_flip() {
    let alice = bolt_core::crypto::generate_ephemeral_keypair();
    let bob = bolt_core::crypto::generate_ephemeral_keypair();
    let plaintext = b"MAC integrity test payload";

    let sealed =
        bolt_core::crypto::seal_box_payload(plaintext, &bob.public_key, &alice.secret_key).unwrap();

    let raw = bolt_core::encoding::from_base64(&sealed).unwrap();

    // Flip one bit in the ciphertext portion (after the 24-byte nonce).
    // Try multiple positions to be thorough.
    let nonce_len = bolt_core::constants::NONCE_LENGTH;
    let ciphertext_len = raw.len() - nonce_len;
    assert!(ciphertext_len > 0, "ciphertext must be non-empty");

    let positions = [0, ciphertext_len / 2, ciphertext_len - 1];
    for &offset in &positions {
        let mut tampered = raw.clone();
        tampered[nonce_len + offset] ^= 0x01;
        let tampered_b64 = bolt_core::encoding::to_base64(&tampered);
        let result =
            bolt_core::crypto::open_box_payload(&tampered_b64, &alice.public_key, &bob.secret_key);
        assert!(
            result.is_err(),
            "bit flip at ciphertext offset {offset} was not rejected"
        );
    }
}

/// SEC-06: Nonce-only payload (no ciphertext) MUST be rejected.
#[test]
fn conformance_mac_rejects_nonce_only_payload() {
    let kp = bolt_core::crypto::generate_ephemeral_keypair();
    let nonce_only = bolt_core::encoding::to_base64(&[0u8; bolt_core::constants::NONCE_LENGTH]);
    let result = bolt_core::crypto::open_box_payload(&nonce_only, &kp.public_key, &kp.secret_key);
    assert!(result.is_err(), "nonce-only payload should be rejected");
}

/// SEC-06: Payload shorter than nonce length MUST be rejected.
#[test]
fn conformance_mac_rejects_short_payload() {
    let kp = bolt_core::crypto::generate_ephemeral_keypair();
    let short = bolt_core::encoding::to_base64(&[0u8; 10]);
    let result = bolt_core::crypto::open_box_payload(&short, &kp.public_key, &kp.secret_key);
    assert!(result.is_err(), "short payload should be rejected");
}

// ── Conformance: Nonce / Replay Sanity ──────────────────────────

/// SEC-01: Every sealed envelope contains a 24-byte nonce prefix.
#[test]
fn conformance_nonce_always_24_bytes() {
    let alice = bolt_core::crypto::generate_ephemeral_keypair();
    let bob = bolt_core::crypto::generate_ephemeral_keypair();

    let payloads: &[&[u8]] = &[b"", b"x", b"Hello, Bolt!", &[0xFFu8; 256]];

    for payload in payloads {
        let sealed =
            bolt_core::crypto::seal_box_payload(payload, &bob.public_key, &alice.secret_key)
                .unwrap();
        let raw = bolt_core::encoding::from_base64(&sealed).unwrap();
        assert!(
            raw.len() >= bolt_core::constants::NONCE_LENGTH,
            "sealed payload too short for nonce"
        );
        // Ciphertext = plaintext + BOX_OVERHEAD (16 bytes MAC)
        let expected_len =
            bolt_core::constants::NONCE_LENGTH + payload.len() + bolt_core::constants::BOX_OVERHEAD;
        assert_eq!(
            raw.len(),
            expected_len,
            "wire format length mismatch for {}-byte payload",
            payload.len()
        );
    }
}

/// SEC-01 + SEC-02: Multiple seals produce unique nonces (no reuse).
#[test]
fn conformance_nonce_no_reuse_256_seals() {
    use std::collections::HashSet;

    const N: usize = 256;
    let alice = bolt_core::crypto::generate_ephemeral_keypair();
    let bob = bolt_core::crypto::generate_ephemeral_keypair();

    let mut seen = HashSet::new();
    let zero = [0u8; bolt_core::constants::NONCE_LENGTH];

    for i in 0..N {
        let sealed = bolt_core::crypto::seal_box_payload(
            b"nonce-conformance",
            &bob.public_key,
            &alice.secret_key,
        )
        .unwrap();
        let raw = bolt_core::encoding::from_base64(&sealed).unwrap();
        let nonce: [u8; 24] = raw[..bolt_core::constants::NONCE_LENGTH]
            .try_into()
            .unwrap();

        assert_ne!(nonce, zero, "nonce must not be all-zero (seal #{i})");
        assert!(
            seen.insert(nonce),
            "duplicate nonce at seal #{i} after {} unique seals",
            seen.len()
        );
    }

    assert_eq!(seen.len(), N);
}

/// SEC-01: Sealed output is always valid base64.
#[test]
fn conformance_wire_format_valid_base64() {
    let alice = bolt_core::crypto::generate_ephemeral_keypair();
    let bob = bolt_core::crypto::generate_ephemeral_keypair();

    let sealed = bolt_core::crypto::seal_box_payload(
        b"base64 conformance",
        &bob.public_key,
        &alice.secret_key,
    )
    .unwrap();

    // Must decode without error.
    let decoded = bolt_core::encoding::from_base64(&sealed);
    assert!(
        decoded.is_ok(),
        "sealed output is not valid base64: {:?}",
        decoded.err()
    );
}

// ── Conformance: Seal-then-Open Round Trip ──────────────────────

/// PROTO-07: Seal then open round trip across varied payload sizes.
#[test]
fn conformance_seal_open_roundtrip_varied_sizes() {
    let alice = bolt_core::crypto::generate_ephemeral_keypair();
    let bob = bolt_core::crypto::generate_ephemeral_keypair();

    let payloads: Vec<Vec<u8>> = vec![
        vec![],                   // empty
        vec![0xFF],               // single byte
        b"Hello, Bolt!".to_vec(), // typical
        vec![0xAB; 1024],         // 1 KiB
        (0..=255).collect(),      // all byte values
    ];

    for (i, plaintext) in payloads.iter().enumerate() {
        let sealed =
            bolt_core::crypto::seal_box_payload(plaintext, &bob.public_key, &alice.secret_key)
                .unwrap_or_else(|e| panic!("seal failed for payload #{i}: {e}"));

        let opened =
            bolt_core::crypto::open_box_payload(&sealed, &alice.public_key, &bob.secret_key)
                .unwrap_or_else(|e| panic!("open failed for payload #{i}: {e}"));

        assert_eq!(
            opened,
            *plaintext,
            "round-trip mismatch for payload #{i} ({} bytes)",
            plaintext.len()
        );
    }
}
