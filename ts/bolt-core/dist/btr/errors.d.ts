/**
 * BTR error types — §16.7 error behavior.
 *
 * Four error codes with deterministic behavior mapping.
 * Must match Rust bolt-btr/src/errors.rs semantics exactly.
 */
import { BTR_WIRE_ERROR_CODES } from './constants.js';
type BtrWireErrorCode = (typeof BTR_WIRE_ERROR_CODES)[number];
/** BTR-specific error — carries a wire error code and disconnect semantics. */
export declare class BtrError extends Error {
    /** Canonical wire error code from §16.7. */
    readonly wireCode: BtrWireErrorCode;
    constructor(wireCode: BtrWireErrorCode, message: string);
    /**
     * Returns true if the required action is disconnect (vs cancel transfer).
     * Matches Rust BtrError::requires_disconnect().
     */
    requiresDisconnect(): boolean;
}
/** Ratchet generation mismatch, unexpected DH key, or missing BTR fields. Action: disconnect. */
export declare function ratchetStateError(detail: string): BtrError;
/** chain_index != expected next, chain index gap, or replay. Action: cancel transfer. */
export declare function ratchetChainError(detail: string): BtrError;
/** NaCl secretbox open fails with BTR message key. Action: cancel transfer. */
export declare function ratchetDecryptFail(detail: string): BtrError;
/** Peer advertised BTR but sends invalid envelopes. Action: disconnect. */
export declare function ratchetDowngradeRejected(detail: string): BtrError;
export {};
