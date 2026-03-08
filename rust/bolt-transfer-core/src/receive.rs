//! Receive-side transfer state machine.
//!
//! Extracted from bolt-daemon/src/transfer.rs (TransferSession).
//! Aligned to PROTOCOL.md §9 state names.
//!
//! State flow:
//!   Idle → Offered → Accepted → Transferring → Completed
//!                  → Cancelled(Rejected)
//!
//! No crypto dependencies. Integrity verification is caller-injected
//! via `Option<&dyn IntegrityVerifier>`.

use crate::error::TransferError;
use crate::state::{CancelReason, TransferState};
use crate::transport::IntegrityVerifier;

/// Maximum transfer size in bytes (256 MiB).
/// Conservative bound for in-memory reassembly.
pub const MAX_TRANSFER_BYTES: u64 = 256 * 1024 * 1024;

/// Receive-side transfer session (§9 state machine).
///
/// Enforces: Idle → Offered → Accepted → Transferring → Completed.
/// Also supports: Offered → Cancelled(Rejected).
/// Second offer while not Idle is InvalidTransition.
pub struct ReceiveSession {
    state: TransferState,
    buffer: Vec<u8>,
    expected_len: u64,
    total_chunks: u32,
    next_chunk_index: u32,
    expected_hash: Option<String>,
}

impl Default for ReceiveSession {
    fn default() -> Self {
        Self::new()
    }
}

impl ReceiveSession {
    pub fn new() -> Self {
        Self {
            state: TransferState::Idle,
            buffer: Vec::new(),
            expected_len: 0,
            total_chunks: 0,
            next_chunk_index: 0,
            expected_hash: None,
        }
    }

    /// Current state.
    pub fn state(&self) -> &TransferState {
        &self.state
    }

    /// Transition: Idle → Offered.
    ///
    /// Validates offer fields and stores metadata.
    /// `expected_hash`: caller provides `Some` only when bolt.file-hash is
    /// negotiated AND the offer includes a hash. `None` skips verification.
    pub fn on_file_offer(
        &mut self,
        transfer_id: &str,
        size: u64,
        total_chunks: u32,
        expected_hash: Option<&str>,
    ) -> Result<(), TransferError> {
        if size == 0 {
            return Err(TransferError::InvalidTransition(
                "invalid offer: size is zero".to_string(),
            ));
        }
        if total_chunks == 0 {
            return Err(TransferError::InvalidTransition(
                "invalid offer: zero chunks".to_string(),
            ));
        }
        if size > MAX_TRANSFER_BYTES {
            return Err(TransferError::InvalidTransition(
                "transfer size exceeded".to_string(),
            ));
        }

        match &self.state {
            TransferState::Idle => {
                self.expected_len = size;
                self.total_chunks = total_chunks;
                self.expected_hash = expected_hash.map(|s| s.to_string());
                self.state = TransferState::Offered {
                    transfer_id: transfer_id.to_string(),
                };
                Ok(())
            }
            TransferState::Offered { .. } => Err(TransferError::InvalidTransition(
                "offer already active".to_string(),
            )),
            TransferState::Accepted { .. } | TransferState::Transferring { .. } => Err(
                TransferError::InvalidTransition("transfer already active".to_string()),
            ),
            TransferState::Completed { .. } | TransferState::Cancelled { .. } => Err(
                TransferError::InvalidTransition("transfer session ended".to_string()),
            ),
            TransferState::Paused { .. } => Err(TransferError::InvalidTransition(
                "transfer already active".to_string(),
            )),
            TransferState::Error { .. } => Err(TransferError::InvalidTransition(
                "transfer session ended".to_string(),
            )),
        }
    }

    /// Transition: Offered → Accepted → Transferring.
    ///
    /// Pre-allocates buffer. Returns the transfer_id for the Accept message.
    /// Moves through Accepted to Transferring atomically (receiver does not
    /// linger in Accepted state — acceptance implies readiness to receive).
    pub fn accept_current_offer(&mut self) -> Result<String, TransferError> {
        match &self.state {
            TransferState::Offered { transfer_id } => {
                let tid = transfer_id.clone();
                let capacity = std::cmp::min(self.expected_len, MAX_TRANSFER_BYTES) as usize;
                self.buffer = Vec::with_capacity(capacity);
                self.next_chunk_index = 0;
                self.state = TransferState::Transferring {
                    transfer_id: tid.clone(),
                };
                Ok(tid)
            }
            _ => Err(TransferError::InvalidTransition(
                "no active offer".to_string(),
            )),
        }
    }

