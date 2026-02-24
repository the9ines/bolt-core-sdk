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
