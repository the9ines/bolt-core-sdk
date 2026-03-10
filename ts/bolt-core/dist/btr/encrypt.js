/**
 * BTR encryption — NaCl secretbox keyed by BTR message_key (§16.4).
 *
 * Uses symmetric NaCl secretbox (XSalsa20-Poly1305), NOT asymmetric box.
 * Both peers derive identical message_key deterministically via HKDF.
 * Fresh 24-byte CSPRNG nonce per envelope (production) or fixed nonce (test vectors).
 */
import tweetnacl from 'tweetnacl';
import { ratchetDecryptFail } from './errors.js';
/** NaCl secretbox nonce length (24 bytes). */
const SECRETBOX_NONCE_LENGTH = 24;
/** NaCl secretbox MAC overhead (Poly1305, 16 bytes). */
const SECRETBOX_OVERHEAD = 16;
/**
 * Encrypt a chunk using NaCl secretbox with a BTR-derived message_key.
 *
 * Returns nonce || ciphertext (24 + plaintext.length + 16 bytes).
 * Fresh 24-byte CSPRNG nonce generated internally.
 */
export function btrSeal(messageKey, plaintext) {
    const nonce = tweetnacl.randomBytes(SECRETBOX_NONCE_LENGTH);
    const ciphertext = tweetnacl.secretbox(plaintext, nonce, messageKey);
    const combined = new Uint8Array(SECRETBOX_NONCE_LENGTH + ciphertext.length);
    combined.set(nonce);
    combined.set(ciphertext, SECRETBOX_NONCE_LENGTH);
    return combined;
}
/**
 * Encrypt with a caller-provided nonce (deterministic vectors only).
 * NOT for production — nonce reuse with the same key is catastrophic.
 */
export function btrSealDeterministic(messageKey, plaintext, nonce) {
    const ciphertext = tweetnacl.secretbox(plaintext, nonce, messageKey);
    const combined = new Uint8Array(SECRETBOX_NONCE_LENGTH + ciphertext.length);
    combined.set(nonce);
    combined.set(ciphertext, SECRETBOX_NONCE_LENGTH);
    return combined;
}
/**
 * Decrypt a chunk using NaCl secretbox with a BTR-derived message_key.
 *
 * Expects nonce || ciphertext format (first 24 bytes are nonce).
 * Throws BtrError with RATCHET_DECRYPT_FAIL on MAC failure or truncation.
 */
export function btrOpen(messageKey, sealed) {
    if (sealed.length < SECRETBOX_NONCE_LENGTH + SECRETBOX_OVERHEAD) {
        throw ratchetDecryptFail('sealed payload too short');
    }
    const nonce = sealed.slice(0, SECRETBOX_NONCE_LENGTH);
    const ciphertext = sealed.slice(SECRETBOX_NONCE_LENGTH);
    const plaintext = tweetnacl.secretbox.open(ciphertext, nonce, messageKey);
    if (plaintext === null) {
        throw ratchetDecryptFail('secretbox open failed');
    }
    return plaintext;
}
