import { BoltError } from './errors.js';
/** Long-lived X25519 identity keypair. Persisted by the transport layer. */
export interface IdentityKeyPair {
    publicKey: Uint8Array;
    secretKey: Uint8Array;
}
/**
 * Generate a persistent identity keypair (X25519).
 *
 * Identity keys are long-lived and stored by the transport layer.
 * They MUST NOT be sent through the signaling server — identity
 * material travels only inside encrypted DataChannel messages (HELLO).
 */
export declare function generateIdentityKeyPair(): IdentityKeyPair;
/**
 * Thrown when a peer's identity public key does not match a previously
 * pinned value. This is a TOFU violation — the session MUST be aborted.
 */
export declare class KeyMismatchError extends BoltError {
    readonly peerCode: string;
    readonly expected: Uint8Array;
    readonly received: Uint8Array;
    constructor(peerCode: string, expected: Uint8Array, received: Uint8Array);
}
