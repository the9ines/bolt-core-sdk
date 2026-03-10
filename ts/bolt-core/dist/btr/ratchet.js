/**
 * Inter-transfer DH ratchet — §16.3 DH ratchet step.
 *
 * Uses tweetnacl.scalarMult for X25519 DH (parity with Rust x25519-dalek).
 * Uses HKDF-SHA256 for session root key derivation after DH.
 */
import tweetnacl from 'tweetnacl';
import { hkdf } from '@noble/hashes/hkdf.js';
import { sha256 } from '@noble/hashes/sha2.js';
import { BTR_DH_RATCHET_INFO, BTR_KEY_LENGTH } from './constants.js';
const encoder = new TextEncoder();
/**
 * Generate a fresh X25519 keypair for the DH ratchet step.
 *
 * Returns { publicKey, secretKey } both 32 bytes.
 * The secretKey should be consumed once for DH then discarded.
 */
export function generateRatchetKeypair() {
    // tweetnacl.box.keyPair() generates X25519 keypairs
    const kp = tweetnacl.box.keyPair();
    return { publicKey: kp.publicKey, secretKey: kp.secretKey };
}
/**
 * Perform X25519 DH with a remote peer's ratchet public key.
 *
 * Returns the raw 32-byte DH output.
 * Matches Rust: x25519-dalek EphemeralSecret::diffie_hellman().
 */
export function scalarMult(localSecretKey, remotePublicKey) {
    return tweetnacl.scalarMult(localSecretKey, remotePublicKey);
}
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
export function deriveRatchetedSessionRoot(currentSessionRootKey, dhOutput) {
    return hkdf(sha256, dhOutput, currentSessionRootKey, encoder.encode(BTR_DH_RATCHET_INFO), BTR_KEY_LENGTH);
}
