//! Identity — long-lived X25519 keypairs and TOFU error.
//!
//! Identity keys are persistent across sessions. The transport layer
//! stores them (IndexedDB for web, filesystem for native). This module
//! only provides generation and the mismatch error type.
//!
//! ## Parity
//! - `generate_identity_keypair()` produces valid 32-byte keys.
//! - Public key is derivable from secret key (X25519 property).
//! - `KeyMismatchError` carries peer_code, expected, received fields.
//!
//! ## Non-goals
//! - No TOFU pin storage (transport concern).
//! - No persistence logic.

use crate::crypto::{generate_ephemeral_keypair, KeyPair};

/// Long-lived X25519 identity keypair.
///
/// Alias for `KeyPair` — same structure, different lifetime semantics.
/// Identity keys MUST NOT be sent through the signaling server; they
/// travel only inside encrypted DataChannel messages (HELLO).
pub type IdentityKeyPair = KeyPair;

/// Generate a persistent identity keypair (X25519).
///
/// # Parity
/// TS equivalent: `generateIdentityKeyPair()` (tweetnacl `box.keyPair()`).
/// Both use X25519 via OS CSPRNG — keypairs are structurally compatible.
pub fn generate_identity_keypair() -> IdentityKeyPair {
    generate_ephemeral_keypair()
}

/// TOFU violation error.
///
/// Thrown when a peer's identity public key does not match a previously
/// pinned value. The session MUST be aborted.
///
/// # Parity
/// TS equivalent: `KeyMismatchError` class (extends `BoltError`).
#[derive(Debug)]
pub struct KeyMismatchError {
    /// Peer code of the offending peer.
    pub peer_code: String,
    /// Previously pinned public key (32 bytes).
    pub expected: [u8; 32],
    /// Received public key that does not match (32 bytes).
    pub received: [u8; 32],
}

impl std::fmt::Display for KeyMismatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Identity key mismatch for peer {}", self.peer_code)
    }
}

impl std::error::Error for KeyMismatchError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_mismatch_error_display() {
        let err = KeyMismatchError {
            peer_code: "ABC123".into(),
            expected: [1u8; 32],
            received: [2u8; 32],
        };
        assert_eq!(err.to_string(), "Identity key mismatch for peer ABC123");
    }

    #[test]
    fn key_mismatch_error_is_error_trait() {
        let err = KeyMismatchError {
            peer_code: "XYZ789".into(),
            expected: [0u8; 32],
            received: [0u8; 32],
        };
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn identity_keypair_lengths() {
        let kp = generate_identity_keypair();
        assert_eq!(kp.public_key.len(), 32);
        assert_eq!(kp.secret_key.len(), 32);
    }

    #[test]
    fn identity_keypair_nonzero() {
        let kp = generate_identity_keypair();
        assert_ne!(kp.public_key, [0u8; 32]);
    }
}
