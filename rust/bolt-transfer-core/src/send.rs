//! Send-side transfer state machine.
//!
//! Extracted from bolt-daemon/src/transfer.rs (SendSession).
//! Aligned to PROTOCOL.md §9 state names.
//!
//! State flow:
//!   Idle → Offered → Accepted → Transferring ↔ Paused → Completed
//!                                    |                      |
//!                                  Cancelled ←──────── Cancelled
//!
//! No crypto dependencies. Hash computation is caller-provided.
//! Transfer ID generation is caller-provided.

use crate::error::TransferError;
use crate::state::{CancelReason, TransferState};

/// Default chunk size in bytes (16 KiB). Matches bolt-core DEFAULT_CHUNK_SIZE.
pub const DEFAULT_CHUNK_SIZE: usize = 16_384;

/// Metadata returned by `begin_send()`.
#[derive(Debug)]
pub struct SendOffer {
    pub transfer_id: String,
    pub filename: String,
    pub size: u64,
    pub total_chunks: u32,
    pub chunk_size: u32,
    /// Caller-computed file hash, if provided.
    pub file_hash: Option<String>,
}

/// Single chunk returned by `next_chunk()`.
#[derive(Debug)]
pub struct SendChunk {
    pub transfer_id: String,
    pub chunk_index: u32,
    pub total_chunks: u32,
    pub data: Vec<u8>,
}

/// Send-side transfer session (§9 state machine).
///
/// Enforces: Idle → Offered → Accepted → Transferring → Completed.
/// Also supports: Offered/Transferring/Paused → Cancelled.
/// Sender-side pause: Transferring ↔ Paused.
pub struct SendSession {
    state: TransferState,
    payload: Vec<u8>,
    chunk_size: usize,
    cursor: usize,
    total_chunks: u32,
    next_chunk_index: u32,
}

impl Default for SendSession {
    fn default() -> Self {
        Self::new()
    }
}

impl SendSession {
    pub fn new() -> Self {
        Self {
            state: TransferState::Idle,
            payload: Vec::new(),
            chunk_size: DEFAULT_CHUNK_SIZE,
            cursor: 0,
            total_chunks: 0,
            next_chunk_index: 0,
        }
    }

    /// Create with a custom chunk size.
    pub fn with_chunk_size(chunk_size: usize) -> Self {
        assert!(chunk_size > 0, "chunk_size must be > 0");
        Self {
            chunk_size,
            ..Self::new()
        }
    }

    /// Current state.
    pub fn state(&self) -> &TransferState {
        &self.state
    }

    /// Begin an outbound transfer. Must be Idle.
    ///
    /// `transfer_id`: caller-generated (core does not generate IDs).
    /// `file_hash`: caller-computed hash, or None if bolt.file-hash not negotiated.
    pub fn begin_send(
        &mut self,
        transfer_id: &str,
        payload: Vec<u8>,
        filename: &str,
        file_hash: Option<String>,
    ) -> Result<SendOffer, TransferError> {
        if !matches!(self.state, TransferState::Idle) {
            return Err(TransferError::InvalidTransition(
                "outbound transfer already active".to_string(),
            ));
        }

        if payload.is_empty() {
            return Err(TransferError::InvalidTransition(
                "empty payload".to_string(),
            ));
        }

        let size = payload.len() as u64;
        let total_chunks = payload.len().div_ceil(self.chunk_size) as u32;

        self.payload = payload;
        self.cursor = 0;
        self.total_chunks = total_chunks;
        self.next_chunk_index = 0;
        self.state = TransferState::Offered {
            transfer_id: transfer_id.to_string(),
        };

        Ok(SendOffer {
            transfer_id: transfer_id.to_string(),
            filename: filename.to_string(),
            size,
            total_chunks,
            chunk_size: self.chunk_size as u32,
            file_hash,
        })
    }

    /// Receiver accepted our offer. Transitions Offered → Transferring.
    pub fn on_accept(&mut self, transfer_id: &str) -> Result<(), TransferError> {
        match &self.state {
            TransferState::Offered {
                transfer_id: tid, ..
            } => {
                if transfer_id != tid {
                    return Err(TransferError::InvalidTransition(
                        "transfer_id mismatch".to_string(),
                    ));
                }
                let tid = tid.clone();
                self.cursor = 0;
                self.next_chunk_index = 0;
                self.state = TransferState::Transferring { transfer_id: tid };
                Ok(())
            }
            _ => Err(TransferError::InvalidTransition(
                "not awaiting accept".to_string(),
            )),
        }
    }

