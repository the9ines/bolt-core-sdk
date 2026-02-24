//! Hashing utilities — SHA-256 and hex encoding.
//!
//! TS uses `crypto.subtle.digest('SHA-256', ...)` (async, Web Crypto).
//! Rust uses `sha2` crate (synchronous).
//!
//! `hashFile` (Blob -> SHA-256) is a transport concern and stays in
//! TS/transport layer. Core provides `sha256(bytes)` only.
//!
//! ## Parity
//! - SHA-256 of each golden vector plaintext matches TS output.
//! - `buffer_to_hex` matches TS `bufferToHex` for all vector nonces/keys.

use sha2::{Digest, Sha256};

use crate::encoding;

/// Compute SHA-256 hash of arbitrary data.
///
/// # Parity
/// TS equivalent: `sha256(data: ArrayBuffer | Uint8Array)`.
/// Both use SHA-256 — output is deterministic and identical.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Compute SHA-256 and return lowercase hex string.
///
/// Convenience wrapper combining `sha256` + `buffer_to_hex`.
pub fn sha256_hex(data: &[u8]) -> String {
    encoding::to_hex(&sha256(data))
}

/// Convert bytes to lowercase hex string.
///
/// Delegates to `encoding::to_hex`. This function exists for API
/// symmetry with the TS export `bufferToHex`.
///
/// # Parity
/// TS equivalent: `bufferToHex(buffer: ArrayBuffer)`.
pub fn buffer_to_hex(data: &[u8]) -> String {
    encoding::to_hex(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_empty() {
        // NIST: SHA-256("") = e3b0c442...
        let hex = sha256_hex(&[]);
        assert_eq!(
            hex,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn sha256_abc() {
        // NIST: SHA-256("abc") = ba7816bf...
        let hex = sha256_hex(b"abc");
        assert_eq!(
            hex,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn sha256_returns_32_bytes() {
        let hash = sha256(b"Hello, Bolt!");
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn buffer_to_hex_known_value() {
        assert_eq!(buffer_to_hex(&[0xde, 0xad, 0xbe, 0xef]), "deadbeef");
    }

    #[test]
    fn buffer_to_hex_empty() {
        assert_eq!(buffer_to_hex(&[]), "");
    }
}
