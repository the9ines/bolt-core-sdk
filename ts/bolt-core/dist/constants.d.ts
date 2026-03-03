/** NaCl box nonce length in bytes */
export declare const NONCE_LENGTH = 24;
/** X25519 public key length in bytes */
export declare const PUBLIC_KEY_LENGTH = 32;
/** X25519 secret key length in bytes */
export declare const SECRET_KEY_LENGTH = 32;
/** Default plaintext chunk size in bytes (16KB) */
export declare const DEFAULT_CHUNK_SIZE = 16384;
/** Peer code length in characters */
export declare const PEER_CODE_LENGTH = 6;
/** Unambiguous base32 alphabet for peer codes (no 0/O, 1/I/L) */
export declare const PEER_CODE_ALPHABET = "ABCDEFGHJKMNPQRSTUVWXYZ23456789";
/** SAS display length in hex characters */
export declare const SAS_LENGTH = 6;
/** Current Bolt Protocol version */
export declare const BOLT_VERSION = 1;
/** Transfer ID length in bytes (§14) */
export declare const TRANSFER_ID_LENGTH = 16;
/** SAS entropy in bits (§14) */
export declare const SAS_ENTROPY = 24;
/** File hash algorithm identifier (§14) */
export declare const FILE_HASH_ALGORITHM = "SHA-256";
/** File hash length in bytes (§14) */
export declare const FILE_HASH_LENGTH = 32;
/** Capability namespace prefix (§14) — all capability strings start with this */
export declare const CAPABILITY_NAMESPACE = "bolt.";
