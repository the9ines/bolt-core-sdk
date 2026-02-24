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
//! ## Parity
//! - Generated codes pass `is_valid_peer_code`.
//! - `normalize_peer_code` matches TS for all test inputs.
//! - Long code format: `XXXX-XXXX`.

use rand_core::{OsRng, RngCore};

use crate::constants::PEER_CODE_ALPHABET;

/// Rejection sampling threshold: largest multiple of 31 that fits in a byte.
/// Bytes >= 248 are discarded to eliminate modulo bias.
const REJECTION_MAX: u8 = 248; // floor(256 / 31) * 31

/// Fill a buffer with `count` unbiased alphabet characters via rejection sampling.
///
/// Matches TS `fillUnbiased()` exactly: request random bytes in batches,
/// discard bytes >= REJECTION_MAX, use `byte % 31` for survivors.
fn fill_unbiased(count: usize) -> String {
    let alphabet = PEER_CODE_ALPHABET.as_bytes();
    let n = alphabet.len(); // 31
    let mut result = String::with_capacity(count);
    while result.len() < count {
        let needed = count - result.len() + 4; // small over-request like TS
        let mut batch = vec![0u8; needed];
        OsRng.fill_bytes(&mut batch);
        for &byte in &batch {
            if result.len() >= count {
                break;
            }
            if byte < REJECTION_MAX {
                result.push(alphabet[byte as usize % n] as char);
            }
        }
    }
    result
}

/// Generate a cryptographically secure 6-character peer code.
///
/// Uses rejection sampling to eliminate modulo bias.
///
/// # Parity
/// TS equivalent: `generateSecurePeerCode()`.
/// Outputs differ (random), but alphabet, length, and sampling
/// algorithm are identical.
pub fn generate_secure_peer_code() -> String {
    fill_unbiased(6)
}

/// Generate a longer peer code with dash separator.
///
/// Format: `XXXX-XXXX` (~40 bits of entropy).
///
/// # Parity
/// TS equivalent: `generateLongPeerCode()`.
pub fn generate_long_peer_code() -> String {
    let chars = fill_unbiased(8);
    let mut result = String::with_capacity(9);
    result.push_str(&chars[..4]);
    result.push('-');
    result.push_str(&chars[4..]);
    result
}

/// Validate peer code format.
///
/// Accepts 6-char or 8-char (with optional dash) codes using the
/// unambiguous alphabet.
///
/// # Parity
/// TS equivalent: `isValidPeerCode(code)`. Must return identical
/// results for all inputs.
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

    // ── validation (existing) ───────────────────────────────────────

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
        assert!(!is_valid_peer_code("A0CDEF"));
        assert!(!is_valid_peer_code("AOCDEF"));
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

    // ── generation (R3) ─────────────────────────────────────────────

    #[test]
    fn secure_peer_code_length() {
        let code = generate_secure_peer_code();
        assert_eq!(code.len(), 6);
    }

    #[test]
    fn secure_peer_code_chars_in_alphabet() {
        let code = generate_secure_peer_code();
        for c in code.chars() {
            assert!(PEER_CODE_ALPHABET.contains(c), "char '{c}' not in alphabet");
        }
    }

    #[test]
    fn secure_peer_code_passes_validation() {
        let code = generate_secure_peer_code();
        assert!(is_valid_peer_code(&code));
    }

    #[test]
    fn long_peer_code_format() {
        let code = generate_long_peer_code();
        assert_eq!(code.len(), 9);
        assert_eq!(code.as_bytes()[4], b'-');
    }

    #[test]
    fn long_peer_code_chars_in_alphabet() {
        let code = generate_long_peer_code();
        for (i, c) in code.chars().enumerate() {
            if i == 4 {
                assert_eq!(c, '-');
            } else {
                assert!(
                    PEER_CODE_ALPHABET.contains(c),
                    "char '{c}' at index {i} not in alphabet"
                );
            }
        }
    }

    #[test]
    fn long_peer_code_passes_validation() {
        let code = generate_long_peer_code();
        assert!(is_valid_peer_code(&code));
    }

    #[test]
    fn rejection_max_constant() {
        // floor(256 / 31) * 31 = 8 * 31 = 248
        assert_eq!(REJECTION_MAX, 248);
    }
}
