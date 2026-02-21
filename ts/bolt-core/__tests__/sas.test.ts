import { describe, it, expect } from 'vitest';
import { computeSas } from '../src/index.js';

// Fixed 32-byte test keys (deterministic inputs for reproducible tests)
const keyA = new Uint8Array(32);
keyA[0] = 0x01;
keyA[31] = 0xAA;

const keyB = new Uint8Array(32);
keyB[0] = 0x02;
keyB[31] = 0xBB;

const ephA = new Uint8Array(32);
ephA[0] = 0x10;
ephA[31] = 0xCC;

const ephB = new Uint8Array(32);
ephB[0] = 0x20;
ephB[31] = 0xDD;

describe('computeSas', () => {
  it('returns a 6-character uppercase hex string', async () => {
    const sas = await computeSas(keyA, keyB, ephA, ephB);
    expect(sas).toHaveLength(6);
    expect(sas).toMatch(/^[0-9A-F]{6}$/);
  });

  it('is deterministic for same inputs', async () => {
    const sas1 = await computeSas(keyA, keyB, ephA, ephB);
    const sas2 = await computeSas(keyA, keyB, ephA, ephB);
    expect(sas1).toBe(sas2);
  });

  it('is symmetric (A,B == B,A)', async () => {
    const sas1 = await computeSas(keyA, keyB, ephA, ephB);
    const sas2 = await computeSas(keyB, keyA, ephB, ephA);
    expect(sas1).toBe(sas2);
  });

  it('produces different SAS for different keys', async () => {
    const sas1 = await computeSas(keyA, keyB, ephA, ephB);

    const differentKey = new Uint8Array(32);
    differentKey[0] = 0xFF;
    const sas2 = await computeSas(differentKey, keyB, ephA, ephB);

    expect(sas1).not.toBe(sas2);
  });

  it('rejects keys with wrong length', async () => {
    const shortKey = new Uint8Array(16);
    await expect(computeSas(shortKey, keyB, ephA, ephB)).rejects.toThrow('32 bytes');
    await expect(computeSas(keyA, keyB, shortKey, ephB)).rejects.toThrow('32 bytes');
  });
});
