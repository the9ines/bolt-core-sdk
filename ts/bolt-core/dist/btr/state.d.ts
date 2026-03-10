/**
 * BTR session/transfer/chain state — §16.5 key material lifecycle.
 *
 * BtrEngine: session-level ratchet state.
 * BtrTransferContext: per-transfer chain state for seal/open.
 *
 * Must match Rust bolt-btr/src/state.rs semantics exactly.
 */
/**
 * BTR engine — manages session-level ratchet state.
 *
 * Owns the session_root_key, ratchet generation counter, and replay guard.
 * Create via `new BtrEngine(ephemeralSharedSecret)` after handshake completes.
 */
export declare class BtrEngine {
    private sessionRootKey;
    private _ratchetGeneration;
    private replayGuard;
    constructor(ephemeralSharedSecret: Uint8Array);
    /** Current ratchet generation (monotonically increasing per session). */
    get ratchetGeneration(): number;
    /** Current session root key (for testing/vector generation only). */
    getSessionRootKey(): Uint8Array;
    /**
     * Prepare to send a FILE_OFFER — generate local ratchet keypair and
     * perform DH ratchet step with remote peer's ratchet public key.
     *
     * Returns [BtrTransferContext, localRatchetPublicKey].
     */
    beginTransferSend(transferId: Uint8Array, remoteRatchetPub: Uint8Array): [BtrTransferContext, Uint8Array];
    /**
     * Accept a transfer — perform DH ratchet step with the sender's
     * ratchet public key from their FILE_OFFER envelope.
     *
     * Returns [BtrTransferContext, localRatchetPublicKey].
     */
    beginTransferReceive(transferId: Uint8Array, remoteRatchetPub: Uint8Array): [BtrTransferContext, Uint8Array];
    /** Check a received chunk's replay/ordering status. */
    checkReplay(transferId: Uint8Array, generation: number, chainIndex: number): void;
    /** End the current transfer's replay tracking. */
    endTransfer(): void;
    /** Cleanup on disconnect — zero ALL BTR state. */
    cleanupDisconnect(): void;
}
/**
 * BTR transfer context — manages per-transfer chain state.
 *
 * Created by BtrEngine.beginTransferSend/Receive.
 * Used for encrypting/decrypting chunks within a single transfer.
 */
export declare class BtrTransferContext {
    private _transferId;
    private _generation;
    private _chainKey;
    private _chainIndex;
    constructor(transferId: Uint8Array, generation: number, chainKey: Uint8Array);
    get transferId(): Uint8Array;
    get generation(): number;
    get chainIndex(): number;
    /** Current chain key (for testing only). */
    getChainKey(): Uint8Array;
    /**
     * Encrypt a chunk at the current chain position.
     *
     * Advances the chain: derives message_key and next_chain_key,
     * encrypts plaintext via NaCl secretbox.
     *
     * Returns [chainIndex, sealedBytes].
     */
    sealChunk(plaintext: Uint8Array): [number, Uint8Array];
    /**
     * Decrypt a chunk at the expected chain position.
     *
     * Same chain advance as sealChunk — both peers derive identical keys.
     */
    openChunk(expectedChainIndex: number, sealed: Uint8Array): Uint8Array;
    /** Cleanup on transfer complete (FILE_FINISH). */
    cleanupComplete(): void;
    /** Cleanup on transfer cancel (CANCEL). */
    cleanupCancel(): void;
}
