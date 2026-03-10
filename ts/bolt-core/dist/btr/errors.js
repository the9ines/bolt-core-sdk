/**
 * BTR error types — §16.7 error behavior.
 *
 * Four error codes with deterministic behavior mapping.
 * Must match Rust bolt-btr/src/errors.rs semantics exactly.
 */
/** BTR-specific error — carries a wire error code and disconnect semantics. */
export class BtrError extends Error {
    constructor(wireCode, message) {
        super(`${wireCode}: ${message}`);
        this.name = 'BtrError';
        this.wireCode = wireCode;
    }
    /**
     * Returns true if the required action is disconnect (vs cancel transfer).
     * Matches Rust BtrError::requires_disconnect().
     */
    requiresDisconnect() {
        return (this.wireCode === 'RATCHET_STATE_ERROR' ||
            this.wireCode === 'RATCHET_DOWNGRADE_REJECTED');
    }
}
/** Ratchet generation mismatch, unexpected DH key, or missing BTR fields. Action: disconnect. */
export function ratchetStateError(detail) {
    return new BtrError('RATCHET_STATE_ERROR', detail);
}
/** chain_index != expected next, chain index gap, or replay. Action: cancel transfer. */
export function ratchetChainError(detail) {
    return new BtrError('RATCHET_CHAIN_ERROR', detail);
}
/** NaCl secretbox open fails with BTR message key. Action: cancel transfer. */
export function ratchetDecryptFail(detail) {
    return new BtrError('RATCHET_DECRYPT_FAIL', detail);
}
/** Peer advertised BTR but sends invalid envelopes. Action: disconnect. */
export function ratchetDowngradeRejected(detail) {
    return new BtrError('RATCHET_DOWNGRADE_REJECTED', detail);
}
