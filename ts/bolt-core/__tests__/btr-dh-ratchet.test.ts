/**
 * BTR DH ratchet + DH sanity conformance — consumes Rust authority vectors.
 */
import { describe, it, expect } from 'vitest';
import { deriveRatchetedSessionRoot, scalarMult } from '../src/btr/ratchet.js';

import dhRatchetVectors from '../../../rust/bolt-core/test-vectors/btr/btr-dh-ratchet.vectors.json';
import dhSanityVectors from '../../../rust/bolt-core/test-vectors/btr/btr-dh-sanity.vectors.json';

function fromHex(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substring(i, i + 2), 16);
  }
  return bytes;
}

function toHex(bytes: Uint8Array): string {
  return Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('');
}

describe('BTR DH ratchet step (Rust vector parity)', () => {
  for (const v of dhRatchetVectors.vectors) {
    it(`${v.id}: matches Rust output`, () => {
      const srk = fromHex(v.current_session_root_key_hex);
      const dh = fromHex(v.dh_output_hex);
      const result = deriveRatchetedSessionRoot(srk, dh);
      expect(toHex(result)).toBe(v.expected_new_session_root_key_hex);
    });
  }
});

describe('X25519 DH sanity (tweetnacl vs x25519-dalek)', () => {
  for (const v of dhSanityVectors.vectors) {
    it(`${v.id}: ${v.description}`, () => {
      const scalar = fromHex(v.secret_scalar_hex);
      const remotePub = fromHex(v.remote_public_hex);
      const result = scalarMult(scalar, remotePub);
      expect(toHex(result)).toBe(v.expected_shared_secret_hex);
    });
  }

  it('DH commutativity: a0×b0 == b0×a0', () => {
    const v0 = dhSanityVectors.vectors.find((v) => v.id === 'dh-a0-b0')!;
    const v1 = dhSanityVectors.vectors.find((v) => v.id === 'dh-b0-a0')!;
    expect(v0.expected_shared_secret_hex).toBe(v1.expected_shared_secret_hex);
  });

  it('different pairs yield different outputs', () => {
    const v0 = dhSanityVectors.vectors.find((v) => v.id === 'dh-a0-b0')!;
    const v2 = dhSanityVectors.vectors.find((v) => v.id === 'dh-c0-a0')!;
    expect(v0.expected_shared_secret_hex).not.toBe(v2.expected_shared_secret_hex);
  });
});
