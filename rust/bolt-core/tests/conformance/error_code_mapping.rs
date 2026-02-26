//! Conformance: Error Code Mapping (Appendix A, Rust-surface only)
//!
//! Tests that Rust-exposed error types are stable and correctly mapped.
//!
//! PROTOCOL_ENFORCEMENT.md Appendix A defines 14 transport-level error codes.
//! Of these, the Rust core SDK exposes:
//! - BoltError::Encryption  — maps to ENVELOPE_DECRYPT_FAIL, HELLO_DECRYPT_FAIL
//! - BoltError::Encoding    — maps to encoding/parse failures
//! - BoltError::Integrity   — maps to file integrity failures
//! - BoltError::Connection  — maps to connection-level errors
//! - BoltError::Transfer    — maps to transfer-level errors
//! - KeyMismatchError       — maps to KEY_MISMATCH
//!
//! Appendix A codes NOT represented as stable Rust types (TS-owned):
//! - DUPLICATE_HELLO, ENVELOPE_REQUIRED, ENVELOPE_UNNEGOTIATED,
//!   ENVELOPE_INVALID, HELLO_PARSE_ERROR, HELLO_SCHEMA_ERROR,
//!   INVALID_MESSAGE, UNKNOWN_MESSAGE_TYPE, INVALID_STATE,
//!   LIMIT_EXCEEDED, PROTOCOL_VIOLATION
//!
//! These are documented in the AAR as TS-owned invariants.

// ── Conformance: BoltError Variant Stability ────────────────────

/// All 5 BoltError variants produce the expected display format.
/// Format: "{Category} error: {message}"
/// Parity gate R1: these MUST match TS error class names.
#[test]
fn conformance_bolt_error_display_format_stable() {
    use bolt_core::errors::BoltError;

    let cases: Vec<(BoltError, &str)> = vec![
        (
            BoltError::Encryption("Decryption failed".into()),
            "Encryption error: Decryption failed",
        ),
        (
            BoltError::Connection("peer unreachable".into()),
            "Connection error: peer unreachable",
        ),
        (
            BoltError::Transfer("chunk missing".into()),
            "Transfer error: chunk missing",
        ),
        (
            BoltError::Integrity("File integrity check failed".into()),
            "Integrity error: File integrity check failed",
        ),
        (
            BoltError::Encoding("invalid base64".into()),
            "Encoding error: invalid base64",
        ),
    ];

    for (error, expected) in &cases {
        assert_eq!(
            error.to_string(),
            *expected,
            "BoltError display drift detected"
        );
    }
}

/// BoltError MUST implement Send + Sync for safe cross-thread use.
#[test]
fn conformance_bolt_error_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<bolt_core::errors::BoltError>();
}

/// BoltError MUST implement std::error::Error.
#[test]
fn conformance_bolt_error_is_std_error() {
    use bolt_core::errors::BoltError;
    let err = BoltError::Encryption("test".into());
    let _: &dyn std::error::Error = &err;
}

// ── Conformance: KeyMismatchError Stability ─────────────────────

/// KeyMismatchError display format MUST be stable for diagnostics.
/// Maps to Appendix A: KEY_MISMATCH.
#[test]
fn conformance_key_mismatch_error_display_stable() {
    let err = bolt_core::identity::KeyMismatchError {
        peer_code: "ABC123".into(),
        expected: [1u8; 32],
        received: [2u8; 32],
    };
    assert_eq!(
        err.to_string(),
        "Identity key mismatch for peer ABC123",
        "KeyMismatchError display drift detected"
    );
}

/// KeyMismatchError MUST implement std::error::Error.
#[test]
fn conformance_key_mismatch_is_std_error() {
    let err = bolt_core::identity::KeyMismatchError {
        peer_code: "XYZ".into(),
        expected: [0u8; 32],
        received: [0u8; 32],
    };
    let _: &dyn std::error::Error = &err;
}

/// KeyMismatchError fields MUST carry the pinned and received keys.
#[test]
fn conformance_key_mismatch_carries_key_data() {
    let expected_key = [0xAAu8; 32];
    let received_key = [0xBBu8; 32];

    let err = bolt_core::identity::KeyMismatchError {
        peer_code: "PEER42".into(),
        expected: expected_key,
        received: received_key,
    };

    assert_eq!(err.peer_code, "PEER42");
    assert_eq!(err.expected, expected_key);
    assert_eq!(err.received, received_key);
}

// ── Conformance: Crypto Operations → Error Variant Mapping ──────

/// open_box_payload with wrong sender key MUST produce BoltError::Encryption.
#[test]
fn conformance_wrong_key_maps_to_encryption_error() {
    let alice = bolt_core::crypto::generate_ephemeral_keypair();
    let bob = bolt_core::crypto::generate_ephemeral_keypair();
    let eve = bolt_core::crypto::generate_ephemeral_keypair();

    let sealed =
        bolt_core::crypto::seal_box_payload(b"test", &bob.public_key, &alice.secret_key).unwrap();

    let err =
        bolt_core::crypto::open_box_payload(&sealed, &eve.public_key, &bob.secret_key).unwrap_err();

    assert!(
        err.to_string().starts_with("Encryption error:"),
        "wrong key should produce Encryption error, got: {err}"
    );
}

/// open_box_payload with tampered ciphertext MUST produce BoltError::Encryption.
#[test]
fn conformance_tampered_payload_maps_to_encryption_error() {
    let alice = bolt_core::crypto::generate_ephemeral_keypair();
    let bob = bolt_core::crypto::generate_ephemeral_keypair();

    let sealed =
        bolt_core::crypto::seal_box_payload(b"test", &bob.public_key, &alice.secret_key).unwrap();

    let mut raw = bolt_core::encoding::from_base64(&sealed).unwrap();
    // Flip last byte.
    let last = raw.len() - 1;
    raw[last] ^= 0xFF;
    let tampered = bolt_core::encoding::to_base64(&raw);

    let err = bolt_core::crypto::open_box_payload(&tampered, &alice.public_key, &bob.secret_key)
        .unwrap_err();

    assert!(
        err.to_string().starts_with("Encryption error:"),
        "tampered payload should produce Encryption error, got: {err}"
    );
}

/// from_base64 with invalid input MUST produce BoltError::Encoding.
#[test]
fn conformance_invalid_base64_maps_to_encoding_error() {
    let err = bolt_core::encoding::from_base64("!!!not-base64!!!").unwrap_err();
    assert!(
        err.to_string().starts_with("Encoding error:"),
        "invalid base64 should produce Encoding error, got: {err}"
    );
}

/// from_hex with odd-length input MUST produce BoltError::Encoding.
#[test]
fn conformance_odd_hex_maps_to_encoding_error() {
    let err = bolt_core::encoding::from_hex("abc").unwrap_err();
    assert!(
        err.to_string().starts_with("Encoding error:"),
        "odd-length hex should produce Encoding error, got: {err}"
    );
}

/// open_box_payload with short payload MUST produce an error (Encryption or Encoding).
#[test]
fn conformance_short_payload_produces_error() {
    let kp = bolt_core::crypto::generate_ephemeral_keypair();
    let short = bolt_core::encoding::to_base64(&[0u8; 10]);
    let result = bolt_core::crypto::open_box_payload(&short, &kp.public_key, &kp.secret_key);
    assert!(
        result.is_err(),
        "payload shorter than nonce must be rejected"
    );
}
