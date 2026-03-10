/**
 * BTR key schedule conformance — consumes Rust authority vectors.
 */
import { describe, it, expect } from 'vitest';
import { deriveSessionRoot, deriveTransferRoot, chainAdvance } from '../src/btr/key-schedule.js';

// Rust authority vectors
import keyScheduleVectors from '../../../rust/bolt-core/test-vectors/btr/btr-key-schedule.vectors.json';
import transferRatchetVectors from '../../../rust/bolt-core/test-vectors/btr/btr-transfer-ratchet.vectors.json';
import chainAdvanceVectors from '../../../rust/bolt-core/test-vectors/btr/btr-chain-advance.vectors.json';

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

describe('BTR key schedule (Rust vector parity)', () => {
  describe('session root derivation', () => {
    for (const v of keyScheduleVectors.vectors) {
      it(`${v.id}: matches Rust output`, () => {
        const secret = fromHex(v.ephemeral_shared_secret_hex);
        const result = deriveSessionRoot(secret);
        expect(toHex(result)).toBe(v.expected_session_root_key_hex);
      });
    }
  });

  describe('transfer root derivation', () => {
    for (const v of transferRatchetVectors.vectors) {
      it(`${v.id}: matches Rust output`, () => {
        const srk = fromHex(v.session_root_key_hex);
        const tid = fromHex(v.transfer_id_hex);
        const result = deriveTransferRoot(srk, tid);
        expect(toHex(result)).toBe(v.expected_transfer_root_key_hex);
      });
    }
  });

  describe('chain advance', () => {
    for (const v of chainAdvanceVectors.vectors) {
      it(`${v.id}: matches Rust output`, () => {
        const ck = fromHex(v.chain_key_hex);
        const result = chainAdvance(ck);
        expect(toHex(result.messageKey)).toBe(v.expected_message_key_hex);
        expect(toHex(result.nextChainKey)).toBe(v.expected_next_chain_key_hex);
      });
    }

    it('5-step chained advance matches Rust vector sequence', () => {
      const vectors = chainAdvanceVectors.vectors;
      let ck = fromHex(vectors[0].chain_key_hex);
      for (const v of vectors) {
        const result = chainAdvance(ck);
        expect(toHex(result.messageKey)).toBe(v.expected_message_key_hex);
        expect(toHex(result.nextChainKey)).toBe(v.expected_next_chain_key_hex);
        ck = result.nextChainKey;
      }
    });
  });
});
