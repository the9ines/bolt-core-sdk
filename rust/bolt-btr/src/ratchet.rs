//! Inter-transfer DH ratchet — §16.3 DH ratchet step.
//!
//! At each transfer boundary (FILE_OFFER sent/received), both peers:
//! 1. Generate a fresh X25519 keypair
//! 2. Compute DH shared secret with remote ratchet public key
//! 3. Derive new session_root_key via HKDF
//! 4. Increment ratchet_generation
//! 5. Zeroize old secret key and old session_root_key

use hkdf::Hkdf;
use rand_core::OsRng;
use sha2::Sha256;
use x25519_dalek::{EphemeralSecret, PublicKey};
use zeroize::Zeroize;

use crate::constants::{BTR_DH_RATCHET_INFO, BTR_KEY_LENGTH};

/// X25519 ratchet keypair for DH ratchet steps.
///
/// The secret key is consumed on DH computation (EphemeralSecret is move-only).
/// The public key is sent to the peer in the envelope.
pub struct RatchetKeypair {
    /// Public key to include in envelope `ratchet_public_key` field.
    pub public_key: [u8; 32],
    /// Ephemeral secret — consumed (moved) on DH computation.
    secret: Option<EphemeralSecret>,
}

impl RatchetKeypair {
    /// Generate a fresh X25519 keypair for the DH ratchet step.
    pub fn generate() -> Self {
        let secret = EphemeralSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);
        Self {
            public_key: *public.as_bytes(),
            secret: Some(secret),
        }
    }

    /// Perform DH with the remote peer's ratchet public key.
    /// Consumes the secret key (cannot be reused).
    ///
    /// Returns the raw 32-byte DH output.
    pub fn diffie_hellman(mut self, remote_public: &[u8; 32]) -> [u8; 32] {
        let secret = self.secret.take().expect("DH secret already consumed");
        let remote_pk = PublicKey::from(*remote_public);
        let shared = secret.diffie_hellman(&remote_pk);
        *shared.as_bytes()
    }
}

impl Drop for RatchetKeypair {
    fn drop(&mut self) {
        // EphemeralSecret handles its own zeroization internally.
        // We only need to ensure it's dropped (which happens automatically).
        self.public_key.zeroize();
    }
}

/// Derive new session root key from DH ratchet step output (§16.3).
///
/// ```text
/// new_session_root_key = HKDF-SHA256(
///   salt  = current_session_root_key,
///   ikm   = dh_output,
///   info  = "bolt-btr-dh-ratchet-v1",
///   len   = 32
/// )
/// ```
pub fn derive_ratcheted_session_root(
    current_session_root_key: &[u8; BTR_KEY_LENGTH],
    dh_output: &[u8; 32],
) -> [u8; BTR_KEY_LENGTH] {
    let hk = Hkdf::<Sha256>::new(Some(current_session_root_key), dh_output);
    let mut okm = [0u8; BTR_KEY_LENGTH];
    hk.expand(BTR_DH_RATCHET_INFO, &mut okm)
        .expect("HKDF expand with 32-byte output must not fail");
    okm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_produces_valid_keypair() {
        let kp = RatchetKeypair::generate();
        assert_ne!(kp.public_key, [0u8; 32]);
        assert!(kp.secret.is_some());
    }

    #[test]
    fn dh_produces_shared_secret() {
        let alice = RatchetKeypair::generate();
        let bob = RatchetKeypair::generate();

        let alice_pub = alice.public_key;
        let bob_pub = bob.public_key;

        let alice_shared = alice.diffie_hellman(&bob_pub);
        let bob_shared = bob.diffie_hellman(&alice_pub);

        assert_eq!(alice_shared, bob_shared);
        assert_ne!(alice_shared, [0u8; 32]);
    }

    #[test]
    fn dh_different_peers_different_secrets() {
        let alice = RatchetKeypair::generate();
        let bob = RatchetKeypair::generate();
        let charlie = RatchetKeypair::generate();

        let bob_pub = bob.public_key;
        let charlie_pub = charlie.public_key;

        let ab = alice.diffie_hellman(&bob_pub);
        // bob DH with charlie instead of alice
        let bc = bob.diffie_hellman(&charlie_pub);

        assert_ne!(ab, bc);
    }

    #[test]
    fn ratcheted_session_root_deterministic() {
        let srk = [0xAB; 32];
        let dh = [0xCD; 32];
        let a = derive_ratcheted_session_root(&srk, &dh);
        let b = derive_ratcheted_session_root(&srk, &dh);
        assert_eq!(a, b);
        assert_ne!(a, [0u8; 32]);
    }

    #[test]
    fn ratcheted_session_root_binds_to_both_inputs() {
        let dh = [0xCD; 32];
        let a = derive_ratcheted_session_root(&[0x01; 32], &dh);
        let b = derive_ratcheted_session_root(&[0x02; 32], &dh);
        assert_ne!(a, b);

        let srk = [0xAB; 32];
        let c = derive_ratcheted_session_root(&srk, &[0x01; 32]);
        let d = derive_ratcheted_session_root(&srk, &[0x02; 32]);
        assert_ne!(c, d);
    }

    #[test]
    fn keypair_public_key_zeroized_on_drop() {
        let kp = Box::new(RatchetKeypair::generate());
        let pk_ptr = kp.public_key.as_ptr();
        // Confirm non-zero
        assert_ne!(kp.public_key, [0u8; 32]);
        drop(kp);
        for i in 0..32 {
            let byte = unsafe { std::ptr::read_volatile(pk_ptr.add(i)) };
            assert_eq!(byte, 0, "public_key byte {i} not zeroed after drop");
        }
    }
}
