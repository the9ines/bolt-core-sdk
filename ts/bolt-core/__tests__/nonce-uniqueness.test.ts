import { describe, it, expect } from 'vitest';
import {
  sealBoxPayload,
  generateEphemeralKeyPair,
  fromBase64,
} from '../src/index.js';
import { NONCE_LENGTH } from '../src/constants.js';

/**
 * Nonce uniqueness sanity test (H6).
 *
 * Verifies that the production sealBoxPayload path produces unique,
 * well-formed nonces. This is an in-process statistical sanity check,
 * NOT a cryptographic guarantee of cross-process uniqueness.
 *
 * Wire format: base64(nonce || ciphertext), nonce is first 24 bytes.
 */
describe('nonce uniqueness', () => {
  const N = 128;
  const alice = generateEphemeralKeyPair();
  const bob = generateEphemeralKeyPair();
  const plaintext = new Uint8Array([1, 2, 3, 4]);

  const nonces: Uint8Array[] = [];

  // Seal N times and extract nonces
  for (let i = 0; i < N; i++) {
    const sealed = sealBoxPayload(plaintext, bob.publicKey, alice.secretKey);
    const raw = fromBase64(sealed);
    nonces.push(raw.slice(0, NONCE_LENGTH));
  }

  it(`all ${N} nonces are exactly ${NONCE_LENGTH} bytes`, () => {
    for (const nonce of nonces) {
      expect(nonce.length).toBe(NONCE_LENGTH);
    }
  });

  it(`no nonce is all-zero`, () => {
    const zero = new Uint8Array(NONCE_LENGTH);
    for (const nonce of nonces) {
      expect(nonce).not.toEqual(zero);
    }
  });

  it(`all ${N} nonces are unique`, () => {
    const seen = new Set<string>();
    for (const nonce of nonces) {
      const hex = Array.from(nonce, (b) => b.toString(16).padStart(2, '0')).join('');
      expect(seen.has(hex)).toBe(false);
      seen.add(hex);
    }
    expect(seen.size).toBe(N);
  });
});
