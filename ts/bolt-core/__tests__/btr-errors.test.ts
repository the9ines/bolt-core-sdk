/**
 * BTR error parity — verifies TS error classes match Rust semantics.
 */
import { describe, it, expect } from 'vitest';
import {
  BtrError,
  ratchetStateError,
  ratchetChainError,
  ratchetDecryptFail,
  ratchetDowngradeRejected,
} from '../src/btr/errors.js';
import { WIRE_ERROR_CODES, isValidWireErrorCode } from '../src/errors.js';
import { BTR_WIRE_ERROR_CODES } from '../src/btr/constants.js';

describe('BTR errors (Rust parity)', () => {
  describe('wire codes match spec', () => {
    it('RATCHET_STATE_ERROR', () => {
      const err = ratchetStateError('test');
      expect(err.wireCode).toBe('RATCHET_STATE_ERROR');
      expect(err.message).toContain('RATCHET_STATE_ERROR');
    });

    it('RATCHET_CHAIN_ERROR', () => {
      const err = ratchetChainError('test');
      expect(err.wireCode).toBe('RATCHET_CHAIN_ERROR');
    });

    it('RATCHET_DECRYPT_FAIL', () => {
      const err = ratchetDecryptFail('test');
      expect(err.wireCode).toBe('RATCHET_DECRYPT_FAIL');
    });

    it('RATCHET_DOWNGRADE_REJECTED', () => {
      const err = ratchetDowngradeRejected('test');
      expect(err.wireCode).toBe('RATCHET_DOWNGRADE_REJECTED');
    });
  });

  describe('disconnect semantics', () => {
    it('RATCHET_STATE_ERROR requires disconnect', () => {
      expect(ratchetStateError('').requiresDisconnect()).toBe(true);
    });

    it('RATCHET_CHAIN_ERROR does NOT require disconnect', () => {
      expect(ratchetChainError('').requiresDisconnect()).toBe(false);
    });

    it('RATCHET_DECRYPT_FAIL does NOT require disconnect', () => {
      expect(ratchetDecryptFail('').requiresDisconnect()).toBe(false);
    });

    it('RATCHET_DOWNGRADE_REJECTED requires disconnect', () => {
      expect(ratchetDowngradeRejected('').requiresDisconnect()).toBe(true);
    });
  });

  describe('display format includes code prefix', () => {
    it('matches Rust format: "CODE: detail"', () => {
      const err = ratchetStateError('generation mismatch');
      expect(err.message).toBe('RATCHET_STATE_ERROR: generation mismatch');
    });
  });

  describe('instanceof BtrError', () => {
    it('all factory functions produce BtrError instances', () => {
      expect(ratchetStateError('')).toBeInstanceOf(BtrError);
      expect(ratchetChainError('')).toBeInstanceOf(BtrError);
      expect(ratchetDecryptFail('')).toBeInstanceOf(BtrError);
      expect(ratchetDowngradeRejected('')).toBeInstanceOf(BtrError);
    });

    it('all factory functions produce Error instances', () => {
      expect(ratchetStateError('')).toBeInstanceOf(Error);
    });
  });
});

describe('wire error code registry parity (22 → 26)', () => {
  it('registry has 26 codes', () => {
    expect(WIRE_ERROR_CODES.length).toBe(26);
  });

  it('first 11 are PROTOCOL class', () => {
    expect(WIRE_ERROR_CODES[0]).toBe('VERSION_MISMATCH');
    expect(WIRE_ERROR_CODES[10]).toBe('KEY_MISMATCH');
  });

  it('indices 11-21 are ENFORCEMENT class', () => {
    expect(WIRE_ERROR_CODES[11]).toBe('DUPLICATE_HELLO');
    expect(WIRE_ERROR_CODES[21]).toBe('PROTOCOL_VIOLATION');
  });

  it('indices 22-25 are BTR class', () => {
    expect(WIRE_ERROR_CODES[22]).toBe('RATCHET_STATE_ERROR');
    expect(WIRE_ERROR_CODES[23]).toBe('RATCHET_CHAIN_ERROR');
    expect(WIRE_ERROR_CODES[24]).toBe('RATCHET_DECRYPT_FAIL');
    expect(WIRE_ERROR_CODES[25]).toBe('RATCHET_DOWNGRADE_REJECTED');
  });

  it('all BTR codes are valid wire error codes', () => {
    for (const code of BTR_WIRE_ERROR_CODES) {
      expect(isValidWireErrorCode(code)).toBe(true);
    }
  });

  it('all 26 codes are unique', () => {
    const unique = new Set(WIRE_ERROR_CODES);
    expect(unique.size).toBe(26);
  });

  it('BTR constants match registry slice', () => {
    const btrSlice = WIRE_ERROR_CODES.slice(22);
    expect(btrSlice).toEqual(BTR_WIRE_ERROR_CODES);
  });
});
