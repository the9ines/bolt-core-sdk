/**
 * BTR negotiate conformance — consumes Rust authority vectors.
 */
import { describe, it, expect } from 'vitest';
import { negotiateBtr, btrLogToken } from '../src/btr/negotiate.js';

import negotiateVectors from '../../../rust/bolt-core/test-vectors/btr/btr-downgrade-negotiate.vectors.json';

describe('BTR negotiate (Rust vector parity)', () => {
  for (const v of negotiateVectors.vectors) {
    it(`${v.id}: ${v.description}`, () => {
      const mode = negotiateBtr(
        v.local_supports_btr,
        v.remote_supports_btr,
        v.remote_well_formed,
      );
      expect(mode).toBe(v.expected_mode);

      const token = btrLogToken(mode);
      expect(token).toBe(v.expected_log_token ?? null);
    });
  }

  it('all 6 matrix cells covered', () => {
    expect(negotiateVectors.vectors.length).toBe(6);
  });
});
