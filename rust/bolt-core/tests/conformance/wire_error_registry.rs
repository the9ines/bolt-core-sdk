//! Conformance: Wire Error Code Registry (PROTOCOL.md §10 + §16.7)
//!
//! Asserts the canonical 26-code registry exists in Rust bolt-core and
//! matches the expected exact list and order from PROTOCOL.md §10
//! (11 PROTOCOL + 11 ENFORCEMENT + 4 BTR).
//!
//! TS parity: `ts/bolt-core/__tests__/wire-error-codes.test.ts` asserts
//! the same code list in TypeScript.

use bolt_core::errors::{is_valid_wire_error_code, WIRE_ERROR_CODES};

/// Exact list and order must match PROTOCOL.md §10 + §16.7.
#[test]
fn conformance_wire_error_registry_exact_list() {
    let expected: [&str; 26] = [
        // PROTOCOL class (11)
        "VERSION_MISMATCH",
        "ENCRYPTION_FAILED",
        "INTEGRITY_FAILED",
        "REPLAY_DETECTED",
        "TRANSFER_FAILED",
        "LIMIT_EXCEEDED",
        "CONNECTION_LOST",
        "PEER_NOT_FOUND",
        "ALREADY_CONNECTED",
        "INVALID_STATE",
        "KEY_MISMATCH",
        // ENFORCEMENT class (11)
        "DUPLICATE_HELLO",
        "ENVELOPE_REQUIRED",
        "ENVELOPE_UNNEGOTIATED",
        "ENVELOPE_DECRYPT_FAIL",
        "ENVELOPE_INVALID",
        "HELLO_PARSE_ERROR",
        "HELLO_DECRYPT_FAIL",
        "HELLO_SCHEMA_ERROR",
        "INVALID_MESSAGE",
        "UNKNOWN_MESSAGE_TYPE",
        "PROTOCOL_VIOLATION",
        // BTR class (4) — §16.7
        "RATCHET_STATE_ERROR",
        "RATCHET_CHAIN_ERROR",
        "RATCHET_DECRYPT_FAIL",
        "RATCHET_DOWNGRADE_REJECTED",
    ];

    assert_eq!(
        WIRE_ERROR_CODES, expected,
        "WIRE_ERROR_CODES drift from PROTOCOL.md §10 + §16.7"
    );
}

/// Length must be exactly 26 (11 PROTOCOL + 11 ENFORCEMENT + 4 BTR).
#[test]
fn conformance_wire_error_registry_length() {
    assert_eq!(WIRE_ERROR_CODES.len(), 26);
}

/// All codes must be unique.
#[test]
fn conformance_wire_error_registry_unique() {
    let mut seen = std::collections::HashSet::new();
    for code in &WIRE_ERROR_CODES {
        assert!(seen.insert(code), "duplicate wire error code: {code}");
    }
}

/// is_valid_wire_error_code accepts all 26 canonical codes.
#[test]
fn conformance_wire_error_validator_accepts_all() {
    for code in &WIRE_ERROR_CODES {
        assert!(
            is_valid_wire_error_code(code),
            "validator rejected canonical code: {code}"
        );
    }
}

/// is_valid_wire_error_code rejects unknown codes.
#[test]
fn conformance_wire_error_validator_rejects_unknown() {
    assert!(!is_valid_wire_error_code("NOT_A_REAL_CODE"));
    assert!(!is_valid_wire_error_code(""));
    assert!(!is_valid_wire_error_code("key_mismatch")); // case-sensitive
}
