//! Crypto primitives — NaCl box (XSalsa20-Poly1305).
//!
//! Mirrors TS `sealBoxPayload` / `openBoxPayload` exactly.
//! Wire format: `base64(nonce || ciphertext)`.
//!
//! ## Parity
//! - For each golden vector: `open_box_payload(sealed, sender_pk, receiver_sk)`
//!   returns expected plaintext.
//! - For each corrupt vector: `open_box_payload` returns `BoltError::Encryption`.
//! - Round-trip: `open(seal(plaintext)) == plaintext`.

use crypto_box::{aead::Aead, Nonce, SalsaBox, SecretKey};
use rand_core::{OsRng, RngCore};

use crate::constants::NONCE_LENGTH;
use crate::encoding::{from_base64, to_base64};
use crate::errors::BoltError;

/// X25519 keypair (ephemeral or identity).
///
/// 32-byte public key (Curve25519 point) and 32-byte secret key.
/// Secret key is deterministically zeroized on drop via volatile writes.
#[derive(Clone)]
pub struct KeyPair {
    /// Curve25519 public key (32 bytes).
    pub public_key: [u8; 32],
    /// Curve25519 secret key (32 bytes).
    pub secret_key: [u8; 32],
}

impl Drop for KeyPair {
    fn drop(&mut self) {
        // Volatile writes prevent the compiler from optimizing away the zeroization.
        for byte in self.secret_key.iter_mut() {
            unsafe { std::ptr::write_volatile(byte as *mut u8, 0u8) };
        }
        std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
    }
}

/// Generate a fresh ephemeral X25519 keypair.
///
/// Discard after session ends. Uses OS CSPRNG.
///
/// # Parity
/// TS equivalent: `generateEphemeralKeyPair()` (tweetnacl `box.keyPair()`).
/// Both use X25519 — keypairs are structurally compatible.
pub fn generate_ephemeral_keypair() -> KeyPair {
    let mut secret_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut secret_bytes);
    let sk = SecretKey::from(secret_bytes);
    let pk = sk.public_key();
    KeyPair {
        public_key: *pk.as_bytes(),
        secret_key: secret_bytes,
    }
}

