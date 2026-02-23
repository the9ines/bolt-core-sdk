import tweetnacl from 'tweetnacl';
const { box } = tweetnacl;
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
export function generateIdentityKeyPair(): IdentityKeyPair {
  return box.keyPair();
}

/**
 * Thrown when a peer's identity public key does not match a previously
 * pinned value. This is a TOFU violation — the session MUST be aborted.
 */
export class KeyMismatchError extends BoltError {
  constructor(
    public readonly peerCode: string,
    public readonly expected: Uint8Array,
    public readonly received: Uint8Array,
  ) {
    super(`Identity key mismatch for peer ${peerCode}`);
    this.name = 'KeyMismatchError';
  }
}
