/**
 * Replay rejection — (transfer_id, generation, chain_index) guard (§11).
 *
 * ORDER-BTR: chain_index must equal expected_next_index (no gaps).
 * REPLAY-BTR: (transfer_id, ratchet_generation, chain_index) triple
 *   prevents cross-generation replay.
 *
 * Must match Rust bolt-btr/src/replay.rs semantics exactly.
 */
/**
 * Replay guard tracking seen (transfer_id, generation, chain_index) triples.
 *
 * Enforces ORDER-BTR: chain_index must be strictly monotonic per transfer
 * (no skipped-key buffer). Also rejects cross-generation replay.
 */
export declare class ReplayGuard {
    private seen;
    private expected;
    /** Begin tracking a new transfer at the given generation. Resets expected chain_index to 0. */
    beginTransfer(transferId: Uint8Array, generation: number): void;
    /**
     * Check and record a (transfer_id, generation, chain_index) triple.
     * Throws BtrError on violation.
     */
    check(transferId: Uint8Array, generation: number, chainIndex: number): void;
    /** End tracking for the current transfer. Retains seen set for cross-transfer replay detection. */
    endTransfer(): void;
    /** Full reset — clears all state. Used on disconnect. */
    reset(): void;
}
