import { box, randomBytes } from 'tweetnacl';
import { toBase64, fromBase64 } from './encoding.js';
import { EncryptionError } from './errors.js';
/**
 * Generate a fresh ephemeral X25519 keypair for a single connection.
 * Discard after session ends.
 */
export function generateEphemeralKeyPair() {
    return box.keyPair();
}
/**
 * Seal a plaintext payload using NaCl box (XSalsa20-Poly1305).
 *
 * Wire format: base64(nonce || ciphertext)
 * This matches the exact format used by all current product repos.
 *
 * @param plaintext - Raw bytes to encrypt
 * @param remotePublicKey - Receiver's ephemeral public key (32 bytes)
 * @param senderSecretKey - Sender's ephemeral secret key (32 bytes)
 * @returns base64-encoded string of nonce + ciphertext
 */
export function sealBoxPayload(plaintext, remotePublicKey, senderSecretKey) {
    const nonce = randomBytes(box.nonceLength);
    const encrypted = box(plaintext, nonce, remotePublicKey, senderSecretKey);
    if (!encrypted)
        throw new EncryptionError('Encryption returned null');
    const combined = new Uint8Array(nonce.length + encrypted.length);
    combined.set(nonce);
    combined.set(encrypted, nonce.length);
    return toBase64(combined);
}
/**
 * Open a sealed payload using NaCl box.open.
 *
 * Expects wire format: base64(nonce || ciphertext)
 *
 * @param sealed - base64-encoded string from sealBoxPayload
 * @param senderPublicKey - Sender's ephemeral public key (32 bytes)
 * @param receiverSecretKey - Receiver's ephemeral secret key (32 bytes)
 * @returns Decrypted plaintext bytes
 */
export function openBoxPayload(sealed, senderPublicKey, receiverSecretKey) {
    const data = fromBase64(sealed);
    const nonce = data.slice(0, box.nonceLength);
    const ciphertext = data.slice(box.nonceLength);
    const decrypted = box.open(ciphertext, nonce, senderPublicKey, receiverSecretKey);
    if (!decrypted)
        throw new EncryptionError('Decryption failed');
    return decrypted;
}
