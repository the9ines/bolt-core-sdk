//! Peer code generation and validation.
//!
//! Peer codes are short human-readable identifiers used as routing
//! hints (NOT authentication secrets — see PROTOCOL.md §2).
//!
//! ## Algorithm
//! Rejection sampling from 31-character unambiguous alphabet.
//! `REJECTION_MAX = floor(256 / 31) * 31 = 248`. Bytes >= 248 are
//! discarded to eliminate modulo bias.
//!
//! ## Parity gates (R3)
//! - Generated codes pass `is_valid_peer_code`.
//! - `normalize_peer_code` matches TS for all test inputs.
//! - Rejection sampling bias test (chi-squared, 100K samples).
//! - Long code format: `XXXX-XXXX`.

use crate::constants::PEER_CODE_ALPHABET;

/// Generate a cryptographically secure 6-character peer code.
///
/// Uses rejection sampling to eliminate modulo bias.
///
/// # Parity
/// TS equivalent: `generateSecurePeerCode()`.
/// Outputs differ (random), but alphabet and length are identical.
pub fn generate_secure_peer_code() -> String {
    todo!("R3: implement rejection sampling with OsRng")
}

/// Generate a longer peer code with dash separator.
///
/// Format: `XXXX-XXXX` (~40 bits of entropy).
///
/// # Parity
/// TS equivalent: `generateLongPeerCode()`.
pub fn generate_long_peer_code() -> String {
    todo!("R3: implement rejection sampling with OsRng, dash at position 4")
}

/// Validate peer code format.
///
/// Accepts 6-char or 8-char (with optional dash) codes using the
/// unambiguous alphabet.
///
/// # Parity
/// TS equivalent: `isValidPeerCode(code)`. Must return identical
/// results for all inputs in the golden vector suite.
pub fn is_valid_peer_code(code: &str) -> bool {
    let normalized = code.replace('-', "").to_uppercase();
    if normalized.len() != 6 && normalized.len() != 8 {
        return false;
    }
    normalized.chars().all(|c| PEER_CODE_ALPHABET.contains(c))
}

/// Normalize peer code for comparison (remove dashes, uppercase).
///
/// # Parity
/// TS equivalent: `normalizePeerCode(code)`.
pub fn normalize_peer_code(code: &str) -> String {
    code.replace('-', "").to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_6_char() {
        assert!(is_valid_peer_code("ABCDEF"));
    }

    #[test]
    fn valid_8_char_with_dash() {
        assert!(is_valid_peer_code("ABCD-EFGH"));
    }

    #[test]
    fn valid_8_char_no_dash() {
        assert!(is_valid_peer_code("ABCDEFGH"));
    }

    #[test]
    fn valid_lowercase_accepted() {
        assert!(is_valid_peer_code("abcdef"));
    }

    #[test]
    fn invalid_empty() {
        assert!(!is_valid_peer_code(""));
    }

    #[test]
    fn invalid_wrong_length() {
        assert!(!is_valid_peer_code("ABC"));
        assert!(!is_valid_peer_code("ABCDEFGHIJK"));
    }

    #[test]
    fn invalid_ambiguous_chars() {
        // 0 and O are excluded from alphabet
        assert!(!is_valid_peer_code("A0CDEF"));
        assert!(!is_valid_peer_code("AOCDEF"));
        // 1, I, L are excluded
        assert!(!is_valid_peer_code("A1CDEF"));
        assert!(!is_valid_peer_code("AICDEF"));
        assert!(!is_valid_peer_code("ALCDEF"));
    }

    #[test]
    fn normalize_strips_dashes_and_uppercases() {
        assert_eq!(normalize_peer_code("abcd-efgh"), "ABCDEFGH");
        assert_eq!(normalize_peer_code("ABC-DEF"), "ABCDEF");
        assert_eq!(normalize_peer_code("xyzw"), "XYZW");
    }
}
