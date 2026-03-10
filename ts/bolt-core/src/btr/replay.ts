/**
 * Replay rejection — (transfer_id, generation, chain_index) guard (§11).
 *
 * ORDER-BTR: chain_index must equal expected_next_index (no gaps).
 * REPLAY-BTR: (transfer_id, ratchet_generation, chain_index) triple
 *   prevents cross-generation replay.
 *
 * Must match Rust bolt-btr/src/replay.rs semantics exactly.
 */

import { ratchetStateError, ratchetChainError } from './errors.js';

function tripleKey(tid: Uint8Array, generation: number, chainIndex: number): string {
  // Encode triple as a string key for Set lookup
  const hexTid = Array.from(tid, (b) => b.toString(16).padStart(2, '0')).join('');
  return `${hexTid}:${generation}:${chainIndex}`;
}

interface ExpectedState {
  transferId: Uint8Array;
  generation: number;
  nextIndex: number;
}

/**
 * Replay guard tracking seen (transfer_id, generation, chain_index) triples.
 *
 * Enforces ORDER-BTR: chain_index must be strictly monotonic per transfer
 * (no skipped-key buffer). Also rejects cross-generation replay.
 */
export class ReplayGuard {
  private seen = new Set<string>();
  private expected: ExpectedState | null = null;

  /** Begin tracking a new transfer at the given generation. Resets expected chain_index to 0. */
  beginTransfer(transferId: Uint8Array, generation: number): void {
    this.expected = {
      transferId: new Uint8Array(transferId),
      generation,
      nextIndex: 0,
    };
  }

  /**
   * Check and record a (transfer_id, generation, chain_index) triple.
   * Throws BtrError on violation.
   */
  check(transferId: Uint8Array, generation: number, chainIndex: number): void {
    if (this.expected === null) {
      throw ratchetStateError('no active transfer in replay guard');
    }

    // Check transfer_id matches
    if (!uint8ArrayEqual(transferId, this.expected.transferId)) {
      throw ratchetStateError('transfer_id mismatch');
    }

    // Check generation matches
    if (generation !== this.expected.generation) {
      throw ratchetStateError(
        `generation mismatch: expected ${this.expected.generation}, got ${generation}`,
      );
    }

    // ORDER-BTR: chain_index must equal expected next (no gaps)
    if (chainIndex !== this.expected.nextIndex) {
      throw ratchetChainError(
        `chain_index out of order: expected ${this.expected.nextIndex}, got ${chainIndex}`,
      );
    }

    // REPLAY-BTR: check for duplicate triple
    const key = tripleKey(transferId, generation, chainIndex);
    if (this.seen.has(key)) {
      throw ratchetChainError(
        `replay detected: generation=${generation}, chain_index=${chainIndex}`,
      );
    }

    this.seen.add(key);
    this.expected.nextIndex = chainIndex + 1;
  }

  /** End tracking for the current transfer. Retains seen set for cross-transfer replay detection. */
  endTransfer(): void {
    this.expected = null;
  }

  /** Full reset — clears all state. Used on disconnect. */
  reset(): void {
    this.seen.clear();
    this.expected = null;
  }
}

function uint8ArrayEqual(a: Uint8Array, b: Uint8Array): boolean {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) return false;
  }
  return true;
}
