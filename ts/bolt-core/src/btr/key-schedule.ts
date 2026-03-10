/**
 * Key schedule — HKDF-SHA256 derivation chain (§16.3).
 *
 * All derivations use HKDF-SHA256 with info strings from §14.
 * Output length is always 32 bytes (BTR_KEY_LENGTH).
 * Must match Rust bolt-btr/src/key_schedule.rs exactly.
 */

import { hkdf } from '@noble/hashes/hkdf.js';
import { sha256 } from '@noble/hashes/sha2.js';

import {
  BTR_KEY_LENGTH,
  BTR_SESSION_ROOT_INFO,
  BTR_TRANSFER_ROOT_INFO,
  BTR_MESSAGE_KEY_INFO,
  BTR_CHAIN_ADVANCE_INFO,
} from './constants.js';

const encoder = new TextEncoder();

/**
 * Derive session root key from ephemeral shared secret (§16.3).
 *
 * session_root_key = HKDF-SHA256(salt=empty, ikm=shared_secret, info="bolt-btr-session-root-v1", len=32)
 */
export function deriveSessionRoot(ephemeralSharedSecret: Uint8Array): Uint8Array {
  return hkdf(sha256, ephemeralSharedSecret, undefined, encoder.encode(BTR_SESSION_ROOT_INFO), BTR_KEY_LENGTH);
}

/**
 * Derive transfer root key from session root key and transfer_id (§16.3).
 *
 * transfer_root_key = HKDF-SHA256(salt=transfer_id, ikm=session_root_key, info="bolt-btr-transfer-root-v1", len=32)
 */
export function deriveTransferRoot(sessionRootKey: Uint8Array, transferId: Uint8Array): Uint8Array {
  return hkdf(sha256, sessionRootKey, transferId, encoder.encode(BTR_TRANSFER_ROOT_INFO), BTR_KEY_LENGTH);
}

/** Output of a single chain advance step. */
export interface ChainAdvanceOutput {
  /** Key for encrypting/decrypting one chunk. Single-use. */
  messageKey: Uint8Array;
  /** Replacement chain key for the next advance step. */
  nextChainKey: Uint8Array;
}

/**
 * Advance the symmetric chain: derive message_key and next_chain_key (§16.3).
 *
 * message_key = HKDF-SHA256(salt=empty, ikm=chain_key, info="bolt-btr-message-key-v1", len=32)
 * next_chain_key = HKDF-SHA256(salt=empty, ikm=chain_key, info="bolt-btr-chain-advance-v1", len=32)
 */
export function chainAdvance(chainKey: Uint8Array): ChainAdvanceOutput {
  const messageKey = hkdf(sha256, chainKey, undefined, encoder.encode(BTR_MESSAGE_KEY_INFO), BTR_KEY_LENGTH);
  const nextChainKey = hkdf(sha256, chainKey, undefined, encoder.encode(BTR_CHAIN_ADVANCE_INFO), BTR_KEY_LENGTH);
  return { messageKey, nextChainKey };
}