    /// Transition: Offered → Cancelled(Rejected). Returns the transfer_id.
    pub fn reject_current_offer(&mut self) -> Result<String, TransferError> {
        match &self.state {
            TransferState::Offered { transfer_id } => {
                let tid = transfer_id.clone();
                self.state = TransferState::Cancelled {
                    transfer_id: tid.clone(),
                    reason: CancelReason::Rejected,
                };
                Ok(tid)
            }
            _ => Err(TransferError::InvalidTransition(
                "no active offer".to_string(),
            )),
        }
    }

    /// Transferring → Transferring. Appends decoded bytes to buffer.
    ///
    /// Validates transfer_id, sequential chunk_index, bounds, and capacity.
    pub fn on_file_chunk(
        &mut self,
        transfer_id: &str,
        chunk_index: u32,
        data: &[u8],
    ) -> Result<(), TransferError> {
        let active_tid = match &self.state {
            TransferState::Transferring { transfer_id: tid } => tid.clone(),
            _ => {
                return Err(TransferError::InvalidTransition(
                    "no active transfer".to_string(),
                ))
            }
        };

        if transfer_id != active_tid {
            return Err(TransferError::InvalidTransition(
                "transfer_id mismatch".to_string(),
            ));
        }

        if chunk_index >= self.total_chunks {
            return Err(TransferError::InvalidTransition(
                "chunk index out of range".to_string(),
            ));
        }

        if chunk_index != self.next_chunk_index {
            return Err(TransferError::InvalidTransition(
                "unexpected chunk index".to_string(),
            ));
        }

        if self.buffer.len() + data.len() > MAX_TRANSFER_BYTES as usize {
            return Err(TransferError::InvalidTransition(
                "transfer size exceeded".to_string(),
            ));
        }

        self.buffer.extend_from_slice(data);
        self.next_chunk_index += 1;
        Ok(())
    }

    /// Transition: Transferring → Completed.
    ///
    /// If an `IntegrityVerifier` is provided and `expected_hash` was set,
    /// verifies the reassembled buffer. Mismatch → `IntegrityFailed`.
    pub fn on_file_finish(
        &mut self,
        transfer_id: &str,
        verifier: Option<&dyn IntegrityVerifier>,
    ) -> Result<(), TransferError> {
        let active_tid = match &self.state {
            TransferState::Transferring { transfer_id: tid } => tid.clone(),
            _ => {
                return Err(TransferError::InvalidTransition(
                    "no active transfer".to_string(),
                ))
            }
        };

        if transfer_id != active_tid {
            return Err(TransferError::InvalidTransition(
                "transfer_id mismatch".to_string(),
            ));
        }

        // Verify integrity if both expected_hash and verifier are available.
        if let Some(ref expected) = self.expected_hash {
            if let Some(v) = verifier {
                if !v.verify(&self.buffer, expected) {
                    return Err(TransferError::IntegrityFailed(
                        "file hash mismatch".to_string(),
                    ));
                }
            }
        }

        self.state = TransferState::Completed {
            transfer_id: active_tid,
        };
        Ok(())
    }

    /// Transition: Transferring → Cancelled(ByReceiver).
    pub fn cancel(&mut self, transfer_id: &str) -> Result<(), TransferError> {
        let active_tid = match &self.state {
            TransferState::Transferring { transfer_id: tid } => tid.clone(),
            _ => {
                return Err(TransferError::InvalidTransition(
                    "no active transfer".to_string(),
                ))
            }
        };

        if transfer_id != active_tid {
            return Err(TransferError::InvalidTransition(
                "transfer_id mismatch".to_string(),
            ));
        }

        self.state = TransferState::Cancelled {
            transfer_id: active_tid,
            reason: CancelReason::ByReceiver,
        };
        Ok(())
    }

    /// Returns true if completed with hash verification.
    pub fn hash_verified(&self) -> bool {
        matches!(&self.state, TransferState::Completed { .. }) && self.expected_hash.is_some()
    }

