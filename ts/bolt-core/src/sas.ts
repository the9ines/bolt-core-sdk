import { sha256, bufferToHex } from './hash.js';
import { PUBLIC_KEY_LENGTH, SAS_LENGTH } from './constants.js';

/**
 * Lexicographically sort two 32-byte values and concatenate them.
 */
function sort32(a: Uint8Array, b: Uint8Array): Uint8Array {
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) {
      const first = a[i] < b[i] ? a : b;
      const second = a[i] < b[i] ? b : a;
      const result = new Uint8Array(first.length + second.length);
      result.set(first);
      result.set(second, first.length);
      return result;
    }
  }
  // Keys are identical — concatenate as-is
  const result = new Uint8Array(a.length + b.length);
  result.set(a);
  result.set(b, a.length);
  return result;
}

/**
 * Compute a 6-character SAS (Short Authentication String) per PROTOCOL.md.
 *
 * SAS_input = SHA-256( sort32(identityA, identityB) || sort32(ephemeralA, ephemeralB) )
 * Display first 6 hex chars uppercase.
 *
 * @param identityA - Raw 32-byte identity public key of peer A
 * @param identityB - Raw 32-byte identity public key of peer B
 * @param ephemeralA - Raw 32-byte ephemeral public key of peer A
 * @param ephemeralB - Raw 32-byte ephemeral public key of peer B
 * @returns 6-character uppercase hex string (24 bits of entropy)
 */
export async function computeSas(
  identityA: Uint8Array,
  identityB: Uint8Array,
  ephemeralA: Uint8Array,
  ephemeralB: Uint8Array,
): Promise<string> {
  if (identityA.length !== PUBLIC_KEY_LENGTH || identityB.length !== PUBLIC_KEY_LENGTH) {
    throw new Error(`Identity keys must be ${PUBLIC_KEY_LENGTH} bytes`);
  }
  if (ephemeralA.length !== PUBLIC_KEY_LENGTH || ephemeralB.length !== PUBLIC_KEY_LENGTH) {
    throw new Error(`Ephemeral keys must be ${PUBLIC_KEY_LENGTH} bytes`);
  }

  const sortedIdentity = sort32(identityA, identityB);
  const sortedEphemeral = sort32(ephemeralA, ephemeralB);

  const combined = new Uint8Array(sortedIdentity.length + sortedEphemeral.length);
  combined.set(sortedIdentity);
  combined.set(sortedEphemeral, sortedIdentity.length);

  const hash = await sha256(combined);
  const hex = bufferToHex(hash);
  return hex.substring(0, SAS_LENGTH).toUpperCase();
}
