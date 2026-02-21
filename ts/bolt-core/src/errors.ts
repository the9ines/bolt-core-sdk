export class BoltError extends Error {
  constructor(message: string, public details?: unknown) {
    super(message);
    this.name = 'BoltError';
  }
}

export class EncryptionError extends BoltError {
  constructor(message: string, details?: unknown) {
    super(message, details);
    this.name = 'EncryptionError';
  }
}

export class ConnectionError extends BoltError {
  constructor(message: string, details?: unknown) {
    super(message, details);
    this.name = 'ConnectionError';
  }
}

export class TransferError extends BoltError {
  constructor(message: string, details?: unknown) {
    super(message, details);
    this.name = 'TransferError';
  }
}
