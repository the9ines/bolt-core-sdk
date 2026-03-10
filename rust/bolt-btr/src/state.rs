//! BTR session/transfer/chain state — §16.5 key material lifecycle.
//!
//! All secret-holding structs zeroize on drop via `zeroize` crate.
//! Memory-only policy: MUST NOT persist to disk, log, or non-volatile storage.

use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::constants::BTR_KEY_LENGTH;
use crate::encrypt::{btr_open, btr_seal};
use crate::errors::BtrError;
use crate::key_schedule::{chain_advance, derive_session_root, derive_transfer_root};
use crate::ratchet::{derive_ratcheted_session_root, RatchetKeypair};
use crate::replay::ReplayGuard;

/// BTR engine — manages session-level ratchet state.
///
/// Owns the session_root_key, ratchet generation counter, and replay guard.
/// Create via `BtrEngine::new()` after handshake completes.
pub struct BtrEngine {
    session_root_key: SecretKey32,
    ratchet_generation: u32,
    local_ratchet_keypair: Option<RatchetKeypair>,
    replay_guard: ReplayGuard,
}

/// 32-byte secret key wrapper with zeroize-on-drop.
#[derive(Zeroize, ZeroizeOnDrop)]
struct SecretKey32 {
    bytes: [u8; BTR_KEY_LENGTH],
}

impl SecretKey32 {
    fn new(bytes: [u8; BTR_KEY_LENGTH]) -> Self {
        Self { bytes }
    }

    fn as_bytes(&self) -> &[u8; BTR_KEY_LENGTH] {
        &self.bytes
    }
}

impl BtrEngine {
    /// Create a new BTR engine from the ephemeral shared secret (§16.3).
    ///
    /// Called after HELLO handshake completes, when both peers have
    /// negotiated `bolt.transfer-ratchet-v1`.
    pub fn new(ephemeral_shared_secret: &[u8; 32]) -> Self {
        let session_root_key = derive_session_root(ephemeral_shared_secret);
        Self {
            session_root_key: SecretKey32::new(session_root_key),
            ratchet_generation: 0,
            local_ratchet_keypair: None,
            replay_guard: ReplayGuard::new(),
        }
    }

    /// Current ratchet generation (monotonically increasing per session).
    pub fn ratchet_generation(&self) -> u32 {
        self.ratchet_generation
    }

    /// Current session root key (for testing/vector generation only).
    #[cfg(any(test, feature = "vectors"))]
    pub fn session_root_key(&self) -> &[u8; BTR_KEY_LENGTH] {
        self.session_root_key.as_bytes()
    }

    /// Prepare to send a FILE_OFFER — generate local ratchet keypair and
    /// perform DH ratchet step with remote peer's ratchet public key.
    ///
    /// Returns a `BtrTransferContext` for encrypting chunks and the local
    /// ratchet public key to include in the envelope.
    ///
    /// Both the FILE_OFFER sender and FILE_ACCEPT responder call this
    /// (each with their own fresh keypair).
    pub fn begin_transfer_send(
        &mut self,
        transfer_id: &[u8; 16],
        remote_ratchet_pub: &[u8; 32],
    ) -> Result<(BtrTransferContext, [u8; 32]), BtrError> {
        let local_kp = RatchetKeypair::generate();
        let local_pub = local_kp.public_key;

        // DH ratchet step
        let dh_output = local_kp.diffie_hellman(remote_ratchet_pub);
        let new_srk = derive_ratcheted_session_root(self.session_root_key.as_bytes(), &dh_output);

        // Update session state
        self.session_root_key = SecretKey32::new(new_srk);
        self.ratchet_generation += 1;

        // Derive transfer root
        let transfer_root = derive_transfer_root(self.session_root_key.as_bytes(), transfer_id);

        // Set up replay guard for this transfer
        self.replay_guard
            .begin_transfer(*transfer_id, self.ratchet_generation);

        let ctx = BtrTransferContext {
            transfer_id: *transfer_id,
            generation: self.ratchet_generation,
            chain_key: SecretKey32::new(transfer_root),
            chain_index: 0,
        };

        Ok((ctx, local_pub))
    }

    /// Accept a transfer — perform DH ratchet step with the sender's
    /// ratchet public key from their FILE_OFFER envelope.
    ///
    /// Returns a `BtrTransferContext` for decrypting chunks and the local
    /// ratchet public key to include in the FILE_ACCEPT envelope.
    pub fn begin_transfer_receive(
        &mut self,
        transfer_id: &[u8; 16],
        remote_ratchet_pub: &[u8; 32],
    ) -> Result<(BtrTransferContext, [u8; 32]), BtrError> {
        // Same DH ratchet step as send side
        self.begin_transfer_send(transfer_id, remote_ratchet_pub)
    }

    /// Check a received chunk's replay/ordering status.
    pub fn check_replay(
        &mut self,
        transfer_id: &[u8; 16],
        generation: u32,
        chain_index: u32,
    ) -> Result<(), BtrError> {
        self.replay_guard
            .check(transfer_id, generation, chain_index)
    }

