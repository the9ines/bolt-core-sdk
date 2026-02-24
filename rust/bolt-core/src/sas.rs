//! SAS — Short Authentication String computation.
//!
//! CANONICAL: `compute_sas()` is the ONLY SAS implementation in the
//! Bolt ecosystem. No SAS logic may exist in transport or product
//! packages. See `scripts/verify-no-shadow-sas.sh`.
//!
//! ## Algorithm
//! ```text
//! SAS_input = SHA-256(sort32(identityA, identityB) || sort32(ephemeralA, ephemeralB))
//! Display   = first 6 hex chars, uppercase
//! ```
//!
//! ## Parity gates (R2)
//! - Each SAS golden vector produces expected 6-char string.
//! - Commutative: `compute_sas(A, B, ...) == compute_sas(B, A, ...)`.
//! - Wrong key lengths rejected.
//!
//! ## Design note
//! TS `computeSas` is async (Web Crypto digest is async). Rust SHA-256
//! via `sha2` is synchronous. The Rust function is sync.

use crate::constants::PUBLIC_KEY_LENGTH;
use crate::errors::BoltError;

/// Compute a 6-character SAS (Short Authentication String).
///
/// Both peers compute the same SAS from their exchanged public keys.
/// The SAS is displayed to users for out-of-band verification.
///
/// # Arguments
/// - `identity_a`, `identity_b` — 32-byte identity public keys
/// - `ephemeral_a`, `ephemeral_b` — 32-byte ephemeral public keys
///
/// # Returns
/// 6-character uppercase hex string (24 bits of entropy).
///
/// # Errors
/// Returns `BoltError::Encryption` if any key is not exactly 32 bytes.
///
/// # Parity
/// TS equivalent: `computeSas(identityA, identityB, ephemeralA, ephemeralB)`.
pub fn compute_sas(
    _identity_a: &[u8; PUBLIC_KEY_LENGTH],
    _identity_b: &[u8; PUBLIC_KEY_LENGTH],
    _ephemeral_a: &[u8; PUBLIC_KEY_LENGTH],
    _ephemeral_b: &[u8; PUBLIC_KEY_LENGTH],
) -> Result<String, BoltError> {
    todo!("R2: implement sort32 + SHA-256 + hex truncation")
}

// sort32 helper will be implemented in R2.
// Lexicographically sorts two 32-byte values and concatenates them.
// Matches TS sort32() exactly.

#[cfg(test)]
mod tests {
    #[test]
    fn sas_length_constant() {
        assert_eq!(crate::constants::SAS_LENGTH, 6);
    }
}
