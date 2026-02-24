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