    /// Cancel. Transitions Offered/Transferring/Paused → Cancelled.
    pub fn on_cancel(&mut self, transfer_id: &str) -> Result<(), TransferError> {
        match &self.state {
            TransferState::Offered {
                transfer_id: tid, ..
            }
            | TransferState::Transferring { transfer_id: tid }
            | TransferState::Paused { transfer_id: tid } => {
                if transfer_id != tid {
                    return Err(TransferError::InvalidTransition(
                        "transfer_id mismatch".to_string(),
                    ));
                }
                self.state = TransferState::Cancelled {
                    transfer_id: transfer_id.to_string(),
                    reason: CancelReason::ByReceiver,
                };
                Ok(())
            }
            _ => Err(TransferError::InvalidTransition(
                "no active outbound transfer".to_string(),
            )),
        }
    }

    /// Pause sending. Transitions Transferring → Paused. Idempotent if already Paused.
    pub fn on_pause(&mut self, transfer_id: &str) -> Result<(), TransferError> {
        match &self.state {
            TransferState::Transferring { transfer_id: tid } => {
                if transfer_id != tid {
                    return Err(TransferError::InvalidTransition(
                        "transfer_id mismatch".to_string(),
                    ));
                }
                let tid = tid.clone();
                self.state = TransferState::Paused { transfer_id: tid };
                Ok(())
            }
            TransferState::Paused { .. } => {
                // Idempotent: already paused.
                Ok(())
            }
            _ => Err(TransferError::InvalidTransition(
                "not in sending state".to_string(),
            )),
        }
    }

    /// Resume sending. Transitions Paused → Transferring. Idempotent if already Transferring.
    pub fn on_resume(&mut self, transfer_id: &str) -> Result<(), TransferError> {
        match &self.state {
            TransferState::Paused { transfer_id: tid } => {
                if transfer_id != tid {
                    return Err(TransferError::InvalidTransition(
                        "transfer_id mismatch".to_string(),
                    ));
                }
                let tid = tid.clone();
                self.state = TransferState::Transferring { transfer_id: tid };
                Ok(())
            }
            TransferState::Transferring { .. } => {
                // Idempotent: already sending.
                Ok(())
            }
            _ => Err(TransferError::InvalidTransition("not paused".to_string())),
        }
    }

    /// Returns true if in Transferring state with chunks remaining.
    pub fn is_send_active(&self) -> bool {
        matches!(self.state, TransferState::Transferring { .. }) && self.cursor < self.payload.len()
    }

    /// Yield next chunk. Must be Transferring. Returns None when all chunks yielded.
    pub fn next_chunk(&mut self) -> Result<Option<SendChunk>, TransferError> {
        let tid = match &self.state {
            TransferState::Transferring { transfer_id } => transfer_id.clone(),
            _ => {
                return Err(TransferError::InvalidTransition(
                    "not in sending state".to_string(),
                ))
            }
        };

        if self.cursor >= self.payload.len() {
            return Ok(None);
        }

        let end = std::cmp::min(self.cursor + self.chunk_size, self.payload.len());
        let data = self.payload[self.cursor..end].to_vec();
        let chunk_index = self.next_chunk_index;

        self.cursor = end;
        self.next_chunk_index += 1;

        Ok(Some(SendChunk {
            transfer_id: tid,
            chunk_index,
            total_chunks: self.total_chunks,
            data,
        }))
    }