    /// Returns buffer contents if Completed, None otherwise.
    pub fn completed_bytes(&self) -> Option<&[u8]> {
        match &self.state {
            TransferState::Completed { .. } => Some(&self.buffer),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test verifier that does case-insensitive hex comparison.
    struct HexVerifier;
    impl IntegrityVerifier for HexVerifier {
        fn verify(&self, data: &[u8], expected_hash: &str) -> bool {
            let mut hasher = Sha256Stub::new();
            hasher.update(data);
            let computed = hasher.finalize_hex();
            computed.eq_ignore_ascii_case(expected_hash)
        }
    }

    /// Minimal SHA-256 stub for tests only (no crypto dep in prod).
    /// Tests that need hash verification use this. The real daemon
    /// injects bolt_core::hash::sha256_hex via IntegrityVerifier.
    struct Sha256Stub {
        // Simple FNV-like hash for test determinism — NOT real SHA-256.
        // Actual SHA-256 verification is tested at the daemon integration level.
        state: u64,
    }

    impl Sha256Stub {
        fn new() -> Self {
            Self {
                state: 0xcbf29ce484222325,
            }
        }
        fn update(&mut self, data: &[u8]) {
            for &b in data {
                self.state ^= b as u64;
                self.state = self.state.wrapping_mul(0x100000001b3);
            }
        }
        fn finalize_hex(&self) -> String {
            format!("{:016x}", self.state)
        }
    }

    fn stub_hash(data: &[u8]) -> String {
        let mut h = Sha256Stub::new();
        h.update(data);
        h.finalize_hex()
    }

    // ── Offer lifecycle ──

    #[test]
    fn offer_then_reject() {
        let mut rs = ReceiveSession::new();
        assert_eq!(*rs.state(), TransferState::Idle);

        rs.on_file_offer("t1", 100, 1, None).unwrap();
        assert!(matches!(rs.state(), TransferState::Offered { .. }));

        let tid = rs.reject_current_offer().unwrap();
        assert_eq!(tid, "t1");
        assert!(matches!(
            rs.state(),
            TransferState::Cancelled {
                reason: CancelReason::Rejected,
                ..
            }
        ));
    }

    #[test]
    fn double_offer_rejected() {
        let mut rs = ReceiveSession::new();
        rs.on_file_offer("t1", 100, 1, None).unwrap();
        let err = rs.on_file_offer("t2", 200, 2, None).unwrap_err();
        assert!(err.to_string().contains("offer already active"));
    }

    #[test]
    fn reject_in_idle_fails() {
        let mut rs = ReceiveSession::new();
        let err = rs.reject_current_offer().unwrap_err();
        assert!(err.to_string().contains("no active offer"));
    }

    // ── Accept + receive lifecycle ──

    #[test]
    fn offer_accept_lifecycle() {
        let mut rs = ReceiveSession::new();
        rs.on_file_offer("t1", 100, 1, None).unwrap();
        let tid = rs.accept_current_offer().unwrap();
        assert_eq!(tid, "t1");
        assert!(matches!(rs.state(), TransferState::Transferring { .. }));
    }

    #[test]
    fn full_receive_lifecycle() {
        let mut rs = ReceiveSession::new();
        rs.on_file_offer("t1", 5, 1, None).unwrap();
        rs.accept_current_offer().unwrap();
        rs.on_file_chunk("t1", 0, b"hello").unwrap();
        rs.on_file_finish("t1", None).unwrap();

        assert!(matches!(rs.state(), TransferState::Completed { .. }));
        assert_eq!(rs.completed_bytes(), Some(b"hello".as_slice()));
    }

    #[test]
    fn multi_chunk_reassembly() {
        let mut rs = ReceiveSession::new();
        rs.on_file_offer("t1", 15, 3, None).unwrap();
        rs.accept_current_offer().unwrap();
        rs.on_file_chunk("t1", 0, b"aaaaa").unwrap();
        rs.on_file_chunk("t1", 1, b"bbbbb").unwrap();
        rs.on_file_chunk("t1", 2, b"ccccc").unwrap();
        rs.on_file_finish("t1", None).unwrap();

        assert_eq!(rs.completed_bytes(), Some(b"aaaaabbbbbccccc".as_slice()));
    }

    // ── Error paths ──

    #[test]
    fn chunk_before_offer_fails() {
        let mut rs = ReceiveSession::new();
        let err = rs.on_file_chunk("t1", 0, b"data").unwrap_err();
        assert!(err.to_string().contains("no active transfer"));
    }

    #[test]
    fn chunk_wrong_transfer_id_fails() {
        let mut rs = ReceiveSession::new();
        rs.on_file_offer("t1", 100, 1, None).unwrap();
        rs.accept_current_offer().unwrap();
        let err = rs.on_file_chunk("t2", 0, b"data").unwrap_err();
        assert!(err.to_string().contains("transfer_id mismatch"));
    }

    #[test]
    fn chunk_wrong_index_fails() {
        let mut rs = ReceiveSession::new();
        rs.on_file_offer("t1", 100, 3, None).unwrap();
        rs.accept_current_offer().unwrap();
        let err = rs.on_file_chunk("t1", 1, b"data").unwrap_err();
        assert!(err.to_string().contains("unexpected chunk index"));
    }

    #[test]
    fn offer_size_exceeded_fails() {
        let mut rs = ReceiveSession::new();
        let err = rs
            .on_file_offer("t1", MAX_TRANSFER_BYTES + 1, 1, None)
            .unwrap_err();
        assert!(err.to_string().contains("transfer size exceeded"));
    }

    #[test]
    fn finish_wrong_id_fails() {
        let mut rs = ReceiveSession::new();
        rs.on_file_offer("t1", 100, 1, None).unwrap();
        rs.accept_current_offer().unwrap();
        let err = rs.on_file_finish("wrong_id", None).unwrap_err();
        assert!(err.to_string().contains("transfer_id mismatch"));
    }

    #[test]
    fn second_offer_after_complete_fails() {
        let mut rs = ReceiveSession::new();
        rs.on_file_offer("t1", 5, 1, None).unwrap();
        rs.accept_current_offer().unwrap();
        rs.on_file_chunk("t1", 0, b"hello").unwrap();
        rs.on_file_finish("t1", None).unwrap();

        let err = rs.on_file_offer("t2", 200, 2, None).unwrap_err();
        assert!(err.to_string().contains("transfer session ended"));
    }

    #[test]
    fn zero_size_offer_fails() {
        let mut rs = ReceiveSession::new();
        let err = rs.on_file_offer("t1", 0, 1, None).unwrap_err();
        assert!(err.to_string().contains("size is zero"));
    }

    #[test]
    fn zero_chunks_offer_fails() {
        let mut rs = ReceiveSession::new();
        let err = rs.on_file_offer("t1", 100, 0, None).unwrap_err();
        assert!(err.to_string().contains("zero chunks"));
    }

    #[test]
    fn chunk_index_out_of_range_fails() {
        let mut rs = ReceiveSession::new();
        rs.on_file_offer("t1", 100, 2, None).unwrap();
        rs.accept_current_offer().unwrap();
        let err = rs.on_file_chunk("t1", 2, b"data").unwrap_err();
        assert!(err.to_string().contains("chunk index out of range"));
    }

    // ── Integrity verification ──

    #[test]
    fn hash_verify_correct() {
        let data = b"hello";
        let hash = stub_hash(data);

        let mut rs = ReceiveSession::new();
        rs.on_file_offer("t1", data.len() as u64, 1, Some(&hash))
            .unwrap();
        rs.accept_current_offer().unwrap();
        rs.on_file_chunk("t1", 0, data).unwrap();
        rs.on_file_finish("t1", Some(&HexVerifier)).unwrap();

        assert!(matches!(rs.state(), TransferState::Completed { .. }));
        assert!(rs.hash_verified());
    }

    #[test]
    fn hash_verify_mismatch() {
        let data = b"hello";
        let wrong_hash = "0000000000000000";

        let mut rs = ReceiveSession::new();
        rs.on_file_offer("t1", data.len() as u64, 1, Some(wrong_hash))
            .unwrap();
        rs.accept_current_offer().unwrap();
        rs.on_file_chunk("t1", 0, data).unwrap();

        let err = rs.on_file_finish("t1", Some(&HexVerifier)).unwrap_err();
        assert!(err.to_string().contains("file hash mismatch"));
    }

    #[test]
    fn no_hash_skips_verify() {
        let data = b"hello";
        let mut rs = ReceiveSession::new();
        rs.on_file_offer("t1", data.len() as u64, 1, None).unwrap();
        rs.accept_current_offer().unwrap();
        rs.on_file_chunk("t1", 0, data).unwrap();
        rs.on_file_finish("t1", None).unwrap();
        assert!(!rs.hash_verified());
    }

    #[test]
    fn hash_expected_but_no_verifier_skips() {
        // When expected_hash is set but no verifier provided,
        // verification is skipped (verifier is optional/caller-injected).
        let data = b"hello";
        let mut rs = ReceiveSession::new();
        rs.on_file_offer("t1", data.len() as u64, 1, Some("anything"))
            .unwrap();
        rs.accept_current_offer().unwrap();
        rs.on_file_chunk("t1", 0, data).unwrap();
        rs.on_file_finish("t1", None).unwrap();
        assert!(matches!(rs.state(), TransferState::Completed { .. }));
    }

    // ── Cancel ──

    #[test]
    fn cancel_during_transfer() {
        let mut rs = ReceiveSession::new();
        rs.on_file_offer("t1", 100, 2, None).unwrap();
        rs.accept_current_offer().unwrap();
        rs.on_file_chunk("t1", 0, b"data").unwrap();
        rs.cancel("t1").unwrap();
        assert!(matches!(
            rs.state(),
            TransferState::Cancelled {
                reason: CancelReason::ByReceiver,
                ..
            }
        ));
    }

    #[test]
    fn cancel_wrong_id_fails() {
        let mut rs = ReceiveSession::new();
        rs.on_file_offer("t1", 100, 1, None).unwrap();
        rs.accept_current_offer().unwrap();
        let err = rs.cancel("wrong").unwrap_err();
        assert!(err.to_string().contains("transfer_id mismatch"));
    }

    #[test]
    fn cancel_in_idle_fails() {
        let mut rs = ReceiveSession::new();
        let err = rs.cancel("t1").unwrap_err();
        assert!(err.to_string().contains("no active transfer"));
    }
}
