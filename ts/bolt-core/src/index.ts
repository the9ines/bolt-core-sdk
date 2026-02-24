// Constants
export {
  NONCE_LENGTH,
  PUBLIC_KEY_LENGTH,
  SECRET_KEY_LENGTH,
  DEFAULT_CHUNK_SIZE,
  PEER_CODE_LENGTH,
  PEER_CODE_ALPHABET,
  SAS_LENGTH,
  BOLT_VERSION,
} from './constants.js';

// Encoding
export { toBase64, fromBase64 } from './encoding.js';

// Crypto primitives
export { generateEphemeralKeyPair, sealBoxPayload, openBoxPayload } from './crypto.js';

// Peer codes
export {
  generateSecurePeerCode,
  generateLongPeerCode,
  isValidPeerCode,
  normalizePeerCode,
} from './peer-code.js';

// Hashing
export { sha256, bufferToHex, hashFile } from './hash.js';

// SAS
export { computeSas } from './sas.js';

// Identity
export { generateIdentityKeyPair, KeyMismatchError } from './identity.js';
export type { IdentityKeyPair } from './identity.js';

// Errors
export { BoltError, EncryptionError, ConnectionError, TransferError, IntegrityError } from './errors.js';