    /// End the current transfer's replay tracking.
    pub fn end_transfer(&mut self) {
        self.replay_guard.end_transfer();
    }

    /// Cleanup on disconnect — zeroize ALL BTR state.
    /// After this call, the engine is unusable (all keys zeroed).
    pub fn cleanup_disconnect(&mut self) {
        self.session_root_key.bytes.zeroize();
        self.ratchet_generation = 0;
        self.local_ratchet_keypair = None;
        self.replay_guard.reset();
    }
}

/// BTR transfer context — manages per-transfer chain state.
///
/// Created by `BtrEngine::begin_transfer_send/receive`.
/// Used for encrypting/decrypting chunks within a single transfer.
pub struct BtrTransferContext {
    transfer_id: [u8; 16],
    generation: u32,
    chain_key: SecretKey32,
    chain_index: u32,
}

impl BtrTransferContext {
    /// Transfer ID for this context.
    pub fn transfer_id(&self) -> &[u8; 16] {
        &self.transfer_id
    }

    /// Current ratchet generation.
    pub fn generation(&self) -> u32 {
        self.generation
    }

    /// Current chain index (next chunk to encrypt/decrypt).
    pub fn chain_index(&self) -> u32 {
        self.chain_index
    }

    /// Current chain key (for testing/vector generation only).
    #[cfg(any(test, feature = "vectors"))]
    pub fn chain_key(&self) -> &[u8; BTR_KEY_LENGTH] {
        self.chain_key.as_bytes()
    }

    /// Encrypt a chunk at the current chain position.
    ///
    /// Advances the chain: derives message_key and next_chain_key,
    /// encrypts plaintext via NaCl secretbox, zeroizes message_key,
    /// replaces chain_key with next_chain_key.
    ///
    /// Returns `(chain_index, sealed_bytes)`.
    pub fn seal_chunk(&mut self, plaintext: &[u8]) -> Result<(u32, Vec<u8>), BtrError> {
        let idx = self.chain_index;

        // Chain advance
        let mut advance_out = chain_advance(self.chain_key.as_bytes());

        // Encrypt
        let sealed = btr_seal(&advance_out.message_key, plaintext)?;

        // Zeroize message_key (single-use) — also done on Drop, but explicit
        advance_out.message_key.zeroize();

        // Update chain state
        self.chain_key = SecretKey32::new(advance_out.next_chain_key);
        self.chain_index += 1;

        Ok((idx, sealed))
    }

    /// Decrypt a chunk at the expected chain position.
    ///
    /// Same chain advance as seal_chunk — both peers derive identical keys.
    ///
    /// `expected_chain_index` must match the current chain position.
    pub fn open_chunk(
        &mut self,
        expected_chain_index: u32,
        sealed: &[u8],
    ) -> Result<Vec<u8>, BtrError> {
        if expected_chain_index != self.chain_index {
            return Err(BtrError::RatchetChainError(format!(
                "chain_index mismatch: expected {}, got {}",
                self.chain_index, expected_chain_index
            )));
        }

        // Chain advance (same derivation as sender)
        let mut advance_out = chain_advance(self.chain_key.as_bytes());

        // Decrypt
        let plaintext = btr_open(&advance_out.message_key, sealed)?;

        // Zeroize message_key
        advance_out.message_key.zeroize();

        // Update chain state
        self.chain_key = SecretKey32::new(advance_out.next_chain_key);
        self.chain_index += 1;

        Ok(plaintext)
    }

    /// Cleanup on transfer complete (FILE_FINISH).
    /// Zeroizes transfer_root_key, chain_key. Session state retained.
    pub fn cleanup_complete(&mut self) {
        self.chain_key.bytes.zeroize();
        self.transfer_id.zeroize();
    }

    /// Cleanup on transfer cancel (CANCEL).
    /// Zeroizes all transfer-scoped state immediately.
    pub fn cleanup_cancel(&mut self) {
        self.cleanup_complete();
    }
}

