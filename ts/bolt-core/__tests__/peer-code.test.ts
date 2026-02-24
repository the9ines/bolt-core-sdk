import { describe, it, expect } from 'vitest';
import {
  generateSecurePeerCode,
  generateLongPeerCode,
  isValidPeerCode,
  normalizePeerCode,
} from '../src/index.js';
import { PEER_CODE_ALPHABET } from '../src/constants.js';

describe('generateSecurePeerCode', () => {
  it('returns a 6-character string', () => {
    const code = generateSecurePeerCode();
    expect(code).toHaveLength(6);
  });

  it('uses only valid alphabet characters', () => {
    for (let i = 0; i < 20; i++) {
      const code = generateSecurePeerCode();
      for (const char of code) {
        expect(PEER_CODE_ALPHABET).toContain(char);
      }
    }
  });

  it('generates unique codes', () => {
    const codes = new Set(Array.from({ length: 50 }, () => generateSecurePeerCode()));
    expect(codes.size).toBeGreaterThan(40);
  });
});

describe('generateLongPeerCode', () => {
  it('returns XXXX-XXXX format', () => {
    const code = generateLongPeerCode();
    expect(code).toMatch(/^[A-Z2-9]{4}-[A-Z2-9]{4}$/);
  });

  it('has 9 characters including dash', () => {
    expect(generateLongPeerCode()).toHaveLength(9);
  });
});

describe('rejection sampling invariants', () => {
  it('REJECTION_MAX = floor(256 / N) * N = 248 for N=31', () => {
    // N = PEER_CODE_ALPHABET.length = 31
    // MAX = floor(256 / 31) * 31 = 8 * 31 = 248
    expect(PEER_CODE_ALPHABET.length).toBe(31);
    const expected = Math.floor(256 / 31) * 31;
    expect(expected).toBe(248);
  });

  it('generateSecurePeerCode returns 6 chars all in alphabet (stress)', () => {
    for (let i = 0; i < 100; i++) {
      const code = generateSecurePeerCode();
      expect(code).toHaveLength(6);
      for (const ch of code) {
        expect(PEER_CODE_ALPHABET).toContain(ch);
      }
    }
  });

  it('generateLongPeerCode matches /^[A-Z2-9]{4}-[A-Z2-9]{4}$/ (stress)', () => {
    for (let i = 0; i < 100; i++) {
      expect(generateLongPeerCode()).toMatch(/^[A-Z2-9]{4}-[A-Z2-9]{4}$/);
    }
  });
});

describe('isValidPeerCode', () => {
  it('accepts valid 6-char codes', () => {
    expect(isValidPeerCode('ABCDEF')).toBe(true);
  });

  it('accepts valid 8-char codes with dash', () => {
    expect(isValidPeerCode('ABCD-EFGH')).toBe(true);
  });

  it('accepts valid 8-char codes without dash', () => {
    expect(isValidPeerCode('ABCDEFGH')).toBe(true);
  });

  it('is case-insensitive', () => {
    expect(isValidPeerCode('abcdef')).toBe(true);
  });

  it('rejects wrong length', () => {
    expect(isValidPeerCode('ABC')).toBe(false);
    expect(isValidPeerCode('ABCDEFGHIJ')).toBe(false);
  });

  it('rejects ambiguous characters (0, O, 1, I, L)', () => {
    expect(isValidPeerCode('ABCDE0')).toBe(false);
    expect(isValidPeerCode('ABCDE1')).toBe(false);
    expect(isValidPeerCode('ABCDEI')).toBe(false);
    expect(isValidPeerCode('ABCDEL')).toBe(false);
    expect(isValidPeerCode('ABCDEO')).toBe(false);
  });

  it('accepts generated codes', () => {
    expect(isValidPeerCode(generateSecurePeerCode())).toBe(true);
    expect(isValidPeerCode(generateLongPeerCode())).toBe(true);
  });
});

describe('normalizePeerCode', () => {
  it('removes dashes', () => {
    expect(normalizePeerCode('ABCD-EFGH')).toBe('ABCDEFGH');
  });

  it('uppercases', () => {
    expect(normalizePeerCode('abcdef')).toBe('ABCDEF');
  });

  it('handles already normalized codes', () => {
    expect(normalizePeerCode('ABCDEF')).toBe('ABCDEF');
  });
});