    /// Finalize transfer. Must be Transferring with all chunks yielded.
    /// Transitions Transferring → Completed. Returns transfer_id.
    ///
    /// Completion-race guard: finish() only succeeds in Transferring state.
    /// A concurrent cancel transitions to Cancelled, blocking late finish.
    pub fn finish(&mut self) -> Result<String, TransferError> {
        let tid = match &self.state {
            TransferState::Transferring { transfer_id } => transfer_id.clone(),
            _ => {
                return Err(TransferError::InvalidTransition(
                    "not in sending state".to_string(),
                ))
            }
        };

        if self.cursor < self.payload.len() {
            return Err(TransferError::InvalidTransition(
                "not all chunks yielded".to_string(),
            ));
        }

        self.state = TransferState::Completed {
            transfer_id: tid.clone(),
        };
        Ok(tid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Full lifecycle ──

    #[test]
    fn send_lifecycle_complete() {
        let mut ss = SendSession::new();
        assert_eq!(*ss.state(), TransferState::Idle);

        let payload = b"hello world, this is a test payload for send lifecycle".to_vec();
        let offer = ss
            .begin_send("tx-1", payload.clone(), "test.txt", Some("hash123".into()))
            .unwrap();
        assert!(matches!(ss.state(), TransferState::Offered { .. }));
        assert_eq!(offer.file_hash, Some("hash123".into()));
        assert_eq!(offer.size, payload.len() as u64);

        ss.on_accept("tx-1").unwrap();
        assert!(matches!(ss.state(), TransferState::Transferring { .. }));

        let mut reassembled = Vec::new();
        while let Some(chunk) = ss.next_chunk().unwrap() {
            reassembled.extend_from_slice(&chunk.data);
        }
        assert_eq!(reassembled, payload);

        let tid = ss.finish().unwrap();
        assert_eq!(tid, "tx-1");
        assert!(matches!(ss.state(), TransferState::Completed { .. }));
    }

    #[test]
    fn send_lifecycle_no_hash() {
        let mut ss = SendSession::new();
        let payload = b"no hash test".to_vec();
        let offer = ss
            .begin_send("tx-2", payload.clone(), "no_hash.txt", None)
            .unwrap();
        assert!(offer.file_hash.is_none());

        ss.on_accept("tx-2").unwrap();
        let mut reassembled = Vec::new();
        while let Some(chunk) = ss.next_chunk().unwrap() {
            reassembled.extend_from_slice(&chunk.data);
        }
        assert_eq!(reassembled, payload);
        ss.finish().unwrap();
        assert!(matches!(ss.state(), TransferState::Completed { .. }));
    }

    // ── Cancel paths ──

    #[test]
    fn cancel_before_accept() {
        let mut ss = SendSession::new();
        ss.begin_send("tx-1", b"data".to_vec(), "f.txt", None)
            .unwrap();
        ss.on_cancel("tx-1").unwrap();
        assert!(matches!(ss.state(), TransferState::Cancelled { .. }));
    }

    #[test]
    fn cancel_during_send() {
        let mut ss = SendSession::new();
        let payload = vec![0u8; 32768]; // > 1 chunk
        ss.begin_send("tx-1", payload, "big.bin", None).unwrap();
        ss.on_accept("tx-1").unwrap();
        ss.next_chunk().unwrap(); // consume one
        ss.on_cancel("tx-1").unwrap();
        assert!(matches!(ss.state(), TransferState::Cancelled { .. }));
    }

    #[test]
    fn completion_race_guard_cancel_blocks_finish() {
        let mut ss = SendSession::new();
        let payload = vec![0u8; 100];
        ss.begin_send("tx-1", payload, "f.bin", None).unwrap();
        ss.on_accept("tx-1").unwrap();
        while ss.next_chunk().unwrap().is_some() {}
        // Cancel after all chunks yielded
        ss.on_cancel("tx-1").unwrap();
        // finish() must fail — cancel happened first
        let err = ss.finish().unwrap_err();
        assert!(err.to_string().contains("not in sending state"));
    }

    // ── Error paths ──

    #[test]
    fn accept_wrong_transfer_id() {
        let mut ss = SendSession::new();
        ss.begin_send("tx-1", b"data".to_vec(), "f.txt", None)
            .unwrap();
        let err = ss.on_accept("wrong-id").unwrap_err();
        assert!(err.to_string().contains("transfer_id mismatch"));
    }

    #[test]
    fn cancel_wrong_transfer_id() {
        let mut ss = SendSession::new();
        ss.begin_send("tx-1", b"data".to_vec(), "f.txt", None)
            .unwrap();
        let err = ss.on_cancel("wrong-id").unwrap_err();
        assert!(err.to_string().contains("transfer_id mismatch"));
    }

    #[test]
    fn send_while_not_idle() {
        let mut ss = SendSession::new();
        ss.begin_send("tx-1", b"data".to_vec(), "f.txt", None)
            .unwrap();
        let err = ss
            .begin_send("tx-2", b"more".to_vec(), "g.txt", None)
            .unwrap_err();
        assert!(err.to_string().contains("outbound transfer already active"));
    }

    #[test]
    fn next_chunk_before_accept() {
        let mut ss = SendSession::new();
        ss.begin_send("tx-1", b"data".to_vec(), "f.txt", None)
            .unwrap();
        let err = ss.next_chunk().unwrap_err();
        assert!(err.to_string().contains("not in sending state"));
    }

    #[test]
    fn finish_before_all_chunks() {
        let mut ss = SendSession::new();
        let payload = vec![0u8; 32768]; // 2 chunks
        ss.begin_send("tx-1", payload, "big.bin", None).unwrap();
        ss.on_accept("tx-1").unwrap();
        ss.next_chunk().unwrap(); // only 1
        let err = ss.finish().unwrap_err();
        assert!(err.to_string().contains("not all chunks yielded"));
    }

    #[test]
    fn empty_payload_fails() {
        let mut ss = SendSession::new();
        let err = ss
            .begin_send("tx-1", vec![], "empty.txt", None)
            .unwrap_err();
        assert!(err.to_string().contains("empty payload"));
    }

    // ── Pause/Resume ──

    #[test]
    fn pause_during_send() {
        let mut ss = SendSession::new();
        let payload = vec![0u8; 32768];
        ss.begin_send("tx-1", payload, "big.bin", None).unwrap();
        ss.on_accept("tx-1").unwrap();
        ss.next_chunk().unwrap();

        ss.on_pause("tx-1").unwrap();
        assert!(matches!(ss.state(), TransferState::Paused { .. }));

        // next_chunk should fail when paused
        let err = ss.next_chunk().unwrap_err();
        assert!(err.to_string().contains("not in sending state"));
    }

    #[test]
    fn pause_then_resume_continues() {
        let mut ss = SendSession::new();
        let payload = vec![0xAB; 40960]; // 3 chunks
        ss.begin_send("tx-1", payload.clone(), "multi.bin", None)
            .unwrap();
        ss.on_accept("tx-1").unwrap();

        let c0 = ss.next_chunk().unwrap().unwrap();
        assert_eq!(c0.chunk_index, 0);

        ss.on_pause("tx-1").unwrap();
        assert!(!ss.is_send_active());

        ss.on_resume("tx-1").unwrap();
        assert!(ss.is_send_active());

        // Continue from chunk 1, not 0
        let c1 = ss.next_chunk().unwrap().unwrap();
        assert_eq!(c1.chunk_index, 1);
        let c2 = ss.next_chunk().unwrap().unwrap();
        assert_eq!(c2.chunk_index, 2);
        assert!(ss.next_chunk().unwrap().is_none());

        let mut reassembled = Vec::new();
        reassembled.extend_from_slice(&c0.data);
        reassembled.extend_from_slice(&c1.data);
        reassembled.extend_from_slice(&c2.data);
        assert_eq!(reassembled, payload);
        ss.finish().unwrap();
    }

    #[test]
    fn pause_then_cancel() {
        let mut ss = SendSession::new();
        ss.begin_send("tx-1", vec![0u8; 32768], "f.bin", None)
            .unwrap();
        ss.on_accept("tx-1").unwrap();
        ss.next_chunk().unwrap();
        ss.on_pause("tx-1").unwrap();
        ss.on_cancel("tx-1").unwrap();
        assert!(matches!(ss.state(), TransferState::Cancelled { .. }));
    }

    #[test]
    fn resume_when_not_paused_idempotent() {
        let mut ss = SendSession::new();
        ss.begin_send("tx-1", vec![0u8; 100], "f.bin", None)
            .unwrap();
        ss.on_accept("tx-1").unwrap();
        ss.on_resume("tx-1").unwrap(); // already Transferring
        assert!(matches!(ss.state(), TransferState::Transferring { .. }));
    }

    #[test]
    fn pause_when_not_sending() {
        let mut ss = SendSession::new();
        let err = ss.on_pause("tid").unwrap_err();
        assert!(err.to_string().contains("not in sending state"));
    }

    #[test]
    fn resume_when_idle() {
        let mut ss = SendSession::new();
        let err = ss.on_resume("tid").unwrap_err();
        assert!(err.to_string().contains("not paused"));
    }

    #[test]
    fn pause_wrong_transfer_id() {
        let mut ss = SendSession::new();
        ss.begin_send("tx-1", vec![0u8; 100], "f.bin", None)
            .unwrap();
        ss.on_accept("tx-1").unwrap();
        let err = ss.on_pause("wrong-id").unwrap_err();
        assert!(err.to_string().contains("transfer_id mismatch"));
    }

    #[test]
    fn resume_wrong_transfer_id() {
        let mut ss = SendSession::new();
        ss.begin_send("tx-1", vec![0u8; 100], "f.bin", None)
            .unwrap();
        ss.on_accept("tx-1").unwrap();
        ss.on_pause("tx-1").unwrap();
        let err = ss.on_resume("wrong-id").unwrap_err();
        assert!(err.to_string().contains("transfer_id mismatch"));
    }

    #[test]
    fn repeated_pause_idempotent() {
        let mut ss = SendSession::new();
        ss.begin_send("tx-1", vec![0u8; 100], "f.bin", None)
            .unwrap();
        ss.on_accept("tx-1").unwrap();
        ss.on_pause("tx-1").unwrap();
        ss.on_pause("tx-1").unwrap(); // second pause
        assert!(matches!(ss.state(), TransferState::Paused { .. }));
    }

    #[test]
    fn is_send_active_states() {
        let mut ss = SendSession::new();
        assert!(!ss.is_send_active()); // Idle

        ss.begin_send("tx-1", vec![0u8; 100], "f.bin", None)
            .unwrap();
        assert!(!ss.is_send_active()); // Offered

        ss.on_accept("tx-1").unwrap();
        assert!(ss.is_send_active()); // Transferring with chunks

        ss.on_pause("tx-1").unwrap();
        assert!(!ss.is_send_active()); // Paused

        ss.on_resume("tx-1").unwrap();
        assert!(ss.is_send_active()); // Transferring again

        while ss.next_chunk().unwrap().is_some() {}
        assert!(!ss.is_send_active()); // Transferring but exhausted

        ss.finish().unwrap();
        assert!(!ss.is_send_active()); // Completed
    }

    #[test]
    fn finish_rejects_paused() {
        let mut ss = SendSession::new();
        ss.begin_send("tx-1", vec![0u8; 100], "f.bin", None)
            .unwrap();
        ss.on_accept("tx-1").unwrap();
        while ss.next_chunk().unwrap().is_some() {}
        ss.on_pause("tx-1").unwrap();
        let err = ss.finish().unwrap_err();
        assert!(err.to_string().contains("not in sending state"));
    }

    #[test]
    fn chunk_size_correctness() {
        let mut ss = SendSession::new();
        let payload = vec![0xAB; 40960]; // 2.5 chunks at 16384
        let offer = ss
            .begin_send("tx-1", payload.clone(), "multi.bin", None)
            .unwrap();
        assert_eq!(offer.total_chunks, 3);
        assert_eq!(offer.chunk_size, DEFAULT_CHUNK_SIZE as u32);

        ss.on_accept("tx-1").unwrap();

        let mut reassembled = Vec::new();
        let mut chunk_count = 0u32;
        while let Some(chunk) = ss.next_chunk().unwrap() {
            assert!(chunk.data.len() <= DEFAULT_CHUNK_SIZE);
            assert_eq!(chunk.chunk_index, chunk_count);
            assert_eq!(chunk.total_chunks, 3);
            reassembled.extend_from_slice(&chunk.data);
            chunk_count += 1;
        }
        assert_eq!(chunk_count, 3);
        assert_eq!(reassembled, payload);
        ss.finish().unwrap();
    }

    #[test]
    fn custom_chunk_size() {
        let mut ss = SendSession::with_chunk_size(10);
        let payload = vec![0xFF; 25]; // 3 chunks: 10, 10, 5
        let offer = ss.begin_send("tx-1", payload, "small.bin", None).unwrap();
        assert_eq!(offer.total_chunks, 3);
        assert_eq!(offer.chunk_size, 10);
    }
}
