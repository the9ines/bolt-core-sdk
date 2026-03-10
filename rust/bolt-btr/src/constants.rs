//! BTR-specific constants — HKDF info strings and key lengths.
//!
//! All values locked from PROTOCOL.md §14. No divergence permitted.

/// HKDF info string for session root derivation (§16.3).
pub const BTR_SESSION_ROOT_INFO: &[u8] = b"bolt-btr-session-root-v1";

/// HKDF info string for transfer root derivation (§16.3).
pub const BTR_TRANSFER_ROOT_INFO: &[u8] = b"bolt-btr-transfer-root-v1";

/// HKDF info string for message key derivation (§16.3).
pub const BTR_MESSAGE_KEY_INFO: &[u8] = b"bolt-btr-message-key-v1";

/// HKDF info string for chain key advancement (§16.3).
pub const BTR_CHAIN_ADVANCE_INFO: &[u8] = b"bolt-btr-chain-advance-v1";

/// HKDF info string for DH ratchet step (§16.3).
pub const BTR_DH_RATCHET_INFO: &[u8] = b"bolt-btr-dh-ratchet-v1";

/// BTR key length in bytes (all derived keys).
pub const BTR_KEY_LENGTH: usize = 32;

/// BTR wire error codes (§16.7, extends PROTOCOL.md §10 registry).
pub const BTR_WIRE_ERROR_CODES: [&str; 4] = [
    "RATCHET_STATE_ERROR",
    "RATCHET_CHAIN_ERROR",
    "RATCHET_DECRYPT_FAIL",
    "RATCHET_DOWNGRADE_REJECTED",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn info_strings_match_spec() {
        assert_eq!(BTR_SESSION_ROOT_INFO, b"bolt-btr-session-root-v1");
        assert_eq!(BTR_TRANSFER_ROOT_INFO, b"bolt-btr-transfer-root-v1");
        assert_eq!(BTR_MESSAGE_KEY_INFO, b"bolt-btr-message-key-v1");
        assert_eq!(BTR_CHAIN_ADVANCE_INFO, b"bolt-btr-chain-advance-v1");
        assert_eq!(BTR_DH_RATCHET_INFO, b"bolt-btr-dh-ratchet-v1");
    }

    #[test]
    fn key_length_is_32() {
        assert_eq!(BTR_KEY_LENGTH, 32);
    }

    #[test]
    fn btr_wire_codes_count() {
        assert_eq!(BTR_WIRE_ERROR_CODES.len(), 4);
    }

    #[test]
    fn btr_wire_codes_present_in_core_registry() {
        // BTR codes were added to bolt-core's canonical registry (22 → 26).
        for code in &BTR_WIRE_ERROR_CODES {
            assert!(
                bolt_core::errors::is_valid_wire_error_code(code),
                "BTR code {code} missing from bolt-core wire error registry"
            );
        }
    }
}
