//! BTR encryption — NaCl secretbox keyed by BTR message_key (§16.4).
//!
//! Uses symmetric NaCl secretbox (XSalsa20-Poly1305), NOT asymmetric box.
//! Both peers derive identical message_key deterministically via HKDF.
//! Fresh 24-byte CSPRNG nonce per envelope.

use crypto_secretbox::aead::Aead;
use crypto_secretbox::{KeyInit, Nonce, XSalsa20Poly1305};
use rand_core::{OsRng, RngCore};

use crate::errors::BtrError;

/// NaCl secretbox nonce length (24 bytes).
const SECRETBOX_NONCE_LENGTH: usize = 24;

/// NaCl secretbox MAC overhead (Poly1305, 16 bytes).
const SECRETBOX_OVERHEAD: usize = 16;

/// Encrypt a chunk using NaCl secretbox with a BTR-derived message_key.
///
/// Returns `nonce || ciphertext` (24 + plaintext.len() + 16 bytes).
/// Fresh 24-byte CSPRNG nonce generated internally.
///
/// Caller MUST zeroize `message_key` after this call (single-use).
pub fn btr_seal(message_key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, BtrError> {
    let cipher = XSalsa20Poly1305::new(message_key.into());

    let mut nonce_bytes = [0u8; SECRETBOX_NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| BtrError::RatchetDecryptFail("secretbox seal failed".into()))?;

    let mut combined = Vec::with_capacity(SECRETBOX_NONCE_LENGTH + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);
    Ok(combined)
}

/// Decrypt a chunk using NaCl secretbox with a BTR-derived message_key.
///
/// Expects `nonce || ciphertext` format (first 24 bytes are nonce).
///
/// Caller MUST zeroize `message_key` after this call (single-use).
pub fn btr_open(message_key: &[u8; 32], sealed: &[u8]) -> Result<Vec<u8>, BtrError> {
    if sealed.len() < SECRETBOX_NONCE_LENGTH + SECRETBOX_OVERHEAD {
        return Err(BtrError::RatchetDecryptFail(
            "sealed payload too short".into(),
        ));
    }

    let nonce = Nonce::from_slice(&sealed[..SECRETBOX_NONCE_LENGTH]);
    let ciphertext = &sealed[SECRETBOX_NONCE_LENGTH..];

    let cipher = XSalsa20Poly1305::new(message_key.into());
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| BtrError::RatchetDecryptFail("secretbox open failed".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seal_open_roundtrip() {
        let key = [0xAB; 32];
        let plaintext = b"Hello, BTR!";
        let sealed = btr_seal(&key, plaintext).unwrap();
        let opened = btr_open(&key, &sealed).unwrap();
        assert_eq!(opened, plaintext);
    }

    #[test]
    fn seal_open_empty_payload() {
        let key = [0xAB; 32];
        let sealed = btr_seal(&key, &[]).unwrap();
        let opened = btr_open(&key, &sealed).unwrap();
        assert!(opened.is_empty());
    }

    #[test]
    fn seal_output_format() {
        let key = [0xAB; 32];
        let plaintext = b"test";
        let sealed = btr_seal(&key, plaintext).unwrap();
        // nonce(24) + ciphertext(plaintext_len + 16 MAC)
        assert_eq!(sealed.len(), 24 + plaintext.len() + 16);
    }

    #[test]
    fn open_fails_wrong_key() {
        let key_a = [0xAB; 32];
        let key_b = [0xCD; 32];
        let sealed = btr_seal(&key_a, b"secret").unwrap();
        let result = btr_open(&key_b, &sealed);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            BtrError::RatchetDecryptFail(_)
        ));
    }

    #[test]
    fn open_fails_tampered() {
        let key = [0xAB; 32];
        let mut sealed = btr_seal(&key, b"secret").unwrap();
        let last = sealed.len() - 1;
        sealed[last] ^= 0x01;
        let result = btr_open(&key, &sealed);
        assert!(result.is_err());
    }

    #[test]
    fn open_fails_truncated() {
        let key = [0xAB; 32];
        let result = btr_open(&key, &[0u8; 10]);
        assert!(result.is_err());
    }

    #[test]
    fn seal_produces_unique_nonces() {
        let key = [0xAB; 32];
        let mut nonces = std::collections::HashSet::new();
        for _ in 0..64 {
            let sealed = btr_seal(&key, b"test").unwrap();
            let nonce: [u8; 24] = sealed[..24].try_into().unwrap();
            assert!(nonces.insert(nonce), "duplicate nonce");
        }
    }

    #[test]
    fn seal_large_payload() {
        let key = [0xAB; 32];
        let plaintext = vec![0xFFu8; 16384]; // DEFAULT_CHUNK_SIZE
        let sealed = btr_seal(&key, &plaintext).unwrap();
        let opened = btr_open(&key, &sealed).unwrap();
        assert_eq!(opened, plaintext);
    }
}
