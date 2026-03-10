/**
 * BTR-specific constants — HKDF info strings and key lengths (§14).
 *
 * All values locked from PROTOCOL.md §14. No divergence permitted.
 * Must match Rust bolt-btr/src/constants.rs exactly.
 */
/** HKDF info string for session root derivation (§16.3). */
export declare const BTR_SESSION_ROOT_INFO = "bolt-btr-session-root-v1";
/** HKDF info string for transfer root derivation (§16.3). */
export declare const BTR_TRANSFER_ROOT_INFO = "bolt-btr-transfer-root-v1";
/** HKDF info string for message key derivation (§16.3). */
export declare const BTR_MESSAGE_KEY_INFO = "bolt-btr-message-key-v1";
/** HKDF info string for chain key advancement (§16.3). */
export declare const BTR_CHAIN_ADVANCE_INFO = "bolt-btr-chain-advance-v1";
/** HKDF info string for DH ratchet step (§16.3). */
export declare const BTR_DH_RATCHET_INFO = "bolt-btr-dh-ratchet-v1";
/** BTR key length in bytes (all derived keys). */
export declare const BTR_KEY_LENGTH = 32;
/** BTR wire error codes (§16.7, extends PROTOCOL.md §10 registry). */
export declare const BTR_WIRE_ERROR_CODES: readonly ["RATCHET_STATE_ERROR", "RATCHET_CHAIN_ERROR", "RATCHET_DECRYPT_FAIL", "RATCHET_DOWNGRADE_REJECTED"];
