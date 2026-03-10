//! Replay rejection — (transfer_id, generation, chain_index) guard (§11).
//!
//! ORDER-BTR: chain_index must equal expected_next_index (no gaps).
//! REPLAY-BTR: (transfer_id, ratchet_generation, chain_index) triple
//!   prevents cross-generation replay.

use std::collections::HashSet;

use crate::errors::BtrError;

/// Replay guard tracking seen (transfer_id, generation, chain_index) triples.
///
/// Enforces ORDER-BTR: chain_index must be strictly monotonic per transfer
/// (no skipped-key buffer). Also rejects cross-generation replay.
pub struct ReplayGuard {
    /// Set of seen triples: (transfer_id, generation, chain_index).
    seen: HashSet<([u8; 16], u32, u32)>,
    /// Expected next chain_index per active (transfer_id, generation).
    expected: Option<ExpectedState>,
}

struct ExpectedState {
    transfer_id: [u8; 16],
    generation: u32,
    next_index: u32,
}

impl ReplayGuard {
    /// Create a new empty replay guard.
    pub fn new() -> Self {
        Self {
            seen: HashSet::new(),
            expected: None,
        }
    }

    /// Begin tracking a new transfer at the given generation.
    /// Resets the expected chain_index to 0.
    pub fn begin_transfer(&mut self, transfer_id: [u8; 16], generation: u32) {
        self.expected = Some(ExpectedState {
            transfer_id,
            generation,
            next_index: 0,
        });
    }

    /// Check and record a (transfer_id, generation, chain_index) triple.
    ///
    /// Returns `Ok(())` if accepted, or `Err` with the appropriate BTR error:
    /// - `RatchetStateError` if generation doesn't match expected
    /// - `RatchetChainError` if chain_index != expected next
    /// - `RatchetChainError` if duplicate (replay)
    pub fn check(
        &mut self,
        transfer_id: &[u8; 16],
        generation: u32,
        chain_index: u32,
    ) -> Result<(), BtrError> {
        let expected = self.expected.as_ref().ok_or_else(|| {
            BtrError::RatchetStateError("no active transfer in replay guard".into())
        })?;

        // Check transfer_id matches
        if transfer_id != &expected.transfer_id {
            return Err(BtrError::RatchetStateError(format!(
                "transfer_id mismatch: expected {:02x?}, got {:02x?}",
                &expected.transfer_id[..4],
                &transfer_id[..4]
            )));
        }

        // Check generation matches
        if generation != expected.generation {
            return Err(BtrError::RatchetStateError(format!(
                "generation mismatch: expected {}, got {}",
                expected.generation, generation
            )));
        }

        // ORDER-BTR: chain_index must equal expected next (no gaps)
        if chain_index != expected.next_index {
            return Err(BtrError::RatchetChainError(format!(
                "chain_index out of order: expected {}, got {}",
                expected.next_index, chain_index
            )));
        }

        // REPLAY-BTR: check for duplicate triple
        let triple = (*transfer_id, generation, chain_index);
        if !self.seen.insert(triple) {
            return Err(BtrError::RatchetChainError(format!(
                "replay detected: generation={}, chain_index={}",
                generation, chain_index
            )));
        }

        // Advance expected index
        self.expected.as_mut().unwrap().next_index = chain_index + 1;
        Ok(())
    }

    /// End tracking for the current transfer. Clears expected state
    /// but retains seen set for cross-transfer replay detection.
    pub fn end_transfer(&mut self) {
        self.expected = None;
    }

    /// Full reset — clears all state. Used on disconnect.
    pub fn reset(&mut self) {
        self.seen.clear();
        self.expected = None;
    }
}

impl Default for ReplayGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tid(val: u8) -> [u8; 16] {
        [val; 16]
    }

    #[test]
    fn accept_sequential_chunks() {
        let mut guard = ReplayGuard::new();
        guard.begin_transfer(tid(1), 0);
        assert!(guard.check(&tid(1), 0, 0).is_ok());
        assert!(guard.check(&tid(1), 0, 1).is_ok());
        assert!(guard.check(&tid(1), 0, 2).is_ok());
    }

    #[test]
    fn reject_duplicate_index() {
        let mut guard = ReplayGuard::new();
        guard.begin_transfer(tid(1), 0);
        guard.check(&tid(1), 0, 0).unwrap();
        let err = guard.check(&tid(1), 0, 0).unwrap_err();
        assert!(matches!(err, BtrError::RatchetChainError(_)));
    }

    #[test]
    fn reject_skipped_index() {
        let mut guard = ReplayGuard::new();
        guard.begin_transfer(tid(1), 0);
        let err = guard.check(&tid(1), 0, 1).unwrap_err(); // expected 0
        assert!(matches!(err, BtrError::RatchetChainError(_)));
    }

    #[test]
    fn reject_wrong_generation() {
        let mut guard = ReplayGuard::new();
        guard.begin_transfer(tid(1), 0);
        let err = guard.check(&tid(1), 1, 0).unwrap_err();
        assert!(matches!(err, BtrError::RatchetStateError(_)));
    }

    #[test]
    fn reject_wrong_transfer_id() {
        let mut guard = ReplayGuard::new();
        guard.begin_transfer(tid(1), 0);
        let err = guard.check(&tid(2), 0, 0).unwrap_err();
        assert!(matches!(err, BtrError::RatchetStateError(_)));
    }

    #[test]
    fn reject_no_active_transfer() {
        let mut guard = ReplayGuard::new();
        let err = guard.check(&tid(1), 0, 0).unwrap_err();
        assert!(matches!(err, BtrError::RatchetStateError(_)));
    }

    #[test]
    fn end_transfer_clears_expected() {
        let mut guard = ReplayGuard::new();
        guard.begin_transfer(tid(1), 0);
        guard.check(&tid(1), 0, 0).unwrap();
        guard.end_transfer();
        let err = guard.check(&tid(1), 0, 1).unwrap_err();
        assert!(matches!(err, BtrError::RatchetStateError(_)));
    }

    #[test]
    fn reset_clears_all() {
        let mut guard = ReplayGuard::new();
        guard.begin_transfer(tid(1), 0);
        guard.check(&tid(1), 0, 0).unwrap();
        guard.reset();
        // After reset, can start fresh
        guard.begin_transfer(tid(1), 0);
        assert!(guard.check(&tid(1), 0, 0).is_ok());
    }

    #[test]
    fn cross_transfer_new_generation() {
        let mut guard = ReplayGuard::new();
        // First transfer
        guard.begin_transfer(tid(1), 0);
        guard.check(&tid(1), 0, 0).unwrap();
        guard.end_transfer();
        // Second transfer with new generation
        guard.begin_transfer(tid(2), 1);
        assert!(guard.check(&tid(2), 1, 0).is_ok());
    }
}
