//! Bolt Core — canonical reference implementation.
//!
//! This crate is the canonical source of truth for Bolt protocol crypto
//! primitives and constants. The TypeScript SDK (`@the9ines/bolt-core`)
//! is a supported adapter implementation that MUST produce identical
//! outputs for identical inputs, verified by shared golden test vectors.
//!
//! **Status:** Vector authority established. The Rust crate can
//! deterministically reproduce the golden test vectors. Full crypto
//! primitives (public API) are not yet exposed.

/// Deterministic golden vector generator (test use only).
pub mod vectors;

/// Protocol constants.
pub mod constants {
    /// NaCl box nonce length in bytes.
    pub const NONCE_LENGTH: usize = 24;

    /// NaCl public key length in bytes (Curve25519).
    pub const PUBLIC_KEY_LENGTH: usize = 32;

    /// NaCl secret key length in bytes (Curve25519).
    pub const SECRET_KEY_LENGTH: usize = 32;

    /// Default chunk size for file transfer (bytes).
    pub const DEFAULT_CHUNK_SIZE: usize = 16_384;

    /// Peer code length (characters).
    pub const PEER_CODE_LENGTH: usize = 4;

    /// Peer code alphabet.
    pub const PEER_CODE_ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

    /// SAS (Short Authentication String) length in characters.
    pub const SAS_LENGTH: usize = 4;

    /// NaCl box overhead (Poly1305 MAC).
    pub const BOX_OVERHEAD: usize = 16;
}

#[cfg(test)]
mod tests {
    use super::constants::*;

    #[test]
    fn constants_match_protocol() {
        assert_eq!(NONCE_LENGTH, 24);
        assert_eq!(PUBLIC_KEY_LENGTH, 32);
        assert_eq!(SECRET_KEY_LENGTH, 32);
        assert_eq!(DEFAULT_CHUNK_SIZE, 16_384);
        assert_eq!(PEER_CODE_LENGTH, 4);
        assert_eq!(PEER_CODE_ALPHABET, "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789");
        assert_eq!(SAS_LENGTH, 4);
        assert_eq!(BOX_OVERHEAD, 16);
    }
}
