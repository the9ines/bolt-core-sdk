/**
 * Inter-transfer DH ratchet — §16.3 DH ratchet step.
 *
 * Uses tweetnacl.scalarMult for X25519 DH (parity with Rust x25519-dalek).
 * Uses HKDF-SHA256 for session root key derivation after DH.
 */
/**
 * Generate a fresh X25519 keypair for the DH ratchet step.
 *
 * Returns { publicKey, secretKey } both 32 bytes.
 * The secretKey should be consumed once for DH then discarded.
 */
export declare function generateRatchetKeypair(): {
    publicKey: Uint8Array;
    secretKey: Uint8Array;
};
/**
 * Perform X25519 DH with a remote peer's ratchet public key.
 *
 * Returns the raw 32-byte DH output.
 * Matches Rust: x25519-dalek EphemeralSecret::diffie_hellman().
 */
export declare function scalarMult(localSecretKey: Uint8Array, remotePublicKey: Uint8Array): Uint8Array;
/**
 * Derive new session root key from DH ratchet step output (§16.3).
 *
 * new_session_root_key = HKDF-SHA256(
 *   salt  = current_session_root_key,
 *   ikm   = dh_output,
 *   info  = "bolt-btr-dh-ratchet-v1",
 *   len   = 32
 * )
 */
export declare function deriveRatchetedSessionRoot(currentSessionRootKey: Uint8Array, dhOutput: Uint8Array): Uint8Array;
