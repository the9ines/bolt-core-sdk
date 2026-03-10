/**
 * Bolt Transfer Ratchet (BTR) — TypeScript parity implementation.
 *
 * Barrel re-export for all BTR modules.
 */
// Constants
export { BTR_SESSION_ROOT_INFO, BTR_TRANSFER_ROOT_INFO, BTR_MESSAGE_KEY_INFO, BTR_CHAIN_ADVANCE_INFO, BTR_DH_RATCHET_INFO, BTR_KEY_LENGTH, BTR_WIRE_ERROR_CODES, } from './constants.js';
// Errors
export { BtrError, ratchetStateError, ratchetChainError, ratchetDecryptFail, ratchetDowngradeRejected, } from './errors.js';
// Key schedule
export { deriveSessionRoot, deriveTransferRoot, chainAdvance, } from './key-schedule.js';
// DH ratchet
export { generateRatchetKeypair, scalarMult, deriveRatchetedSessionRoot, } from './ratchet.js';
// Encrypt/decrypt
export { btrSeal, btrSealDeterministic, btrOpen } from './encrypt.js';
// Replay guard
export { ReplayGuard } from './replay.js';
// Negotiate
export { BtrMode, negotiateBtr, btrLogToken } from './negotiate.js';
// State engine
export { BtrEngine, BtrTransferContext } from './state.js';
