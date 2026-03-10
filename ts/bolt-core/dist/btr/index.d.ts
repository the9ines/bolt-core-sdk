/**
 * Bolt Transfer Ratchet (BTR) — TypeScript parity implementation.
 *
 * Barrel re-export for all BTR modules.
 */
export { BTR_SESSION_ROOT_INFO, BTR_TRANSFER_ROOT_INFO, BTR_MESSAGE_KEY_INFO, BTR_CHAIN_ADVANCE_INFO, BTR_DH_RATCHET_INFO, BTR_KEY_LENGTH, BTR_WIRE_ERROR_CODES, } from './constants.js';
export { BtrError, ratchetStateError, ratchetChainError, ratchetDecryptFail, ratchetDowngradeRejected, } from './errors.js';
export { deriveSessionRoot, deriveTransferRoot, chainAdvance, } from './key-schedule.js';
export type { ChainAdvanceOutput } from './key-schedule.js';
export { generateRatchetKeypair, scalarMult, deriveRatchetedSessionRoot, } from './ratchet.js';
export { btrSeal, btrSealDeterministic, btrOpen } from './encrypt.js';
export { ReplayGuard } from './replay.js';
export { BtrMode, negotiateBtr, btrLogToken } from './negotiate.js';
export type { BtrModeValue } from './negotiate.js';
export { BtrEngine, BtrTransferContext } from './state.js';
