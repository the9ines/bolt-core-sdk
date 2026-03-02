//! Error types for bolt-core.
//!
//! Maps to TS error hierarchy: `BoltError` (base), `EncryptionError`,
//! `ConnectionError`, `TransferError`, `IntegrityError`. Rust uses an
//! enum instead of class inheritance.
//!
//! ## Parity gate (R1)
//! Error variant names and messages must match TS error class names
//! and default messages for interop diagnostics.

/// Unified error type for all bolt-core operations.
#[derive(Debug, thiserror::Error)]
pub enum BoltError {
    /// Encryption or decryption failure (NaCl box).
    #[error("Encryption error: {0}")]
    Encryption(String),

    /// Connection-level error.
    #[error("Connection error: {0}")]
    Connection(String),

    /// File transfer error.
    #[error("Transfer error: {0}")]
    Transfer(String),

    /// File integrity check failed.
    #[error("Integrity error: {0}")]
    Integrity(String),

    /// Encoding error (base64, hex).
    #[error("Encoding error: {0}")]
    Encoding(String),
}

// ── Wire Error Code Registry (PROTOCOL.md §10, v0.1.3-spec) ──────────

/// Canonical wire error code registry — 22 codes (11 PROTOCOL + 11 ENFORCEMENT).
///
/// Every error frame sent on the wire MUST use a code from this array.
/// String identifiers match the TS `WIRE_ERROR_CODES` array in `errors.ts`.
///
/// Registry is canonical in both Rust bolt-core (here) and TS bolt-core.
/// Emission of certain codes (especially ENFORCEMENT-class) is TS-owned
/// at the transport layer (transport-web). The conformance test in
/// `tests/conformance/error_code_mapping.rs` documents which error
/// flows/types are represented where.
pub const WIRE_ERROR_CODES: [&str; 22] = [
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
];

/// Returns `true` if the given string is a canonical wire error code
/// from PROTOCOL.md §10.
pub fn is_valid_wire_error_code(code: &str) -> bool {
    WIRE_ERROR_CODES.contains(&code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_matches_ts_format() {
        let err = BoltError::Encryption("Decryption failed".into());
        assert_eq!(err.to_string(), "Encryption error: Decryption failed");

        let err = BoltError::Integrity("File integrity check failed".into());
        assert_eq!(
            err.to_string(),
            "Integrity error: File integrity check failed"
        );
    }

    #[test]
    fn error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<BoltError>();
    }

    #[test]
    fn wire_error_registry_has_22_codes() {
        assert_eq!(WIRE_ERROR_CODES.len(), 22);
    }

    #[test]
    fn wire_error_registry_protocol_class_count() {
        // First 11 are PROTOCOL class
        let protocol = &WIRE_ERROR_CODES[..11];
        assert_eq!(protocol.len(), 11);
        assert_eq!(protocol[0], "VERSION_MISMATCH");
        assert_eq!(protocol[10], "KEY_MISMATCH");
    }

    #[test]
    fn wire_error_registry_enforcement_class_count() {
        // Last 11 are ENFORCEMENT class
        let enforcement = &WIRE_ERROR_CODES[11..];
        assert_eq!(enforcement.len(), 11);
        assert_eq!(enforcement[0], "DUPLICATE_HELLO");
        assert_eq!(enforcement[10], "PROTOCOL_VIOLATION");
    }

    #[test]
    fn wire_error_registry_accepts_canonical() {
        assert!(is_valid_wire_error_code("KEY_MISMATCH"));
        assert!(is_valid_wire_error_code("DUPLICATE_HELLO"));
        assert!(is_valid_wire_error_code("PROTOCOL_VIOLATION"));
    }

    #[test]
    fn wire_error_registry_rejects_unknown() {
        assert!(!is_valid_wire_error_code("NOT_A_REAL_CODE"));
        assert!(!is_valid_wire_error_code(""));
        assert!(!is_valid_wire_error_code("key_mismatch")); // case-sensitive
    }

    #[test]
    fn wire_error_registry_all_unique() {
        let mut seen = std::collections::HashSet::new();
        for code in &WIRE_ERROR_CODES {
            assert!(seen.insert(code), "duplicate wire error code: {code}");
        }
    }
}
