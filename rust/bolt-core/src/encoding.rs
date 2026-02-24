//! Encoding utilities — base64 and hex.
//!
//! TS uses `tweetnacl-util` for base64 (standard alphabet, with padding).
//! Rust uses the `base64` crate with STANDARD engine (RFC 4648).
//!
//! ## Parity
//! - `to_base64(bytes) == TS toBase64(bytes)` for all vector payloads.
//! - `to_hex(bytes) == TS bufferToHex(bytes)` for all vector nonces/keys.
//! - Round-trip: `from_base64(to_base64(x)) == x` for arbitrary input.

use base64::engine::general_purpose::STANDARD;
use base64::Engine;

use crate::errors::BoltError;

/// Encode bytes to standard base64 (RFC 4648, with padding).
///
/// # Parity
/// Produces identical output to TS `toBase64(data: Uint8Array)`.
pub fn to_base64(data: &[u8]) -> String {
    STANDARD.encode(data)
}

/// Decode standard base64 to bytes.
///
/// # Parity
/// Accepts all outputs of TS `toBase64` and returns identical bytes.
///
/// # Errors
/// Returns `BoltError::Encoding` on invalid base64 input.
pub fn from_base64(encoded: &str) -> Result<Vec<u8>, BoltError> {
    STANDARD
        .decode(encoded)
        .map_err(|e| BoltError::Encoding(format!("invalid base64: {e}")))
}

/// Encode bytes to lowercase hex string.
///
/// # Parity
/// Produces identical output to TS `bufferToHex(buffer: ArrayBuffer)`.
pub fn to_hex(data: &[u8]) -> String {
    data.iter().map(|b| format!("{b:02x}")).collect()
}

/// Decode hex string to bytes.
///
/// # Errors
/// Returns `BoltError::Encoding` on invalid hex input.
pub fn from_hex(encoded: &str) -> Result<Vec<u8>, BoltError> {
    if !encoded.len().is_multiple_of(2) {
        return Err(BoltError::Encoding("odd-length hex string".into()));
    }
    (0..encoded.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&encoded[i..i + 2], 16)
                .map_err(|e| BoltError::Encoding(format!("invalid hex: {e}")))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── base64 ──────────────────────────────────────────────────────

    #[test]
    fn base64_round_trip() {
        let input = b"Hello, Bolt!";
        let encoded = to_base64(input);
        let decoded = from_base64(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn base64_empty() {
        assert_eq!(to_base64(&[]), "");
        assert_eq!(from_base64("").unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn base64_known_value() {
        // Standard base64 of "Hello, Bolt!" — matches TS toBase64 output.
        assert_eq!(to_base64(b"Hello, Bolt!"), "SGVsbG8sIEJvbHQh");
        assert_eq!(from_base64("SGVsbG8sIEJvbHQh").unwrap(), b"Hello, Bolt!");
    }

    #[test]
    fn base64_decode_rejects_invalid() {
        assert!(from_base64("!!!invalid!!!").is_err());
    }

    #[test]
    fn base64_all_bytes_round_trip() {
        let input: Vec<u8> = (0..=255).collect();
        let encoded = to_base64(&input);
        let decoded = from_base64(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    // ── hex ─────────────────────────────────────────────────────────

    #[test]
    fn hex_round_trip() {
        let input = b"Hello, Bolt!";
        let hex = to_hex(input);
        let decoded = from_hex(&hex).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn hex_empty() {
        assert_eq!(to_hex(&[]), "");
        assert_eq!(from_hex("").unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn hex_known_value() {
        assert_eq!(to_hex(&[0xff]), "ff");
        assert_eq!(to_hex(&[0x00, 0x0a, 0xff]), "000aff");
    }

    #[test]
    fn hex_odd_length_rejected() {
        assert!(from_hex("abc").is_err());
    }

    #[test]
    fn hex_invalid_chars_rejected() {
        assert!(from_hex("zzzz").is_err());
    }
}
