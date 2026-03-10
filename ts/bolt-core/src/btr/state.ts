/**
 * BTR session/transfer/chain state — §16.5 key material lifecycle.
 *
 * BtrEngine: session-level ratchet state.
 * BtrTransferContext: per-transfer chain state for seal/open.
 *
 * Must match Rust bolt-btr/src/state.rs semantics exactly.
 */

import { deriveSessionRoot, deriveTransferRoot, chainAdvance } from './key-schedule.js';
import { deriveRatchetedSessionRoot, generateRatchetKeypair, scalarMult } from './ratchet.js';
import { btrSeal, btrOpen } from './encrypt.js';
import { ratchetChainError, type BtrError } from './errors.js';
import { ReplayGuard } from './replay.js';

/**
 * BTR engine — manages session-level ratchet state.
 *
 * Owns the session_root_key, ratchet generation counter, and replay guard.
 * Create via `new BtrEngine(ephemeralSharedSecret)` after handshake completes.
 */
export class BtrEngine {
  private sessionRootKey: Uint8Array;
  private _ratchetGeneration: number;
  private replayGuard: ReplayGuard;

  constructor(ephemeralSharedSecret: Uint8Array) {
    this.sessionRootKey = deriveSessionRoot(ephemeralSharedSecret);
    this._ratchetGeneration = 0;
    this.replayGuard = new ReplayGuard();
  }

  /** Current ratchet generation (monotonically increasing per session). */
  get ratchetGeneration(): number {
    return this._ratchetGeneration;
  }

  /** Current session root key (for testing/vector generation only). */
  getSessionRootKey(): Uint8Array {
    return this.sessionRootKey;
  }

  /**
   * Prepare to send a FILE_OFFER — generate local ratchet keypair and
   * perform DH ratchet step with remote peer's ratchet public key.
   *
   * Returns [BtrTransferContext, localRatchetPublicKey].
   */
  beginTransferSend(
    transferId: Uint8Array,
    remoteRatchetPub: Uint8Array,
  ): [BtrTransferContext, Uint8Array] {
    const localKp = generateRatchetKeypair();
    const localPub = localKp.publicKey;

    // DH ratchet step
    const dhOutput = scalarMult(localKp.secretKey, remoteRatchetPub);
    const newSrk = deriveRatchetedSessionRoot(this.sessionRootKey, dhOutput);

    // Update session state
    this.sessionRootKey = newSrk;
    this._ratchetGeneration += 1;

    // Derive transfer root
    const transferRoot = deriveTransferRoot(this.sessionRootKey, transferId);

    // Set up replay guard for this transfer
    this.replayGuard.beginTransfer(transferId, this._ratchetGeneration);

    const ctx = new BtrTransferContext(
      new Uint8Array(transferId),
      this._ratchetGeneration,
      transferRoot,
    );

    return [ctx, localPub];
  }

  /**
   * Accept a transfer — perform DH ratchet step with the sender's
   * ratchet public key from their FILE_OFFER envelope.
   *
   * Returns [BtrTransferContext, localRatchetPublicKey].
   */
  beginTransferReceive(
    transferId: Uint8Array,
    remoteRatchetPub: Uint8Array,
  ): [BtrTransferContext, Uint8Array] {
    return this.beginTransferSend(transferId, remoteRatchetPub);
  }

  /** Check a received chunk's replay/ordering status. */
  checkReplay(transferId: Uint8Array, generation: number, chainIndex: number): void {
    this.replayGuard.check(transferId, generation, chainIndex);
  }

  /** End the current transfer's replay tracking. */
  endTransfer(): void {
    this.replayGuard.endTransfer();
  }

  /** Cleanup on disconnect — zero ALL BTR state. */
  cleanupDisconnect(): void {
    this.sessionRootKey.fill(0);
    this._ratchetGeneration = 0;
    this.replayGuard.reset();
  }
}

/**
 * BTR transfer context — manages per-transfer chain state.
 *
 * Created by BtrEngine.beginTransferSend/Receive.
 * Used for encrypting/decrypting chunks within a single transfer.
 */
export class BtrTransferContext {
  private _transferId: Uint8Array;
  private _generation: number;
  private _chainKey: Uint8Array;
  private _chainIndex: number;

  constructor(transferId: Uint8Array, generation: number, chainKey: Uint8Array) {
    this._transferId = transferId;
    this._generation = generation;
    this._chainKey = chainKey;
    this._chainIndex = 0;
  }

  get transferId(): Uint8Array {
    return this._transferId;
  }

  get generation(): number {
    return this._generation;
  }

  get chainIndex(): number {
    return this._chainIndex;
  }

  /** Current chain key (for testing only). */
  getChainKey(): Uint8Array {
    return this._chainKey;
  }

  /**
   * Encrypt a chunk at the current chain position.
   *
   * Advances the chain: derives message_key and next_chain_key,
   * encrypts plaintext via NaCl secretbox.
   *
   * Returns [chainIndex, sealedBytes].
   */
  sealChunk(plaintext: Uint8Array): [number, Uint8Array] {
    const idx = this._chainIndex;
    const { messageKey, nextChainKey } = chainAdvance(this._chainKey);
    const sealed = btrSeal(messageKey, plaintext);
    this._chainKey = nextChainKey;
    this._chainIndex += 1;
    return [idx, sealed];
  }

  /**
   * Decrypt a chunk at the expected chain position.
   *
   * Same chain advance as sealChunk — both peers derive identical keys.
   */
  openChunk(expectedChainIndex: number, sealed: Uint8Array): Uint8Array {
    if (expectedChainIndex !== this._chainIndex) {
      throw ratchetChainError(
        `chain_index mismatch: expected ${this._chainIndex}, got ${expectedChainIndex}`,
      );
    }
    const { messageKey, nextChainKey } = chainAdvance(this._chainKey);
    const plaintext = btrOpen(messageKey, sealed);
    this._chainKey = nextChainKey;
    this._chainIndex += 1;
    return plaintext;
  }

  /** Cleanup on transfer complete (FILE_FINISH). */
  cleanupComplete(): void {
    this._chainKey.fill(0);
    this._transferId.fill(0);
  }

  /** Cleanup on transfer cancel (CANCEL). */
  cleanupCancel(): void {
    this.cleanupComplete();
  }
}