/// Seal plaintext using NaCl box (XSalsa20-Poly1305).
///
/// Wire format: `base64(nonce || ciphertext)`.
/// Random 24-byte nonce generated internally via CSPRNG.
///
/// # Parity
/// TS equivalent: `sealBoxPayload(plaintext, remotePublicKey, senderSecretKey)`.
/// Identical wire format, identical crypto — only the random nonce differs
/// (by design). Parity is verified via golden vector open tests.
///
/// # Errors
/// Returns `BoltError::Encryption` if box sealing fails.
pub fn seal_box_payload(
    plaintext: &[u8],
    remote_public_key: &[u8; 32],
    sender_secret_key: &[u8; 32],
) -> Result<String, BoltError> {
    let pk = crypto_box::PublicKey::from(*remote_public_key);
    let sk = SecretKey::from(*sender_secret_key);
    let salsa_box = SalsaBox::new(&pk, &sk);

    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = salsa_box
        .encrypt(nonce, plaintext)
        .map_err(|_| BoltError::Encryption("Encryption failed".into()))?;

    let mut combined = Vec::with_capacity(NONCE_LENGTH + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(to_base64(&combined))
}

/// Open a sealed payload using NaCl box.open.
///
/// Expects wire format: `base64(nonce || ciphertext)`.
/// Splits first 24 bytes as nonce, remainder as ciphertext.
///
/// # Parity
/// TS equivalent: `openBoxPayload(sealed, senderPublicKey, receiverSecretKey)`.
/// Identical format parsing and decryption.
///
/// # Errors
/// Returns `BoltError::Encryption` on decryption failure (tampered,
/// wrong key, truncated, etc.).
pub fn open_box_payload(
    sealed: &str,
    sender_public_key: &[u8; 32],
    receiver_secret_key: &[u8; 32],
) -> Result<Vec<u8>, BoltError> {
    let data = from_base64(sealed)?;
    if data.len() < NONCE_LENGTH {
        return Err(BoltError::Encryption("Sealed payload too short".into()));
    }

    let nonce = Nonce::from_slice(&data[..NONCE_LENGTH]);
    let ciphertext = &data[NONCE_LENGTH..];

    let pk = crypto_box::PublicKey::from(*sender_public_key);
    let sk = SecretKey::from(*receiver_secret_key);
    let salsa_box = SalsaBox::new(&pk, &sk);

    salsa_box
        .decrypt(nonce, ciphertext)
        .map_err(|_| BoltError::Encryption("Decryption failed".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::from_hex;

    #[test]
    fn keypair_struct_is_clone() {
        let kp = KeyPair {
            public_key: [0u8; 32],
            secret_key: [0u8; 32],
        };
        let _cloned = kp.clone();
    }

    #[test]
    fn keypair_generation_correct_lengths() {
        let kp = generate_ephemeral_keypair();
        assert_eq!(kp.public_key.len(), 32);
        assert_eq!(kp.secret_key.len(), 32);
    }

    #[test]
    fn keypair_generation_nonzero() {
        let kp = generate_ephemeral_keypair();
        // Public key must not be all zeros (astronomically unlikely with CSPRNG).
        assert_ne!(kp.public_key, [0u8; 32]);
    }

    #[test]
    fn seal_open_roundtrip() {
        let alice = generate_ephemeral_keypair();
        let bob = generate_ephemeral_keypair();
        let plaintext = b"Hello, Bolt!";

        let sealed = seal_box_payload(plaintext, &bob.public_key, &alice.secret_key).unwrap();
        let opened = open_box_payload(&sealed, &alice.public_key, &bob.secret_key).unwrap();

        assert_eq!(opened, plaintext);
    }

    #[test]
    fn seal_open_empty_payload() {
        let alice = generate_ephemeral_keypair();
        let bob = generate_ephemeral_keypair();

        let sealed = seal_box_payload(&[], &bob.public_key, &alice.secret_key).unwrap();
        let opened = open_box_payload(&sealed, &alice.public_key, &bob.secret_key).unwrap();

        assert!(opened.is_empty());
    }

    #[test]
    fn open_fails_with_wrong_key() {
        let alice = generate_ephemeral_keypair();
        let bob = generate_ephemeral_keypair();
        let eve = generate_ephemeral_keypair();
        let plaintext = b"secret message";

        let sealed = seal_box_payload(plaintext, &bob.public_key, &alice.secret_key).unwrap();

        // Eve tries to open with her own secret key — must fail.
        let result = open_box_payload(&sealed, &alice.public_key, &eve.secret_key);
        assert!(result.is_err());
    }

    #[test]
    fn open_fails_on_truncated_payload() {
        // Less than NONCE_LENGTH bytes after base64 decode.
        let short = crate::encoding::to_base64(&[0u8; 10]);
        let kp = generate_ephemeral_keypair();
        let result = open_box_payload(&short, &kp.public_key, &kp.secret_key);
        assert!(result.is_err());
    }

    #[test]
    fn open_golden_vector_hello_bolt() {
        // From box-payload.vectors.json: "hello-bolt" vector.
        // sender_pk_hex from the committed vector file.
        let sender_pk_bytes =
            from_hex("07a37cbc142093c8b755dc1b10e86cb426374ad16aa853ed0bdfc0b2b86d1c7c").unwrap();
        let sender_pk: [u8; 32] = sender_pk_bytes.try_into().unwrap();

        // receiver_sk is bytes 0x21..0x40 (offset 33).
        let receiver_sk: [u8; 32] = core::array::from_fn(|i| (i as u8) + 33);

        let sealed = "AAECAwQFBgcICQoLDA0ODxAREhMUFRYXvjBFLvx0BRI+SIiwhwJMy1qQtzU1EV2Qlp41Ig==";

        let result = open_box_payload(sealed, &sender_pk, &receiver_sk).unwrap();
        assert_eq!(result, b"Hello, Bolt!");
    }

    #[test]
    fn open_golden_vector_corrupt_rejected() {
        // From box-payload.vectors.json: "modified-ciphertext" — last byte flipped.
        let sender_pk_bytes =
            from_hex("07a37cbc142093c8b755dc1b10e86cb426374ad16aa853ed0bdfc0b2b86d1c7c").unwrap();
        let sender_pk: [u8; 32] = sender_pk_bytes.try_into().unwrap();
        let receiver_sk: [u8; 32] = core::array::from_fn(|i| (i as u8) + 33);

        let sealed = "AAECAwQFBgcICQoLDA0ODxAREhMUFRYXvjBFLvx0BRI+SIiwhwJMy1qQtzU1EV2Qlp41Iw==";

        let result = open_box_payload(sealed, &sender_pk, &receiver_sk);
        assert!(result.is_err());
    }

    /// SA4: Drop zeroizes secret key via volatile writes.
    ///
    /// Allocates KeyPair on heap, captures a raw pointer to the secret buffer,
    /// drops the Box, then reads the memory region with read_volatile to confirm
    /// all bytes are zero.
    #[test]
    fn keypair_drop_zeroizes_secret() {
        let kp = Box::new(generate_ephemeral_keypair());
        // Confirm secret is non-zero before drop.
        assert_ne!(
            kp.secret_key, [0u8; 32],
            "secret key must be non-zero after generation"
        );

        // Capture raw pointer to the secret key buffer inside the heap allocation.
        let secret_ptr = kp.secret_key.as_ptr();

        // Drop the KeyPair — Drop::drop should volatile-zero the secret.
        drop(kp);

        // Immediately read the memory region. The allocator has not been asked
        // for new memory, so the region should still be accessible (though
        // logically freed). read_volatile prevents the compiler from eliding
        // the reads.
        for i in 0..32 {
            let byte = unsafe { std::ptr::read_volatile(secret_ptr.add(i)) };
            assert_eq!(byte, 0, "secret_key byte {} not zeroed after drop", i);
        }
    }

    /// SA4: Double-drop safety — dropping a default-constructed KeyPair does not panic.
    #[test]
    fn keypair_drop_zeros_safe() {
        let kp = KeyPair {
            public_key: [0u8; 32],
            secret_key: [0u8; 32],
        };
        drop(kp);
        // If we reach here, Drop did not panic on an all-zero secret.
    }

    /// Nonce uniqueness sanity test (H6).
    ///
    /// Seals N times via the production `seal_box_payload` path and verifies
    /// all nonces are unique, exactly 24 bytes, and non-zero.
    ///
    /// This is an in-process statistical sanity check, NOT a cryptographic
    /// guarantee of cross-process uniqueness.
    #[test]
    fn nonce_uniqueness_sanity() {
        use std::collections::HashSet;

        const N: usize = 128;
        let alice = generate_ephemeral_keypair();
        let bob = generate_ephemeral_keypair();
        let plaintext = b"nonce-test";

        let mut seen = HashSet::new();
        let zero_nonce = [0u8; NONCE_LENGTH];

        for _ in 0..N {
            let sealed = seal_box_payload(plaintext, &bob.public_key, &alice.secret_key).unwrap();
            let raw = from_base64(&sealed).unwrap();
            assert!(
                raw.len() >= NONCE_LENGTH,
                "sealed payload shorter than nonce"
            );

            let nonce: [u8; NONCE_LENGTH] = raw[..NONCE_LENGTH].try_into().unwrap();

            // Nonce must not be all-zero.
            assert_ne!(nonce, zero_nonce, "nonce must not be all-zero");

            // Nonce must be unique within this run.
            assert!(
                seen.insert(nonce),
                "duplicate nonce detected after {} seals",
                seen.len()
            );
        }

        assert_eq!(seen.len(), N);
    }
}
