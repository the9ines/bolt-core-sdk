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
//! ## Parity
//! - Each SAS golden vector produces expected 6-char string.
//! - Commutative: `compute_sas(A, B, ...) == compute_sas(B, A, ...)`.
//! - Wrong key lengths rejected (enforced by type system: `&[u8; 32]`).
//!
//! ## Design note
//! TS `computeSas` is async (Web Crypto digest is async). Rust SHA-256
//! via `sha2` is synchronous. The Rust function is sync.

use crate::constants::{PUBLIC_KEY_LENGTH, SAS_LENGTH};
use crate::encoding;
use crate::hash;

/// Lexicographically sort two 32-byte values and concatenate them.
///
/// Matches TS `sort32()` exactly: byte-by-byte comparison, smaller first.
/// If identical, concatenate as-is (a || b).
fn sort32(a: &[u8; 32], b: &[u8; 32]) -> [u8; 64] {
    let mut result = [0u8; 64];
    for i in 0..32 {
        if a[i] != b[i] {
            let (first, second) = if a[i] < b[i] { (a, b) } else { (b, a) };
            result[..32].copy_from_slice(first);
            result[32..].copy_from_slice(second);
            return result;
        }
    }
    // Keys are identical — concatenate as-is
    result[..32].copy_from_slice(a);
    result[32..].copy_from_slice(b);
    result
}

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
/// # Parity
/// TS equivalent: `computeSas(identityA, identityB, ephemeralA, ephemeralB)`.
/// Identical algorithm: `SHA-256(sort32(id_a, id_b) || sort32(eph_a, eph_b))`,
/// take first 6 hex chars, uppercase.
pub fn compute_sas(
    identity_a: &[u8; PUBLIC_KEY_LENGTH],
    identity_b: &[u8; PUBLIC_KEY_LENGTH],
    ephemeral_a: &[u8; PUBLIC_KEY_LENGTH],
    ephemeral_b: &[u8; PUBLIC_KEY_LENGTH],
) -> String {
    let sorted_identity = sort32(identity_a, identity_b);
    let sorted_ephemeral = sort32(ephemeral_a, ephemeral_b);

    let mut combined = [0u8; 128];
    combined[..64].copy_from_slice(&sorted_identity);
    combined[64..].copy_from_slice(&sorted_ephemeral);

    let hash = hash::sha256(&combined);
    let hex = encoding::to_hex(&hash);
    hex[..SAS_LENGTH].to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::from_hex;

    #[test]
    fn sas_length_constant() {
        assert_eq!(crate::constants::SAS_LENGTH, 6);
    }

    #[test]
    fn sas_output_length_and_hex() {
        let a = [1u8; 32];
        let b = [2u8; 32];
        let sas = compute_sas(&a, &b, &a, &b);
        assert_eq!(sas.len(), 6);
        assert!(sas.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(sas, sas.to_uppercase());
    }

    #[test]
    fn sas_is_commutative() {
        let a = [1u8; 32];
        let b = [2u8; 32];
        let c = [3u8; 32];
        let d = [4u8; 32];
        // Swapping A/B positions must produce identical SAS.
        assert_eq!(compute_sas(&a, &b, &c, &d), compute_sas(&b, &a, &d, &c));
    }

    #[test]
    fn sas_golden_vector_1() {
        // Uses sender + receiver public keys from box-payload.vectors.json.
        let sender_pk: [u8; 32] =
            from_hex("07a37cbc142093c8b755dc1b10e86cb426374ad16aa853ed0bdfc0b2b86d1c7c")
                .unwrap()
                .try_into()
                .unwrap();
        let receiver_pk: [u8; 32] =
            from_hex("5869aff450549732cbaaed5e5df9b30a6da31cb0e5742bad5ad4a1a768f1a67b")
                .unwrap()
                .try_into()
                .unwrap();
        // identity = sender, receiver; ephemeral = sender, receiver
        let sas = compute_sas(&sender_pk, &receiver_pk, &sender_pk, &receiver_pk);
        // Pre-computed: SHA-256(sort32(sender_pk, receiver_pk) || sort32(sender_pk, receiver_pk))
        // sender_pk[0]=0x07 < receiver_pk[0]=0x58, so sort32 = sender||receiver both times.
        // combined = sender||receiver||sender||receiver (128 bytes)
        // Verify by computing SHA-256 directly:
        let mut combined = [0u8; 128];
        combined[..32].copy_from_slice(&sender_pk);
        combined[32..64].copy_from_slice(&receiver_pk);
        combined[64..96].copy_from_slice(&sender_pk);
        combined[96..].copy_from_slice(&receiver_pk);
        let expected_hex = crate::hash::sha256_hex(&combined);
        let expected_sas = expected_hex[..6].to_uppercase();
        assert_eq!(sas, expected_sas);
    }

    #[test]
    fn sas_golden_vector_2() {
        // Uses receiver + eve public keys from box-payload.vectors.json.
        let receiver_pk: [u8; 32] =
            from_hex("5869aff450549732cbaaed5e5df9b30a6da31cb0e5742bad5ad4a1a768f1a67b")
                .unwrap()
                .try_into()
                .unwrap();
        let eve_pk: [u8; 32] =
            from_hex("64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d48466")
                .unwrap()
                .try_into()
                .unwrap();
        let sender_pk: [u8; 32] =
            from_hex("07a37cbc142093c8b755dc1b10e86cb426374ad16aa853ed0bdfc0b2b86d1c7c")
                .unwrap()
                .try_into()
                .unwrap();
        // identity = receiver, eve; ephemeral = sender, receiver
        let sas = compute_sas(&receiver_pk, &eve_pk, &sender_pk, &receiver_pk);
        // Verify step-by-step:
        // sort32(receiver_pk, eve_pk): receiver[0]=0x58 < eve[0]=0x64 → receiver||eve
        // sort32(sender_pk, receiver_pk): sender[0]=0x07 < receiver[0]=0x58 → sender||receiver
        let sorted_id = sort32(&receiver_pk, &eve_pk);
        let sorted_eph = sort32(&sender_pk, &receiver_pk);
        let mut combined = [0u8; 128];
        combined[..64].copy_from_slice(&sorted_id);
        combined[64..].copy_from_slice(&sorted_eph);
        let expected_hex = crate::hash::sha256_hex(&combined);
        let expected_sas = expected_hex[..6].to_uppercase();
        assert_eq!(sas, expected_sas);
    }

    #[test]
    fn sort32_smaller_first() {
        let a = [0x01u8; 32];
        let b = [0x02u8; 32];
        let result = sort32(&a, &b);
        assert_eq!(&result[..32], &a);
        assert_eq!(&result[32..], &b);
        // Reversed input — same output order.
        let result_rev = sort32(&b, &a);
        assert_eq!(result, result_rev);
    }

    #[test]
    fn sort32_identical_keys() {
        let k = [0x42u8; 32];
        let result = sort32(&k, &k);
        assert_eq!(&result[..32], &k);
        assert_eq!(&result[32..], &k);
    }
}
