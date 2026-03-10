/**
 * BTR adversarial vectors (P3).
 *
 * Consumes Rust-authority btr-adversarial.vectors.json:
 * - Wrong-key decrypt: valid ciphertext, wrong message key → RATCHET_DECRYPT_FAIL
 * - Chain-index desync: receiver at idx=0, open called with idx=2 → RATCHET_CHAIN_ERROR
 */
import { describe, it, expect } from 'vitest';
import adversarialVectors from '../../../rust/bolt-core/test-vectors/btr/btr-adversarial.vectors.json';
import { btrOpen } from '../src/btr/encrypt.js';
import { BtrError } from '../src/btr/errors.js';
import { BtrTransferContext } from '../src/btr/state.js';
import { deriveTransferRoot } from '../src/btr/key-schedule.js';

function fromHex(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substring(i, i + 2), 16);
  }
  return bytes;
}

describe('BTR adversarial vectors (Rust authority)', () => {
  const wrongKeyVector = adversarialVectors.vectors.find(
    (v) => v.id === 'wrong-key-decrypt',
  )!;
  const chainDesyncVector = adversarialVectors.vectors.find(
    (v) => v.id === 'chain-index-desync',
  )!;

  describe('wrong-key decrypt', () => {
    it('open with correct key succeeds', () => {
      const correctKey = fromHex(wrongKeyVector.correct_key_hex);
      const sealed = fromHex(wrongKeyVector.sealed_hex);
      const plaintext = btrOpen(correctKey, sealed);
      expect(Array.from(plaintext)).toEqual(
        Array.from(fromHex(wrongKeyVector.plaintext_hex)),
      );
    });

    it('open with wrong key throws RATCHET_DECRYPT_FAIL', () => {
      const wrongKey = fromHex(wrongKeyVector.wrong_key_hex);
      const sealed = fromHex(wrongKeyVector.sealed_hex);
      try {
        btrOpen(wrongKey, sealed);
        expect.unreachable('should throw');
      } catch (err) {
        expect(err).toBeInstanceOf(BtrError);
        expect((err as BtrError).wireCode).toBe(wrongKeyVector.expected_error);
      }
    });

    it('error class matches Rust: RATCHET_DECRYPT_FAIL', () => {
      expect(wrongKeyVector.expected_error).toBe('RATCHET_DECRYPT_FAIL');
    });
  });

  describe('chain-index desync', () => {
    it('transfer root derivation matches vector', () => {
      const srk = fromHex(chainDesyncVector.session_root_key_hex);
      const tid = fromHex(chainDesyncVector.transfer_id_hex);
      const trk = deriveTransferRoot(srk, tid);
      expect(Array.from(trk)).toEqual(
        Array.from(fromHex(chainDesyncVector.transfer_root_key_hex)),
      );
    });

    it('open at wrong chain_index throws RATCHET_CHAIN_ERROR', () => {
      const trk = fromHex(chainDesyncVector.transfer_root_key_hex);
      const tid = fromHex(chainDesyncVector.transfer_id_hex);
      const ctx = new BtrTransferContext(new Uint8Array(tid), 1, new Uint8Array(trk));

      // ctx is at chain_index=0, vector says receiver_open_at_index=2
      const wrongIndex = chainDesyncVector.receiver_open_at_index;
      try {
        ctx.openChunk(wrongIndex, new Uint8Array(64));
        expect.unreachable('should throw');
      } catch (err) {
        expect(err).toBeInstanceOf(BtrError);
        expect((err as BtrError).wireCode).toBe(chainDesyncVector.expected_error);
      }
    });

    it('error class matches Rust: RATCHET_CHAIN_ERROR', () => {
      expect(chainDesyncVector.expected_error).toBe('RATCHET_CHAIN_ERROR');
    });
  });
});
