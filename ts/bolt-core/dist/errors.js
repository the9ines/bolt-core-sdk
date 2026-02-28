export class BoltError extends Error {
    constructor(message, details) {
        super(message);
        this.details = details;
        this.name = 'BoltError';
    }
}
export class EncryptionError extends BoltError {
    constructor(message, details) {
        super(message, details);
        this.name = 'EncryptionError';
    }
}
export class ConnectionError extends BoltError {
    constructor(message, details) {
        super(message, details);
        this.name = 'ConnectionError';
    }
}
export class TransferError extends BoltError {
    constructor(message, details) {
        super(message, details);
        this.name = 'TransferError';
    }
}
export class IntegrityError extends BoltError {
    constructor(message = 'File integrity check failed') {
        super(message);
        this.name = 'IntegrityError';
    }
}
// ── Wire Error Code Registry (PROTOCOL.md §10, v0.1.3-spec) ──────────
/**
 * Canonical wire error code registry — 22 codes (11 PROTOCOL + 11 ENFORCEMENT).
 * Every error frame sent on the wire MUST use a code from this array.
 * Implementations MUST reject inbound error frames carrying codes not listed here.
 */
export const WIRE_ERROR_CODES = [
    // PROTOCOL class (11)
    'VERSION_MISMATCH',
    'ENCRYPTION_FAILED',
    'INTEGRITY_FAILED',
    'REPLAY_DETECTED',
    'TRANSFER_FAILED',
    'LIMIT_EXCEEDED',
    'CONNECTION_LOST',
    'PEER_NOT_FOUND',
    'ALREADY_CONNECTED',
    'INVALID_STATE',
    'KEY_MISMATCH',
    // ENFORCEMENT class (11)
    'DUPLICATE_HELLO',
    'ENVELOPE_REQUIRED',
    'ENVELOPE_UNNEGOTIATED',
    'ENVELOPE_DECRYPT_FAIL',
    'ENVELOPE_INVALID',
    'HELLO_PARSE_ERROR',
    'HELLO_DECRYPT_FAIL',
    'HELLO_SCHEMA_ERROR',
    'INVALID_MESSAGE',
    'UNKNOWN_MESSAGE_TYPE',
    'PROTOCOL_VIOLATION',
];
/** Type guard: returns true if `x` is a canonical wire error code. */
export function isValidWireErrorCode(x) {
    return typeof x === 'string' && WIRE_ERROR_CODES.includes(x);
}
