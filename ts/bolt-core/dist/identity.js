import tweetnacl from 'tweetnacl';
const { box } = tweetnacl;
import { BoltError } from './errors.js';
/**
 * Generate a persistent identity keypair (X25519).
 *
 * Identity keys are long-lived and stored by the transport layer.
 * They MUST NOT be sent through the signaling server — identity
 * material travels only inside encrypted DataChannel messages (HELLO).
 */
export function generateIdentityKeyPair() {
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
