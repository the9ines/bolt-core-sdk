/**
 * BtrTransferAdapter — thin wrapper bridging BTR core primitives
 * to the transport layer's transfer send/receive operations.
 *
 * Manages BTR session state (session root key, ratchet generation)
 * and provides sender/receiver-appropriate DH ratchet step methods.
 *
 * Sender side: generates fresh ratchet keypair, DH with remote pub.
 * Receiver side: DH with ephemeral secret key (commutativity).
 */

import {
  deriveSessionRoot,
  deriveTransferRoot,
  deriveRatchetedSessionRoot,
  generateRatchetKeypair,
  scalarMult,
  BtrTransferContext,
  toBase64,
  fromBase64,
} from '@the9ines/bolt-core';

/** BTR envelope metadata for a single chunk/message. */
export interface BtrEnvelopeFields {
  ratchet_public_key?: string;  // base64 of 32 bytes (first chunk only)
  ratchet_generation?: number;  // uint32 (with ratchet_public_key)
  chain_index: number;          // uint32 (every chunk)
}

export class BtrTransferAdapter {
  private sessionRootKey: Uint8Array;
  private ratchetGeneration: number;
  private activeCtx: BtrTransferContext | null = null;
  private lastLocalRatchetPub: Uint8Array | null = null;

  constructor(ephemeralSharedSecret: Uint8Array) {
    this.sessionRootKey = deriveSessionRoot(ephemeralSharedSecret);
    this.ratchetGeneration = 0;
  }

  /** Current ratchet generation. */
  get generation(): number {
    return this.ratchetGeneration;
  }

  /** Active transfer context (null if no transfer in progress). */
  get activeTransferCtx(): BtrTransferContext | null {
    return this.activeCtx;
  }

  /**
   * Sender side: begin a new transfer with DH ratchet step.
   *
   * Generates fresh ratchet keypair, performs DH with remote peer's
   * current ratchet public key (initially their ephemeral pub).
   *
   * Returns [BtrTransferContext, localRatchetPublicKey].
   */
  beginSend(
    transferId: Uint8Array,
    remoteRatchetPub: Uint8Array,
  ): [BtrTransferContext, Uint8Array] {
    const localKp = generateRatchetKeypair();
    const dhOutput = scalarMult(localKp.secretKey, remoteRatchetPub);

    const newSrk = deriveRatchetedSessionRoot(this.sessionRootKey, dhOutput);
    // Zeroize old session root key
    this.sessionRootKey.fill(0);
    this.sessionRootKey = newSrk;
    this.ratchetGeneration += 1;

    const transferRoot = deriveTransferRoot(this.sessionRootKey, transferId);

    // Zeroize DH output and local secret key
    dhOutput.fill(0);
    localKp.secretKey.fill(0);

    this.activeCtx = new BtrTransferContext(
      new Uint8Array(transferId),
      this.ratchetGeneration,
      transferRoot,
    );
    this.lastLocalRatchetPub = localKp.publicKey;

    return [this.activeCtx, localKp.publicKey];
  }

  /**
   * Receiver side: begin receiving a transfer with DH ratchet step.
   *
   * Uses the receiver's existing secret key (ephemeral or last ratchet)
   * with the sender's ratchet public key from the envelope. This produces
   * the same DH output as the sender (DH commutativity).
   */
  beginReceive(
    transferId: Uint8Array,
    senderRatchetPub: Uint8Array,
    localSecretKey: Uint8Array,
  ): BtrTransferContext {
    const dhOutput = scalarMult(localSecretKey, senderRatchetPub);

    const newSrk = deriveRatchetedSessionRoot(this.sessionRootKey, dhOutput);
    // Zeroize old session root key
    this.sessionRootKey.fill(0);
    this.sessionRootKey = newSrk;
    this.ratchetGeneration += 1;

    const transferRoot = deriveTransferRoot(this.sessionRootKey, transferId);

    // Zeroize DH output (don't zeroize localSecretKey — caller owns it)
    dhOutput.fill(0);

    this.activeCtx = new BtrTransferContext(
      new Uint8Array(transferId),
      this.ratchetGeneration,
      transferRoot,
    );

    return this.activeCtx;
  }

  /**
   * Build BTR envelope fields for a chunk.
   *
   * First chunk (chainIndex 0) includes ratchet_public_key + ratchet_generation.
   * Subsequent chunks include only chain_index.
   */
  buildEnvelopeFields(chainIndex: number, localRatchetPub?: Uint8Array): BtrEnvelopeFields {
    if (chainIndex === 0 && localRatchetPub) {
      return {
        ratchet_public_key: toBase64(localRatchetPub),
        ratchet_generation: this.ratchetGeneration,
        chain_index: chainIndex,
      };
    }
    return { chain_index: chainIndex };
  }

  /** End the current transfer (FILE_FINISH). Zeroizes transfer-scoped state. */
  endTransfer(): void {
    this.activeCtx?.cleanupComplete();
    this.activeCtx = null;
  }

  /** Cancel the current transfer. Zeroizes transfer-scoped state. */
  cancelTransfer(): void {
    this.activeCtx?.cleanupCancel();
    this.activeCtx = null;
  }

  /** Cleanup on disconnect — zeroize ALL BTR state. */
  cleanupDisconnect(): void {
    this.sessionRootKey.fill(0);
    this.ratchetGeneration = 0;
    this.activeCtx?.cleanupCancel();
    this.activeCtx = null;
    this.lastLocalRatchetPub = null;
  }
}