impl Drop for BtrTransferContext {
    fn drop(&mut self) {
        // chain_key is SecretKey32 which has ZeroizeOnDrop,
        // but we also zeroize transfer_id defensively.
        self.transfer_id.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_shared_secret() -> [u8; 32] {
        [0xAB; 32]
    }

    #[test]
    fn engine_creation() {
        let engine = BtrEngine::new(&make_shared_secret());
        assert_eq!(engine.ratchet_generation(), 0);
        assert_ne!(*engine.session_root_key(), [0u8; 32]);
    }

    #[test]
    fn engine_deterministic_session_root() {
        let a = BtrEngine::new(&make_shared_secret());
        let b = BtrEngine::new(&make_shared_secret());
        assert_eq!(a.session_root_key(), b.session_root_key());
    }

    #[test]
    fn transfer_roundtrip() {
        let shared_secret = make_shared_secret();

        // Verify chain derivation parity: two engines from the same shared
        // secret, performing the same DH ratchet, produce identical keys.
        let engine_a = BtrEngine::new(&shared_secret);
        let engine_b = BtrEngine::new(&shared_secret);

        // Generate keypairs externally for controlled DH
        let kp_a = RatchetKeypair::generate();
        let kp_b = RatchetKeypair::generate();
        let pub_a = kp_a.public_key;
        let pub_b = kp_b.public_key;

        // Both compute DH
        let dh_a = kp_a.diffie_hellman(&pub_b);
        let dh_b = kp_b.diffie_hellman(&pub_a);
        assert_eq!(dh_a, dh_b, "X25519 DH must be symmetric");

        // Both derive new session root from same DH output
        let new_srk_a = derive_ratcheted_session_root(engine_a.session_root_key(), &dh_a);
        let new_srk_b = derive_ratcheted_session_root(engine_b.session_root_key(), &dh_b);
        assert_eq!(new_srk_a, new_srk_b);

        // Both derive transfer root
        let tid = [0x01; 16];
        let trk_a = derive_transfer_root(&new_srk_a, &tid);
        let trk_b = derive_transfer_root(&new_srk_b, &tid);
        assert_eq!(trk_a, trk_b);

        // Chain advance produces same keys
        let adv_a = chain_advance(&trk_a);
        let adv_b = chain_advance(&trk_b);
        assert_eq!(adv_a.message_key, adv_b.message_key);
        assert_eq!(adv_a.next_chain_key, adv_b.next_chain_key);
    }

    #[test]
    fn seal_open_chunk_parity() {
        // Two contexts with identical state should be able to seal/open
        let shared = make_shared_secret();
        let srk = derive_session_root(&shared);
        let tid = [0x42; 16];
        let trk = derive_transfer_root(&srk, &tid);

        let mut sender = BtrTransferContext {
            transfer_id: tid,
            generation: 1,
            chain_key: SecretKey32::new(trk),
            chain_index: 0,
        };
        let mut receiver = BtrTransferContext {
            transfer_id: tid,
            generation: 1,
            chain_key: SecretKey32::new(trk),
            chain_index: 0,
        };

        // Seal 3 chunks and open them
        for i in 0..3u32 {
            let plaintext = format!("chunk {i}");
            let (idx, sealed) = sender.seal_chunk(plaintext.as_bytes()).unwrap();
            assert_eq!(idx, i);
            let opened = receiver.open_chunk(i, &sealed).unwrap();
            assert_eq!(opened, plaintext.as_bytes());
        }
    }

    #[test]
    fn open_chunk_wrong_index_rejected() {
        let trk = [0xAB; 32];
        let mut ctx = BtrTransferContext {
            transfer_id: [0x01; 16],
            generation: 1,
            chain_key: SecretKey32::new(trk),
            chain_index: 0,
        };

        // Try to open at index 1 when expected is 0
        let err = ctx.open_chunk(1, &[0u8; 64]).unwrap_err();
        assert!(matches!(err, BtrError::RatchetChainError(_)));
    }

    #[test]
    fn different_transfers_different_keys() {
        let shared = make_shared_secret();
        let srk = derive_session_root(&shared);
        let trk_a = derive_transfer_root(&srk, &[0x01; 16]);
        let trk_b = derive_transfer_root(&srk, &[0x02; 16]);
        assert_ne!(
            trk_a, trk_b,
            "ISOLATION-BTR: different transfers must have different root keys"
        );
    }

    #[test]
    fn generation_increments() {
        let shared = make_shared_secret();
        let mut engine = BtrEngine::new(&shared);
        assert_eq!(engine.ratchet_generation(), 0);

        let remote_kp = RatchetKeypair::generate();
        let remote_pub = remote_kp.public_key;
        let tid = [0x01; 16];
        let _ = engine.begin_transfer_send(&tid, &remote_pub).unwrap();
        assert_eq!(engine.ratchet_generation(), 1);
    }

    #[test]
    fn cleanup_disconnect_zeroes_state() {
        let mut engine = BtrEngine::new(&make_shared_secret());
        assert_ne!(engine.session_root_key.bytes, [0u8; 32]);

        engine.cleanup_disconnect();

        assert_eq!(engine.session_root_key.bytes, [0u8; 32]);
        assert_eq!(engine.ratchet_generation, 0);
    }

    #[test]
    fn cleanup_complete_zeroes_transfer() {
        let trk = [0xAB; 32];
        let mut ctx = BtrTransferContext {
            transfer_id: [0x01; 16],
            generation: 1,
            chain_key: SecretKey32::new(trk),
            chain_index: 5,
        };
        ctx.cleanup_complete();
        assert_eq!(ctx.chain_key.bytes, [0u8; 32]);
        assert_eq!(ctx.transfer_id, [0u8; 16]);
    }

    #[test]
    fn seal_chunk_advances_index() {
        let trk = [0xAB; 32];
        let mut ctx = BtrTransferContext {
            transfer_id: [0x01; 16],
            generation: 1,
            chain_key: SecretKey32::new(trk),
            chain_index: 0,
        };
        let (idx0, _) = ctx.seal_chunk(b"a").unwrap();
        assert_eq!(idx0, 0);
        assert_eq!(ctx.chain_index(), 1);

        let (idx1, _) = ctx.seal_chunk(b"b").unwrap();
        assert_eq!(idx1, 1);
        assert_eq!(ctx.chain_index(), 2);
    }
}
