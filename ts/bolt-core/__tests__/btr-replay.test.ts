/**
 * BTR replay guard conformance — consumes Rust authority vectors.
 */
import { describe, it, expect } from 'vitest';
import { ReplayGuard } from '../src/btr/replay.js';
import { BtrError } from '../src/btr/errors.js';

import replayVectors from '../../../rust/bolt-core/test-vectors/btr/btr-replay-reject.vectors.json';

function fromHex(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substring(i, i + 2), 16);
  }
  return bytes;
}

describe('BTR replay guard (Rust vector parity)', () => {
  for (const v of replayVectors.vectors) {
    it(`${v.id}: ${v.description}`, () => {
      const guard = new ReplayGuard();
      const tid = fromHex(v.transfer_id_hex);

      // Set up prior accepted state
      if (v.prior_accepted.length > 0) {
        const firstPrior = v.prior_accepted[0];
        const priorTid = fromHex(firstPrior.transfer_id_hex);
        guard.beginTransfer(priorTid, firstPrior.ratchet_generation);
        for (const prior of v.prior_accepted) {
          const pTid = fromHex(prior.transfer_id_hex);
          guard.check(pTid, prior.ratchet_generation, prior.chain_index);
        }
      }

      // Begin transfer for the test vector if not already tracking
      // For reject-wrong-generation, we need a new beginTransfer
      if (v.prior_accepted.length === 0) {
        guard.beginTransfer(tid, v.ratchet_generation);
      }

      if (v.expected_reject) {
        try {
          guard.check(tid, v.ratchet_generation, v.chain_index);
          expect.unreachable('should have thrown');
        } catch (err) {
          expect(err).toBeInstanceOf(BtrError);
          const btrErr = err as BtrError;
          expect(btrErr.wireCode).toBe(v.expected_error_code);
        }
      } else {
        expect(() => {
          guard.check(tid, v.ratchet_generation, v.chain_index);
        }).not.toThrow();
      }
    });
  }

  describe('unit tests (behavioral parity with Rust)', () => {
    it('accept sequential chunks', () => {
      const guard = new ReplayGuard();
      const tid = new Uint8Array(16).fill(1);
      guard.beginTransfer(tid, 0);
      guard.check(tid, 0, 0);
      guard.check(tid, 0, 1);
      guard.check(tid, 0, 2);
    });

    it('reject skipped index', () => {
      const guard = new ReplayGuard();
      const tid = new Uint8Array(16).fill(1);
      guard.beginTransfer(tid, 0);
      expect(() => guard.check(tid, 0, 1)).toThrow(BtrError);
    });

    it('reject no active transfer', () => {
      const guard = new ReplayGuard();
      const tid = new Uint8Array(16).fill(1);
      expect(() => guard.check(tid, 0, 0)).toThrow(BtrError);
    });

    it('end_transfer clears expected', () => {
      const guard = new ReplayGuard();
      const tid = new Uint8Array(16).fill(1);
      guard.beginTransfer(tid, 0);
      guard.check(tid, 0, 0);
      guard.endTransfer();
      expect(() => guard.check(tid, 0, 1)).toThrow(BtrError);
    });

    it('reset clears all', () => {
      const guard = new ReplayGuard();
      const tid = new Uint8Array(16).fill(1);
      guard.beginTransfer(tid, 0);
      guard.check(tid, 0, 0);
      guard.reset();
      guard.beginTransfer(tid, 0);
      guard.check(tid, 0, 0); // should succeed after reset
    });

    it('cross-transfer new generation', () => {
      const guard = new ReplayGuard();
      const tid1 = new Uint8Array(16).fill(1);
      guard.beginTransfer(tid1, 0);
      guard.check(tid1, 0, 0);
      guard.endTransfer();

      const tid2 = new Uint8Array(16).fill(2);
      guard.beginTransfer(tid2, 1);
      guard.check(tid2, 1, 0);
    });
  });
});
