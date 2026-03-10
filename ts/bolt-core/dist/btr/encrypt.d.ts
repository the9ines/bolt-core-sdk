/**
 * BTR encryption — NaCl secretbox keyed by BTR message_key (§16.4).
 *
 * Uses symmetric NaCl secretbox (XSalsa20-Poly1305), NOT asymmetric box.
 * Both peers derive identical message_key deterministically via HKDF.
 * Fresh 24-byte CSPRNG nonce per envelope (production) or fixed nonce (test vectors).
 */
/**
 * Encrypt a chunk using NaCl secretbox with a BTR-derived message_key.
 *
 * Returns nonce || ciphertext (24 + plaintext.length + 16 bytes).
 * Fresh 24-byte CSPRNG nonce generated internally.
 */
export declare function btrSeal(messageKey: Uint8Array, plaintext: Uint8Array): Uint8Array;
/**
 * Encrypt with a caller-provided nonce (deterministic vectors only).
 * NOT for production — nonce reuse with the same key is catastrophic.
 */
export declare function btrSealDeterministic(messageKey: Uint8Array, plaintext: Uint8Array, nonce: Uint8Array): Uint8Array;
/**
 * Decrypt a chunk using NaCl secretbox with a BTR-derived message_key.
 *
 * Expects nonce || ciphertext format (first 24 bytes are nonce).
 * Throws BtrError with RATCHET_DECRYPT_FAIL on MAC failure or truncation.
 */
export declare function btrOpen(messageKey: Uint8Array, sealed: Uint8Array): Uint8Array;
