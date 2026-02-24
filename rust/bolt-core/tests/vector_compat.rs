#![cfg(feature = "vectors")]
//! Vector compatibility gate.
//!
//! Parses the golden test vectors produced by the TypeScript SDK and
//! validates structure, field presence, and counts. This ensures the
//! Rust crate is anchored to the same canonical test data.
//!
//! Does NOT perform cryptographic operations yet — that requires
//! NaCl/crypto_box implementation. This is a plumbing gate only.

#![allow(non_snake_case, dead_code)]

use serde::Deserialize;
use std::path::PathBuf;

// ── box-payload vector schema ───────────────────────────────────────

#[derive(Deserialize)]
struct BoxPayloadVectors {
    _WARNING: String,
    description: String,
    sender: Keypair,
    receiver: Keypair,
    eve: Keypair,
    vectors: Vec<BoxVector>,
    corrupt_vectors: Vec<CorruptVector>,
}

#[derive(Deserialize)]
struct Keypair {
    publicKey_base64: String,
    secretKey_base64: String,
    publicKey_hex: String,
    secretKey_hex: String,
}

#[derive(Deserialize)]
struct BoxVector {
    id: String,
    plaintext_utf8: Option<String>,
    plaintext_hex: Option<String>,
    nonce_hex: String,
    sealed_base64: String,
}

#[derive(Deserialize)]
struct CorruptVector {
    id: String,
    description: String,
    sealed_base64: String,
    expected_error: String,
    use_eve_as_sender: Option<bool>,
}

// ── framing vector schema ───────────────────────────────────────────

#[derive(Deserialize)]
struct FramingVectors {
    _WARNING: String,
    description: String,
    constants: FramingConstants,
    vectors: Vec<FramingVector>,
}

#[derive(Deserialize)]
struct FramingConstants {
    nonce_length: usize,
    box_overhead: usize,
}

#[derive(Deserialize)]
struct FramingVector {
    id: String,
    sealed_base64: String,
    expected_decoded_length: usize,
    expected_nonce_hex: String,
    expected_ciphertext_length: usize,
    plaintext_length: usize,
}

// ── helpers ─────────────────────────────────────────────────────────

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

// ── tests ───────────────────────────────────────────────────────────

#[test]
fn box_payload_vectors_parse() {
    let path = vectors_dir().join("box-payload.vectors.json");
    let data = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
    let vecs: BoxPayloadVectors =
        serde_json::from_str(&data).expect("box-payload vectors failed to parse");

    // Validate structure
    assert!(!vecs._WARNING.is_empty());
    assert!(!vecs.description.is_empty());

    // Keypairs present and non-empty
    for (name, kp) in [
        ("sender", &vecs.sender),
        ("receiver", &vecs.receiver),
        ("eve", &vecs.eve),
    ] {
        assert!(
            !kp.publicKey_base64.is_empty(),
            "{name} publicKey_base64 empty"
        );
        assert!(
            !kp.secretKey_base64.is_empty(),
            "{name} secretKey_base64 empty"
        );
        assert!(!kp.publicKey_hex.is_empty(), "{name} publicKey_hex empty");
        assert!(!kp.secretKey_hex.is_empty(), "{name} secretKey_hex empty");
        // Hex public key should be 64 chars (32 bytes)
        assert_eq!(
            kp.publicKey_hex.len(),
            64,
            "{name} publicKey_hex wrong length"
        );
        // Hex secret key should be 64 chars (32 bytes)
        assert_eq!(
            kp.secretKey_hex.len(),
            64,
            "{name} secretKey_hex wrong length"
        );
    }

    // Exactly 4 valid vectors
    assert_eq!(vecs.vectors.len(), 4, "expected 4 box-payload vectors");
    let expected_ids = [
        "hello-bolt",
        "empty-payload",
        "single-byte-ff",
        "256-byte-pattern",
    ];
    for (i, v) in vecs.vectors.iter().enumerate() {
        assert_eq!(v.id, expected_ids[i], "vector {i} id mismatch");
        assert!(!v.nonce_hex.is_empty(), "vector {} nonce_hex empty", v.id);
        assert!(
            !v.sealed_base64.is_empty(),
            "vector {} sealed_base64 empty",
            v.id
        );
        // Nonce hex should be 48 chars (24 bytes)
        assert_eq!(
            v.nonce_hex.len(),
            48,
            "vector {} nonce_hex wrong length",
            v.id
        );
    }

    // Exactly 4 corrupt vectors
    assert_eq!(vecs.corrupt_vectors.len(), 4, "expected 4 corrupt vectors");
    let corrupt_ids = [
        "modified-ciphertext",
        "truncated-ciphertext",
        "wrong-sender-key",
        "nonce-only",
    ];
    for (i, v) in vecs.corrupt_vectors.iter().enumerate() {
        assert_eq!(v.id, corrupt_ids[i], "corrupt vector {i} id mismatch");
        assert!(
            !v.sealed_base64.is_empty(),
            "corrupt vector {} sealed_base64 empty",
            v.id
        );
        assert_eq!(
            v.expected_error, "Decryption failed",
            "corrupt vector {} expected_error mismatch",
            v.id
        );
    }
}

