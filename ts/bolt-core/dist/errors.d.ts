export declare class BoltError extends Error {
    details?: unknown | undefined;
    constructor(message: string, details?: unknown | undefined);
}
export declare class EncryptionError extends BoltError {
    constructor(message: string, details?: unknown);
}
export declare class ConnectionError extends BoltError {
    constructor(message: string, details?: unknown);
}
export declare class TransferError extends BoltError {
    constructor(message: string, details?: unknown);
}
export declare class IntegrityError extends BoltError {
    constructor(message?: string);
}
/**
 * Canonical wire error code registry — 22 codes (11 PROTOCOL + 11 ENFORCEMENT).
 * Every error frame sent on the wire MUST use a code from this array.
 * Implementations MUST reject inbound error frames carrying codes not listed here.
 */
export declare const WIRE_ERROR_CODES: readonly ["VERSION_MISMATCH", "ENCRYPTION_FAILED", "INTEGRITY_FAILED", "REPLAY_DETECTED", "TRANSFER_FAILED", "LIMIT_EXCEEDED", "CONNECTION_LOST", "PEER_NOT_FOUND", "ALREADY_CONNECTED", "INVALID_STATE", "KEY_MISMATCH", "DUPLICATE_HELLO", "ENVELOPE_REQUIRED", "ENVELOPE_UNNEGOTIATED", "ENVELOPE_DECRYPT_FAIL", "ENVELOPE_INVALID", "HELLO_PARSE_ERROR", "HELLO_DECRYPT_FAIL", "HELLO_SCHEMA_ERROR", "INVALID_MESSAGE", "UNKNOWN_MESSAGE_TYPE", "PROTOCOL_VIOLATION"];
/** A valid wire error code string from PROTOCOL.md §10. */
export type WireErrorCode = (typeof WIRE_ERROR_CODES)[number];
/** Type guard: returns true if `x` is a canonical wire error code. */
export declare function isValidWireErrorCode(x: unknown): x is WireErrorCode;
