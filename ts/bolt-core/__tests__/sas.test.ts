import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { computeSas } from '../src/index.js';

const __dirname = dirname(fileURLToPath(import.meta.url));
const vectorPath = join(__dirname, 'vectors', 'sas.vectors.json');
const vectors = JSON.parse(readFileSync(vectorPath, 'utf-8'));

function fromHex(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substring(i, i + 2), 16);
  }
  return bytes;
}

// Fixed 32-byte test keys (deterministic inputs for behavioral tests)
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

  // ─── Golden Vectors (from sas.vectors.json) ────────────────────────────

  describe('golden vectors', () => {
    for (const c of vectors.cases) {
      it(`${c.name} — expected SAS "${c.expected_sas}"`, async () => {
        const idA = fromHex(c.identity_a_hex);
        const idB = fromHex(c.identity_b_hex);
        const ephemeralA = fromHex(c.ephemeral_a_hex);
        const ephemeralB = fromHex(c.ephemeral_b_hex);

        const sas = await computeSas(idA, idB, ephemeralA, ephemeralB);
        expect(sas).toBe(c.expected_sas);
      });

      it(`${c.name} — symmetry (A,B == B,A)`, async () => {
        const idA = fromHex(c.identity_a_hex);
        const idB = fromHex(c.identity_b_hex);
        const ephemeralA = fromHex(c.ephemeral_a_hex);
        const ephemeralB = fromHex(c.ephemeral_b_hex);

        const forward = await computeSas(idA, idB, ephemeralA, ephemeralB);
        const reverse = await computeSas(idB, idA, ephemeralB, ephemeralA);
        expect(forward).toBe(reverse);
        expect(forward).toBe(c.expected_sas);
      });
    }
  });

  it('sensitivity — changing one byte in any input changes SAS', async () => {
    const baseline = await computeSas(keyA, keyB, ephA, ephB);

    const mutA = Uint8Array.from(keyA);
    mutA[15] = 0xFF;
    expect(await computeSas(mutA, keyB, ephA, ephB)).not.toBe(baseline);

    const mutB = Uint8Array.from(keyB);
    mutB[15] = 0xFF;
    expect(await computeSas(keyA, mutB, ephA, ephB)).not.toBe(baseline);

    const mutEA = Uint8Array.from(ephA);
    mutEA[15] = 0xFF;
    expect(await computeSas(keyA, keyB, mutEA, ephB)).not.toBe(baseline);

    const mutEB = Uint8Array.from(ephB);
    mutEB[15] = 0xFF;
    expect(await computeSas(keyA, keyB, ephA, mutEB)).not.toBe(baseline);
  });
});
