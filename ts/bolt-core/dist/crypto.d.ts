/**
 * Generate a fresh ephemeral X25519 keypair for a single connection.
 * Discard after session ends.
 *
 * RB3: Uses Rust/WASM when available, falls back to tweetnacl.
 */
export declare function generateEphemeralKeyPair(): {
    publicKey: Uint8Array;
    secretKey: Uint8Array;
};
/**
 * Seal a plaintext payload using NaCl box (XSalsa20-Poly1305).
 *
 * Wire format: base64(nonce || ciphertext)
 *
 * RB3: Uses Rust/WASM when available, falls back to tweetnacl.
 */
export declare function sealBoxPayload(plaintext: Uint8Array, remotePublicKey: Uint8Array, senderSecretKey: Uint8Array): string;
/**
 * Open a sealed payload using NaCl box.open.
 *
 * Expects wire format: base64(nonce || ciphertext)
 *
 * RB3: Uses Rust/WASM when available, falls back to tweetnacl.
 */
export declare function openBoxPayload(sealed: string, senderPublicKey: Uint8Array, receiverSecretKey: Uint8Array): Uint8Array;