#[test]
fn framing_vectors_parse() {
    let path = vectors_dir().join("framing.vectors.json");
    let data = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
    let vecs: FramingVectors =
        serde_json::from_str(&data).expect("framing vectors failed to parse");

    // Validate structure
    assert!(!vecs._WARNING.is_empty());
    assert!(!vecs.description.is_empty());

    // Constants match protocol
    assert_eq!(
        vecs.constants.nonce_length,
        bolt_core::constants::NONCE_LENGTH
    );
    assert_eq!(
        vecs.constants.box_overhead,
        bolt_core::constants::BOX_OVERHEAD
    );

    // Exactly 4 framing vectors
    assert_eq!(vecs.vectors.len(), 4, "expected 4 framing vectors");
    let expected_ids = [
        "hello-bolt-framing",
        "empty-framing",
        "single-byte-framing",
        "256-byte-framing",
    ];
    for (i, v) in vecs.vectors.iter().enumerate() {
        assert_eq!(v.id, expected_ids[i], "framing vector {i} id mismatch");
        assert!(
            !v.sealed_base64.is_empty(),
            "framing vector {} sealed_base64 empty",
            v.id
        );
        // Nonce hex should be 48 chars (24 bytes)
        assert_eq!(
            v.expected_nonce_hex.len(),
            48,
            "framing vector {} nonce_hex wrong length",
            v.id
        );
        // decoded_length = nonce_length + ciphertext_length
        assert_eq!(
            v.expected_decoded_length,
            vecs.constants.nonce_length + v.expected_ciphertext_length,
            "framing vector {} decoded_length invariant broken",
            v.id
        );
        // ciphertext_length = plaintext_length + box_overhead
        assert_eq!(
            v.expected_ciphertext_length,
            v.plaintext_length + vecs.constants.box_overhead,
            "framing vector {} ciphertext_length invariant broken",
            v.id
        );
    }
}

#[test]
fn vectors_are_not_modified() {
    // Ensure vector files exist and are non-empty.
    // This gate prevents accidental deletion or truncation.
    let dir = vectors_dir();
    let box_path = dir.join("box-payload.vectors.json");
    let framing_path = dir.join("framing.vectors.json");

    let box_meta =
        std::fs::metadata(&box_path).unwrap_or_else(|e| panic!("{}: {}", box_path.display(), e));
    let framing_meta = std::fs::metadata(&framing_path)
        .unwrap_or_else(|e| panic!("{}: {}", framing_path.display(), e));

    assert!(
        box_meta.len() > 1000,
        "box-payload vectors suspiciously small"
    );
    assert!(
        framing_meta.len() > 500,
        "framing vectors suspiciously small"
    );
}
