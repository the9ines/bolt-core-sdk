import tweetnacl from 'tweetnacl';
const { box, randomBytes } = tweetnacl;
import { toBase64, fromBase64 } from './encoding.js';
import { EncryptionError } from './errors.js';
import { getWasmCrypto } from './wasm-crypto.js';
/**
 * Generate a fresh ephemeral X25519 keypair for a single connection.
 * Discard after session ends.
 *
 * RB3: Uses Rust/WASM when available, falls back to tweetnacl.
 */
export function generateEphemeralKeyPair() {
    const wasm = getWasmCrypto();
    if (wasm)
        return wasm.generateEphemeralKeyPair();
    return box.keyPair();
}
/**
 * Seal a plaintext payload using NaCl box (XSalsa20-Poly1305).
 *
 * Wire format: base64(nonce || ciphertext)
 *
 * RB3: Uses Rust/WASM when available, falls back to tweetnacl.
 */
export function sealBoxPayload(plaintext, remotePublicKey, senderSecretKey) {
    const wasm = getWasmCrypto();
    if (wasm)
        return wasm.sealBoxPayload(plaintext, remotePublicKey, senderSecretKey);
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
 * RB3: Uses Rust/WASM when available, falls back to tweetnacl.
 */
export function openBoxPayload(sealed, senderPublicKey, receiverSecretKey) {
    const wasm = getWasmCrypto();
    if (wasm)
        return wasm.openBoxPayload(sealed, senderPublicKey, receiverSecretKey);
    const data = fromBase64(sealed);
    if (data.length < box.nonceLength) {
        throw new EncryptionError('Sealed payload too short');
    }
    const nonce = data.slice(0, box.nonceLength);
    const ciphertext = data.slice(box.nonceLength);
    const decrypted = box.open(ciphertext, nonce, senderPublicKey, receiverSecretKey);
    if (!decrypted)
        throw new EncryptionError('Decryption failed');
    return decrypted;
}
