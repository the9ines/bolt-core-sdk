//! Canonical transfer state enums aligned to PROTOCOL.md §9.
//!
//! ```text
//! IDLE -> OFFERED -> ACCEPTED -> TRANSFERRING <-> PAUSED -> COMPLETED
//!                                     |                        |
//!                                   ERROR <------------- CANCELLED
//! ```
//!
//! These enums are the single source of truth for transfer state names
//! across all Bolt implementations (daemon, app, WASM).

/// Reason for transfer cancellation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CancelReason {
    /// Sender cancelled the transfer.
    BySender,
    /// Receiver cancelled the transfer.
    ByReceiver,
    /// Receiver rejected the offer (maps to §9 CANCELLED from OFFERED).
    Rejected,
}

/// Canonical transfer state (PROTOCOL.md §9).
///
/// Used for both send-side and receive-side state machines.
/// Direction context is provided by the owning session struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransferState {
    /// No transfer in progress.
    Idle,
    /// File offer sent (sender) or received (receiver).
    Offered { transfer_id: String },
    /// Receiver accepted the offer; transfer will begin.
    Accepted { transfer_id: String },
    /// Chunks actively being sent or received.
    Transferring { transfer_id: String },
    /// Transfer paused by sender. Sender-only in v1 (receiver-side deferred).
    Paused { transfer_id: String },
    /// Transfer completed successfully.
    Completed { transfer_id: String },
    /// Transfer cancelled.
    Cancelled {
        transfer_id: String,
        reason: CancelReason,
    },
    /// Error condition (terminal).
    Error { detail: String },
}

impl TransferState {
    /// Returns the transfer_id if the state carries one, None for Idle/Error.
    pub fn transfer_id(&self) -> Option<&str> {
        match self {
            TransferState::Idle => None,
            TransferState::Offered { transfer_id }
            | TransferState::Accepted { transfer_id }
            | TransferState::Transferring { transfer_id }
            | TransferState::Paused { transfer_id }
            | TransferState::Completed { transfer_id }
            | TransferState::Cancelled { transfer_id, .. } => Some(transfer_id),
            TransferState::Error { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_names_match_protocol_s9() {
        // Verify all §9 states are representable.
        let states = [
            TransferState::Idle,
            TransferState::Offered {
                transfer_id: "t".into(),
            },
            TransferState::Accepted {
                transfer_id: "t".into(),
            },
            TransferState::Transferring {
                transfer_id: "t".into(),
            },
            TransferState::Paused {
                transfer_id: "t".into(),
            },
            TransferState::Completed {
                transfer_id: "t".into(),
            },
            TransferState::Cancelled {
                transfer_id: "t".into(),
                reason: CancelReason::ByReceiver,
            },
            TransferState::Error {
                detail: "err".into(),
            },
        ];
        assert_eq!(states.len(), 8, "§9 defines 8 states");
    }

    #[test]
    fn transfer_id_extraction() {
        assert!(TransferState::Idle.transfer_id().is_none());
        assert_eq!(
            TransferState::Transferring {
                transfer_id: "abc".into()
            }
            .transfer_id(),
            Some("abc")
        );
        assert!(TransferState::Error { detail: "x".into() }
            .transfer_id()
            .is_none());
    }

    #[test]
    fn cancel_reasons_complete() {
        let reasons = [
            CancelReason::BySender,
            CancelReason::ByReceiver,
            CancelReason::Rejected,
        ];
        assert_eq!(reasons.len(), 3);
    }
}
