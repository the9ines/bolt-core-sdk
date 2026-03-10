/**
 * BTR error types — §16.7 error behavior.
 *
 * Four error codes with deterministic behavior mapping.
 * Must match Rust bolt-btr/src/errors.rs semantics exactly.
 */

import { BTR_WIRE_ERROR_CODES } from './constants.js';

type BtrWireErrorCode = (typeof BTR_WIRE_ERROR_CODES)[number];

/** BTR-specific error — carries a wire error code and disconnect semantics. */
export class BtrError extends Error {
  /** Canonical wire error code from §16.7. */
  readonly wireCode: BtrWireErrorCode;

  constructor(wireCode: BtrWireErrorCode, message: string) {
    super(`${wireCode}: ${message}`);
    this.name = 'BtrError';
    this.wireCode = wireCode;
  }

  /**
   * Returns true if the required action is disconnect (vs cancel transfer).
   * Matches Rust BtrError::requires_disconnect().
   */
  requiresDisconnect(): boolean {
    return (
      this.wireCode === 'RATCHET_STATE_ERROR' ||
      this.wireCode === 'RATCHET_DOWNGRADE_REJECTED'
    );
  }
}

/** Ratchet generation mismatch, unexpected DH key, or missing BTR fields. Action: disconnect. */
export function ratchetStateError(detail: string): BtrError {
  return new BtrError('RATCHET_STATE_ERROR', detail);
}

/** chain_index != expected next, chain index gap, or replay. Action: cancel transfer. */
export function ratchetChainError(detail: string): BtrError {
  return new BtrError('RATCHET_CHAIN_ERROR', detail);
}

/** NaCl secretbox open fails with BTR message key. Action: cancel transfer. */
export function ratchetDecryptFail(detail: string): BtrError {
  return new BtrError('RATCHET_DECRYPT_FAIL', detail);
}

/** Peer advertised BTR but sends invalid envelopes. Action: disconnect. */
export function ratchetDowngradeRejected(detail: string): BtrError {
  return new BtrError('RATCHET_DOWNGRADE_REJECTED', detail);
}
