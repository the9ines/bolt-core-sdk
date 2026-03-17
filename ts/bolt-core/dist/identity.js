import tweetnacl from 'tweetnacl';
const { box } = tweetnacl;
import { BoltError } from './errors.js';
import { getWasmCrypto } from './wasm-crypto.js';
/**
 * Generate a persistent identity keypair (X25519).
 *
 * RB3: Uses Rust/WASM when available, falls back to tweetnacl.
 */
export function generateIdentityKeyPair() {
    const wasm = getWasmCrypto();
    if (wasm)
        return wasm.generateIdentityKeyPair();
    return box.keyPair();
}
/**
 * Thrown when a peer's identity public key does not match a previously
 * pinned value. This is a TOFU violation — the session MUST be aborted.
 */
export class KeyMismatchError extends BoltError {
    constructor(peerCode, expected, received) {
        super(`Identity key mismatch for peer ${peerCode}`);
        this.peerCode = peerCode;
        this.expected = expected;
        this.received = received;
        this.name = 'KeyMismatchError';
    }
}
