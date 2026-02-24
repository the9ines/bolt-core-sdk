//! Hashing utilities — SHA-256 and hex encoding.
//!
//! TS uses `crypto.subtle.digest('SHA-256', ...)` (async, Web Crypto).
//! Rust uses `sha2` crate (synchronous).
//!
//! `hashFile` (Blob -> SHA-256) is a transport concern and stays in
//! TS/transport layer. Core provides `sha256(bytes)` only.
//!
//! ## Parity gates (R2)
//! - SHA-256 of each golden vector plaintext matches TS output.
//! - `buffer_to_hex` matches TS `bufferToHex` for all vector nonces/keys.

use crate::encoding;

/// Compute SHA-256 hash of arbitrary data.
///
/// # Parity
/// TS equivalent: `sha256(data: ArrayBuffer | Uint8Array)`.
/// Both use SHA-256 — output is deterministic and identical.
pub fn sha256(_data: &[u8]) -> [u8; 32] {
    todo!("R2: implement using sha2::Sha256")
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
    fn buffer_to_hex_known_value() {
        assert_eq!(buffer_to_hex(&[0xde, 0xad, 0xbe, 0xef]), "deadbeef");
    }

    #[test]
    fn buffer_to_hex_empty() {
        assert_eq!(buffer_to_hex(&[]), "");
    }
}
