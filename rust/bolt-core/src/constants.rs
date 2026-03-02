//! Protocol constants — canonical values shared with TypeScript SDK.
//!
//! Every constant here MUST exactly match the value in
//! `ts/bolt-core/src/constants.ts`. Drift is detected by
//! `scripts/verify-constants.sh` in CI.

/// NaCl box nonce length in bytes.
pub const NONCE_LENGTH: usize = 24;

/// NaCl public key length in bytes (Curve25519).
pub const PUBLIC_KEY_LENGTH: usize = 32;

/// NaCl secret key length in bytes (Curve25519).
pub const SECRET_KEY_LENGTH: usize = 32;

/// Default chunk size for file transfer (bytes).
pub const DEFAULT_CHUNK_SIZE: usize = 16_384;

/// Peer code length (characters).
pub const PEER_CODE_LENGTH: usize = 6;

/// Peer code alphabet (31 chars, unambiguous base32 subset: no 0/O, 1/I/L).
pub const PEER_CODE_ALPHABET: &str = "ABCDEFGHJKMNPQRSTUVWXYZ23456789";

/// SAS (Short Authentication String) length in hex characters.
pub const SAS_LENGTH: usize = 6;

/// NaCl box overhead (Poly1305 MAC).
pub const BOX_OVERHEAD: usize = 16;

/// Transfer ID length in bytes (§14).
pub const TRANSFER_ID_LENGTH: usize = 16;

/// SAS entropy in bits (§14).
pub const SAS_ENTROPY: usize = 24;

/// File hash algorithm identifier (§14).
pub const FILE_HASH_ALGORITHM: &str = "SHA-256";

/// File hash length in bytes (§14).
pub const FILE_HASH_LENGTH: usize = 32;

/// Bolt Protocol version (§14).
pub const BOLT_VERSION: usize = 1;

/// Capability namespace prefix (§14). All capability strings start with this.
pub const CAPABILITY_NAMESPACE: &str = "bolt.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_protocol() {
        assert_eq!(NONCE_LENGTH, 24);
        assert_eq!(PUBLIC_KEY_LENGTH, 32);
        assert_eq!(SECRET_KEY_LENGTH, 32);
        assert_eq!(DEFAULT_CHUNK_SIZE, 16_384);
        assert_eq!(PEER_CODE_LENGTH, 6);
        assert_eq!(PEER_CODE_ALPHABET, "ABCDEFGHJKMNPQRSTUVWXYZ23456789");
        assert_eq!(SAS_LENGTH, 6);
        assert_eq!(BOX_OVERHEAD, 16);
        assert_eq!(TRANSFER_ID_LENGTH, 16);
        assert_eq!(SAS_ENTROPY, 24);
        assert_eq!(FILE_HASH_ALGORITHM, "SHA-256");
        assert_eq!(FILE_HASH_LENGTH, 32);
        assert_eq!(BOLT_VERSION, 1);
        assert_eq!(CAPABILITY_NAMESPACE, "bolt.");
    }

    #[test]
    fn peer_code_alphabet_length() {
        assert_eq!(PEER_CODE_ALPHABET.len(), 31);
        // Must not contain ambiguous characters: 0, O, 1, I, L
        assert!(!PEER_CODE_ALPHABET.contains('0'));
        assert!(!PEER_CODE_ALPHABET.contains('O'));
        assert!(!PEER_CODE_ALPHABET.contains('1'));
        assert!(!PEER_CODE_ALPHABET.contains('I'));
        assert!(!PEER_CODE_ALPHABET.contains('L'));
    }
}
