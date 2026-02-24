//! Crypto primitives — NaCl box (XSalsa20-Poly1305).
//!
//! Mirrors TS `sealBoxPayload` / `openBoxPayload` exactly.
//! Wire format: `base64(nonce || ciphertext)`.
//!
//! ## Parity gates (R1)
//! - For each golden vector: `open_box_payload(sealed, sender_pk, receiver_sk)`
//!   returns expected plaintext.
//! - For each corrupt vector: `open_box_payload` returns `BoltError::Encryption`.
//! - Fixed-nonce seal (test helper) produces byte-identical output to TS.
//! - Round-trip: `open(seal(plaintext)) == plaintext`.

use crate::errors::BoltError;

/// X25519 keypair (ephemeral or identity).
///
/// 32-byte public key (Curve25519 point) and 32-byte secret key.
#[derive(Clone)]
pub struct KeyPair {
    /// Curve25519 public key (32 bytes).
    pub public_key: [u8; 32],
    /// Curve25519 secret key (32 bytes).
    pub secret_key: [u8; 32],
}

/// Generate a fresh ephemeral X25519 keypair.
///
/// Discard after session ends. Uses OS CSPRNG.
///
/// # Parity
/// TS equivalent: `generateEphemeralKeyPair()` (tweetnacl `box.keyPair()`).
/// Both use X25519 — keypairs are structurally compatible.
pub fn generate_ephemeral_keypair() -> KeyPair {
    todo!("R1: implement using crypto_box::SecretKey + OsRng")
}

/// Seal plaintext using NaCl box (XSalsa20-Poly1305).
///
/// Wire format: `base64(nonce || ciphertext)`.
/// Random 24-byte nonce generated internally via CSPRNG.
///
/// # Parity
/// TS equivalent: `sealBoxPayload(plaintext, remotePublicKey, senderSecretKey)`.
/// Identical wire format, identical crypto — only the random nonce differs
/// (by design). Parity is verified via fixed-nonce test helper.
///
/// # Errors
/// Returns `BoltError::Encryption` if box sealing fails.
pub fn seal_box_payload(
    _plaintext: &[u8],
    _remote_public_key: &[u8; 32],
    _sender_secret_key: &[u8; 32],
) -> Result<String, BoltError> {
    todo!("R1: implement using crypto_box::SalsaBox + OsRng nonce")
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
    _sealed: &str,
    _sender_public_key: &[u8; 32],
    _receiver_secret_key: &[u8; 32],
) -> Result<Vec<u8>, BoltError> {
    todo!("R1: implement using crypto_box::SalsaBox + from_base64")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keypair_struct_is_clone() {
        // Verify KeyPair derives Clone (needed for identity persistence).
        let kp = KeyPair {
            public_key: [0u8; 32],
            secret_key: [0u8; 32],
        };
        let _cloned = kp.clone();
    }
}
