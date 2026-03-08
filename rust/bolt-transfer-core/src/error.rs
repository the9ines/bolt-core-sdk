//! Transfer error types.

/// Transfer state machine error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransferError {
    /// State transition is invalid for the current state.
    InvalidTransition(String),
    /// File integrity verification failed (hash mismatch).
    IntegrityFailed(String),
}

impl std::fmt::Display for TransferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransferError::InvalidTransition(detail) => write!(f, "{detail}"),
            TransferError::IntegrityFailed(detail) => write!(f, "{detail}"),
        }
    }
}

impl std::error::Error for TransferError {}
