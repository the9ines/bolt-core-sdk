// Constants
export { DEFAULT_CHUNK_SIZE, } from './constants.js';
// Encoding
export { toBase64, fromBase64 } from './encoding.js';
// Crypto primitives
export { generateEphemeralKeyPair, sealBoxPayload, openBoxPayload } from './crypto.js';
// Peer codes
export { generateSecurePeerCode, generateLongPeerCode, isValidPeerCode, normalizePeerCode, } from './peer-code.js';
// Hashing
export { sha256, bufferToHex, hashFile } from './hash.js';
// SAS
export { computeSas } from './sas.js';
// Identity
export { generateIdentityKeyPair, KeyMismatchError } from './identity.js';
// Errors
export { BoltError, EncryptionError, ConnectionError, TransferError, IntegrityError } from './errors.js';
// Wire error code registry (PROTOCOL.md §10)
export { WIRE_ERROR_CODES, isValidWireErrorCode } from './errors.js';
