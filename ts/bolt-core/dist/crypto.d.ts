/**
 * Generate a fresh ephemeral X25519 keypair for a single connection.
 * Discard after session ends.
 */
export declare function generateEphemeralKeyPair(): {
    publicKey: Uint8Array;
    secretKey: Uint8Array;
};
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
export declare function sealBoxPayload(plaintext: Uint8Array, remotePublicKey: Uint8Array, senderSecretKey: Uint8Array): string;
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
export declare function openBoxPayload(sealed: string, senderPublicKey: Uint8Array, receiverSecretKey: Uint8Array): Uint8Array;
