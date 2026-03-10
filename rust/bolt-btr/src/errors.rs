//! BTR error types — §16.7 error behavior.
//!
//! Four error codes with deterministic behavior mapping:
//! - `RatchetStateError` → disconnect
//! - `RatchetChainError` → cancel transfer
//! - `RatchetDecryptFail` → cancel transfer
//! - `RatchetDowngradeRejected` → disconnect

/// BTR-specific error type.
#[derive(Debug, thiserror::Error)]
pub enum BtrError {
    /// Ratchet generation mismatch, unexpected DH key, or missing required
    /// BTR fields. Required action: send error inside envelope, disconnect.
    #[error("RATCHET_STATE_ERROR: {0}")]
    RatchetStateError(String),

    /// chain_index != expected next, chain index gap.
    /// Required action: send error inside envelope, cancel transfer.
    #[error("RATCHET_CHAIN_ERROR: {0}")]
    RatchetChainError(String),

    /// NaCl secretbox open fails with BTR message key.
    /// Required action: send error inside envelope, cancel transfer.
    #[error("RATCHET_DECRYPT_FAIL: {0}")]
    RatchetDecryptFail(String),

    /// Peer advertised BTR capability but sends non-BTR or invalid BTR
    /// envelopes. Required action: send error inside envelope, disconnect.
    /// Triggering semantics deferred to BTR-4.
    #[error("RATCHET_DOWNGRADE_REJECTED: {0}")]
    RatchetDowngradeRejected(String),
}

impl BtrError {
    /// Returns the wire error code string for this error.
    pub fn wire_code(&self) -> &'static str {
        match self {
            Self::RatchetStateError(_) => "RATCHET_STATE_ERROR",
            Self::RatchetChainError(_) => "RATCHET_CHAIN_ERROR",
            Self::RatchetDecryptFail(_) => "RATCHET_DECRYPT_FAIL",
            Self::RatchetDowngradeRejected(_) => "RATCHET_DOWNGRADE_REJECTED",
        }
    }

    /// Returns `true` if the required action is disconnect (vs cancel transfer).
    pub fn requires_disconnect(&self) -> bool {
        matches!(
            self,
            Self::RatchetStateError(_) | Self::RatchetDowngradeRejected(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_codes_match_spec() {
        let err = BtrError::RatchetStateError("test".into());
        assert_eq!(err.wire_code(), "RATCHET_STATE_ERROR");

        let err = BtrError::RatchetChainError("test".into());
        assert_eq!(err.wire_code(), "RATCHET_CHAIN_ERROR");

        let err = BtrError::RatchetDecryptFail("test".into());
        assert_eq!(err.wire_code(), "RATCHET_DECRYPT_FAIL");

        let err = BtrError::RatchetDowngradeRejected("test".into());
        assert_eq!(err.wire_code(), "RATCHET_DOWNGRADE_REJECTED");
    }

    #[test]
    fn disconnect_semantics() {
        assert!(BtrError::RatchetStateError("".into()).requires_disconnect());
        assert!(!BtrError::RatchetChainError("".into()).requires_disconnect());
        assert!(!BtrError::RatchetDecryptFail("".into()).requires_disconnect());
        assert!(BtrError::RatchetDowngradeRejected("".into()).requires_disconnect());
    }

    #[test]
    fn display_includes_code_prefix() {
        let err = BtrError::RatchetStateError("generation mismatch".into());
        assert!(err.to_string().starts_with("RATCHET_STATE_ERROR:"));
    }

    #[test]
    fn error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<BtrError>();
    }
}
