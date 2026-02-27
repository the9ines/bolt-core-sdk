import { describe, it, expect } from 'vitest';
import { WIRE_ERROR_CODES, isValidWireErrorCode } from '../src/errors.js';

describe('Wire Error Code Registry (PROTOCOL.md §10)', () => {
  it('contains exactly 22 entries', () => {
    expect(WIRE_ERROR_CODES).toHaveLength(22);
  });

  it('all entries are unique', () => {
    const unique = new Set(WIRE_ERROR_CODES);
    expect(unique.size).toBe(WIRE_ERROR_CODES.length);
  });

  it('isValidWireErrorCode accepts a known code (INVALID_STATE)', () => {
    expect(isValidWireErrorCode('INVALID_STATE')).toBe(true);
  });

  it('isValidWireErrorCode accepts all 22 codes', () => {
    for (const code of WIRE_ERROR_CODES) {
      expect(isValidWireErrorCode(code)).toBe(true);
    }
  });

  it('isValidWireErrorCode rejects an unknown code', () => {
    expect(isValidWireErrorCode('NOT_A_REAL_CODE')).toBe(false);
  });

  it('isValidWireErrorCode rejects non-string values', () => {
    expect(isValidWireErrorCode(42)).toBe(false);
    expect(isValidWireErrorCode(null)).toBe(false);
    expect(isValidWireErrorCode(undefined)).toBe(false);
    expect(isValidWireErrorCode({})).toBe(false);
  });

  it('isValidWireErrorCode rejects empty string', () => {
    expect(isValidWireErrorCode('')).toBe(false);
  });
});
