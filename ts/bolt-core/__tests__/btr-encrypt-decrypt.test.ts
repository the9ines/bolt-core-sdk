/**
 * BTR encrypt/decrypt conformance — deterministic fixed-nonce vectors.
 *
 * Tests both:
 * 1. TS decrypts Rust-generated ciphertext
 * 2. TS encrypts with same fixed nonce/key/plaintext → byte-identical ciphertext
 * 3. Tampered/truncated vectors produce RATCHET_DECRYPT_FAIL (same error class as Rust)
 */
import { describe, it, expect } from 'vitest';
import { btrOpen, btrSeal, btrSealDeterministic } from '../src/btr/encrypt.js';
import { BtrError } from '../src/btr/errors.js';

import encryptDecryptVectors from '../../../rust/bolt-core/test-vectors/btr/btr-encrypt-decrypt.vectors.json';

function fromHex(hex: string): Uint8Array {
  if (hex === '') return new Uint8Array(0);
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substring(i, i + 2), 16);
  }
  return bytes;
}

function toHex(bytes: Uint8Array): string {
  return Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('');
}

describe('BTR encrypt/decrypt (Rust vector parity)', () => {
  const validVectors = encryptDecryptVectors.vectors.filter(
    (v) => !('expect_error' in v && v.expect_error),
  );
  const errorVectors = encryptDecryptVectors.vectors.filter(
    (v) => 'expect_error' in v && v.expect_error,
  );

  describe('valid vectors — decrypt Rust ciphertext', () => {
    for (const v of validVectors) {
      it(`${v.id}: TS decrypts Rust ciphertext correctly`, () => {
        const key = fromHex(v.message_key_hex);
        const sealed = fromHex(v.expected_ciphertext_hex);
        const plaintext = btrOpen(key, sealed);
        expect(toHex(plaintext)).toBe(v.plaintext_hex);
      });
    }
  });

  describe('valid vectors — TS encrypt matches Rust byte-for-byte', () => {
    for (const v of validVectors) {
      it(`${v.id}: TS seal with fixed nonce produces identical ciphertext`, () => {
        const key = fromHex(v.message_key_hex);
        const nonce = fromHex(v.nonce_hex);
        const plaintext = fromHex(v.plaintext_hex);
        const sealed = btrSealDeterministic(key, plaintext, nonce);
        expect(toHex(sealed)).toBe(v.expected_ciphertext_hex);
      });
    }
  });

  describe('error vectors — tampered/malformed must reject with RATCHET_DECRYPT_FAIL', () => {
    for (const v of errorVectors) {
      it(`${v.id}: throws BtrError with wireCode=${v.expect_error}`, () => {
        const key = fromHex(v.message_key_hex);
        const sealed = fromHex(v.expected_ciphertext_hex);
        try {
          btrOpen(key, sealed);
          expect.unreachable('should have thrown');
        } catch (err) {
          expect(err).toBeInstanceOf(BtrError);
          const btrErr = err as BtrError;
          expect(btrErr.wireCode).toBe(v.expect_error);
          expect(btrErr.requiresDisconnect()).toBe(false);
        }
      });
    }
  });

  describe('round-trip: seal then open', () => {
    it('random nonce round-trip works', () => {
      const key = fromHex('e0e1e2e3e4e5e6e7e8e9eaebecedeeeff0f1f2f3f4f5f6f7f8f9fafbfcfdfeff');
      const plaintext = new TextEncoder().encode('round-trip test');
      const sealed = btrSeal(key, plaintext);
      const opened = btrOpen(key, sealed);
      expect(new TextDecoder().decode(opened)).toBe('round-trip test');
    });
  });
});
