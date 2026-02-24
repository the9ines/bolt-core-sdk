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
}
