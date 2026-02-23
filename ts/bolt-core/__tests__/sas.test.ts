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

  // ─── Golden Vector ──────────────────────────────────────────────────────

  it('golden vector — fixed keys produce known SAS "65434F"', async () => {
    const sas = await computeSas(keyA, keyB, ephA, ephB);
    expect(sas).toBe('65434F');
  });

  it('cross-side equality — swapping A/B roles matches golden vector', async () => {
    const forward = await computeSas(keyA, keyB, ephA, ephB);
    const reverse = await computeSas(keyB, keyA, ephB, ephA);
    expect(forward).toBe(reverse);
    expect(forward).toBe('65434F');
  });

  it('sensitivity — changing one byte in any input changes SAS', async () => {
    const baseline = await computeSas(keyA, keyB, ephA, ephB);

    // Mutate identity key A (byte 15)
    const mutA = Uint8Array.from(keyA);
    mutA[15] = 0xFF;
    expect(await computeSas(mutA, keyB, ephA, ephB)).not.toBe(baseline);

    // Mutate identity key B (byte 15)
    const mutB = Uint8Array.from(keyB);
    mutB[15] = 0xFF;
    expect(await computeSas(keyA, mutB, ephA, ephB)).not.toBe(baseline);

    // Mutate ephemeral key A (byte 15)
    const mutEA = Uint8Array.from(ephA);
    mutEA[15] = 0xFF;
    expect(await computeSas(keyA, keyB, mutEA, ephB)).not.toBe(baseline);

    // Mutate ephemeral key B (byte 15)
    const mutEB = Uint8Array.from(ephB);
    mutEB[15] = 0xFF;
    expect(await computeSas(keyA, keyB, ephA, mutEB)).not.toBe(baseline);
  });
});
