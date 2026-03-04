/**
 * TransferManager — owns file send/receive, chunk processing,
 * pause/resume/cancel, progress, stats, backpressure, and metrics.
 *
 * Extracted from WebRTCService (A2). All behavior preserved exactly.
 *
 * State ownership: Like HandshakeManager, state lives on WebRTCService.
 * TransferManager reads/writes through a TransferContext bridge so that
 * tests (which set fields via `(service as any).fieldName`) continue to work.
 */
import { sealBoxPayload, openBoxPayload, DEFAULT_CHUNK_SIZE, EncryptionError, IntegrityError, hashFile, toBase64, bufferToHex } from '@the9ines/bolt-core';
import { WebRTCError, TransferError } from '../../types/webrtc-errors.js';
import { ENABLE_TRANSFER_METRICS, TransferMetricsCollector, summarizeTransfer } from './transferMetrics.js';
import { dcSendMessage } from './EnvelopeCodec.js';
import type { FileChunkMessage, ActiveTransfer, TransferProgress } from './types.js';

/** Generate a spec-compliant transfer_id (bytes16 → hex, 32 chars). */
function generateTransferId(): string {
  const bytes = new Uint8Array(16);
  crypto.getRandomValues(bytes);
  return bufferToHex(bytes.buffer);
}

/**
 * TransferContext — the subset of shared state that TransferManager needs.
 */
export interface TransferContext {
  // Crypto keys
  getKeyPair(): { publicKey: Uint8Array; secretKey: Uint8Array } | null;
  getRemotePublicKey(): Uint8Array | null;

  // DataChannel
  getDc(): RTCDataChannel | null;

  // Handshake state (read-only from transfer perspective)
  isHelloComplete(): boolean;
  getSessionGeneration(): number;
  hasCapability(name: string): boolean;
  negotiatedEnvelopeV1(): boolean;

  // Transfer state (read/write)
  getTransferPaused(): boolean;
  setTransferPaused(v: boolean): void;
  getTransferCancelled(): boolean;
  setTransferCancelled(v: boolean): void;
  getReceiveBuffers(): Map<string, (Blob | null)[]>;
  getGuardedTransfers(): Map<string, ActiveTransfer>;
  getSendTransferIds(): Map<string, string>;
  getRecvTransferIds(): Map<string, string>;

  // Stats
  getTransferStartTime(): number;
  setTransferStartTime(v: number): void;
  getPauseDuration(): number;
  setPauseDuration(v: number): void;
  getLastPausedAt(): number | null;
  setLastPausedAt(v: number | null): void;

  // Metrics
  getMetricsCollector(): TransferMetricsCollector | null;
  setMetricsCollector(v: TransferMetricsCollector | null): void;
  getMetricsFirstProgressRecorded(): boolean;
  setMetricsFirstProgressRecorded(v: boolean): void;

  // Backpressure
  getBackpressureReject(): ((err: Error) => void) | undefined;
  setBackpressureReject(v: ((err: Error) => void) | undefined): void;

  // Completion timer
  getCompletionTimeout(): ReturnType<typeof setTimeout> | null;
  setCompletionTimeout(v: ReturnType<typeof setTimeout> | null): void;

  // Progress callback
  getOnProgressCallback(): ((progress: TransferProgress) => void) | undefined;

  // Identity key (for guarded transfers)
  getRemoteIdentityKey(): Uint8Array | null;

  // Callbacks
  onReceiveFile(file: Blob, filename: string): void;
  onError(error: Error): void;
  disconnect(): void;

  // Envelope send helper
  sendMessage(innerMsg: any): void;

  // HELLO wait (delegates to service's waitForHello)
  waitForHello(): Promise<void>;
}

/** DP-9: Max time (ms) to wait for onbufferedamountlow before polling fallback. */
const BACKPRESSURE_TIMEOUT_MS = 5000;

export class TransferManager {
  // DP-9: Guard against concurrent sendFile calls (property overwrite on onbufferedamountlow)
  private sendInProgress = false;

  constructor(private ctx: TransferContext) {}

  // ─── File Transfer (Send) ──────────────────────────────────────────

  async sendFile(file: File): Promise<void> {
    // DP-9: Prevent concurrent sendFile calls — property-based onbufferedamountlow
    // only supports one handler at a time; concurrent sends overwrite each other.
    if (this.sendInProgress) {
      throw new TransferError('Transfer already in progress');
    }
    this.sendInProgress = true;

    // Wait for HELLO handshake before allowing file transfer
    if (!this.ctx.isHelloComplete()) {
      await this.ctx.waitForHello();
    }

    const dc = this.ctx.getDc();
    if (!dc || dc.readyState !== 'open') {
      this.sendInProgress = false;
      throw new TransferError('Data channel not open');
    }
    const remotePublicKey = this.ctx.getRemotePublicKey();
    if (!remotePublicKey) {
      throw new EncryptionError('No remote public key');
    }

    const totalChunks = Math.ceil(file.size / DEFAULT_CHUNK_SIZE);
    const transferId = generateTransferId();
    this.ctx.getSendTransferIds().set(file.name, transferId);

    // Compute file hash if bolt.file-hash was negotiated
    let fileHash: string | undefined;
    if (this.ctx.hasCapability('bolt.file-hash')) {
      fileHash = await hashFile(file);
    }

    console.log(`[TRANSFER] Sending ${file.name} (${file.size} bytes, ${totalChunks} chunks, tid=${transferId})`);
    this.ctx.setTransferCancelled(false);
    this.ctx.setTransferPaused(false);
    this.ctx.setTransferStartTime(Date.now());
    this.ctx.setPauseDuration(0);
    this.ctx.setLastPausedAt(null);

    if (ENABLE_TRANSFER_METRICS) {
      const collector = new TransferMetricsCollector();
      this.ctx.setMetricsCollector(collector);
      this.ctx.setMetricsFirstProgressRecorded(false);
      collector.begin(transferId, file.size, DEFAULT_CHUNK_SIZE, totalChunks);
    }

    try {
      for (let i = 0; i < totalChunks; i++) {
        if (this.ctx.getTransferCancelled()) throw new TransferError('Transfer cancelled by user');

        while (this.ctx.getTransferPaused()) {
          await new Promise(r => setTimeout(r, 100));
          if (this.ctx.getTransferCancelled()) throw new TransferError('Transfer cancelled while paused');
        }

        const start = i * DEFAULT_CHUNK_SIZE;
        const end = Math.min(start + DEFAULT_CHUNK_SIZE, file.size);
        const raw = new Uint8Array(await file.slice(start, end).arrayBuffer());

        const keyPair = this.ctx.getKeyPair();
        if (!keyPair) throw new EncryptionError('No ephemeral key pair');
        const encrypted = sealBoxPayload(raw, remotePublicKey, keyPair.secretKey);

        const msg: FileChunkMessage = {
          type: 'file-chunk',
          filename: file.name,
          chunk: encrypted,
          chunkIndex: i,
          totalChunks,
          fileSize: file.size,
          transferId,
          ...(fileHash && i === 0 ? { fileHash } : {}),
        };

        // Backpressure — wait for buffer to drain (N1: cancelable by disconnect)
        // DP-9: Added timeout fallback — if onbufferedamountlow never fires
        // (e.g. threshold misconfiguration, browser quirk on received DC),
        // fall back to polling after BACKPRESSURE_TIMEOUT_MS.
        const currentDc = this.ctx.getDc();
        if (currentDc && currentDc.bufferedAmount > currentDc.bufferedAmountLowThreshold) {
          const gen = this.ctx.getSessionGeneration();
          this.ctx.getMetricsCollector()?.enterBufferDrainWait();
          await new Promise<void>((resolve, reject) => {
            this.ctx.setBackpressureReject(reject);
            let settled = false;
            const settle = () => {
              if (settled) return;
              settled = true;
              this.ctx.setBackpressureReject(undefined);
              this.ctx.getMetricsCollector()?.exitBufferDrainWait();
              const dcNow = this.ctx.getDc();
              if (dcNow) dcNow.onbufferedamountlow = null;
              if (gen !== this.ctx.getSessionGeneration()) {
                reject(new TransferError('Transfer aborted: session ended'));
                return;
              }
              resolve();
            };
            currentDc.onbufferedamountlow = settle;
            // DP-9: Timeout fallback — poll bufferedAmount if event never fires
            setTimeout(() => {
              if (settled) return;
              const dcCheck = this.ctx.getDc();
              if (!dcCheck || dcCheck.readyState !== 'open') {
                settled = true;
                this.ctx.setBackpressureReject(undefined);
                reject(new TransferError('Data channel closed during backpressure wait'));
                return;
              }
              // Buffer may have drained by now; resolve regardless to unblock transfer
              console.warn('[TRANSFER] Backpressure timeout — forcing drain resolve (bufferedAmount=' + dcCheck.bufferedAmount + ')');
              settle();
            }, BACKPRESSURE_TIMEOUT_MS);
          });
        }

        if (this.ctx.getTransferCancelled()) throw new TransferError('Transfer cancelled by user');

        this.ctx.getMetricsCollector()?.recordChunkSend(this.ctx.getDc()!.bufferedAmount, i + 1);
        this.ctx.sendMessage(msg);
        this.emitProgress(file.name, i + 1, totalChunks, end, file.size, 'transferring');
      }

      console.log(`[TRANSFER] All chunks sent for ${file.name}`);
      this.ctx.getSendTransferIds().delete(file.name);

      const collector = this.ctx.getMetricsCollector();
      if (collector) {
        const metrics = collector.finish();
        this.ctx.setMetricsCollector(null);
        this.ctx.setMetricsFirstProgressRecorded(false);
        if (metrics) console.log('[TRANSFER_METRICS]', JSON.stringify(summarizeTransfer(metrics)));
      }

      // Emit completion after a brief delay so UI can process final progress
      this.ctx.setCompletionTimeout(setTimeout(() => {
        this.emitProgress(file.name, totalChunks, totalChunks, file.size, file.size, 'completed');
      }, 50));
      this.sendInProgress = false;
    } catch (error) {
      this.sendInProgress = false;
      this.ctx.getSendTransferIds().delete(file.name);

      const collector = this.ctx.getMetricsCollector();
      if (collector) {
        const metrics = collector.finish();
        this.ctx.setMetricsCollector(null);
        this.ctx.setMetricsFirstProgressRecorded(false);
        if (metrics) console.log('[TRANSFER_METRICS]', JSON.stringify(summarizeTransfer(metrics)));
      }

      if (!(error instanceof TransferError && error.message.includes('cancelled'))) {
        this.emitProgress(file.name, 0, totalChunks, 0, file.size, 'error');
      }
      throw error;
    }
  }

  // ─── File Transfer (Receive) ──────────────────────────────────────

  /** Route a decoded inner message (from envelope or legacy plaintext). */
  routeInnerMessage(msg: any): void {
    if (msg.type !== 'file-chunk' || !msg.filename) return;

    // Control messages
    if (msg.paused) {
      this.ctx.setTransferPaused(true);
      this.emitProgress(msg.filename, 0, 0, 0, 0, 'paused');
      return;
    }
    if (msg.resumed) {
      this.ctx.setTransferPaused(false);
      this.emitProgress(msg.filename, 0, 0, 0, 0, 'transferring');
      return;
    }
    if (msg.cancelled) {
      this.handleRemoteCancel(msg);
      return;
    }

    // Data chunk
    this.processChunk(msg);
  }

  private handleRemoteCancel(msg: FileChunkMessage): void {
    const status = msg.cancelledBy === 'receiver' ? 'canceled_by_receiver' : 'canceled_by_sender';
    this.ctx.getReceiveBuffers().delete(msg.filename);
    if (msg.transferId) {
      this.ctx.getGuardedTransfers().delete(msg.transferId);
    }
    // Also clean up by filename lookup
    const recvTid = this.ctx.getRecvTransferIds().get(msg.filename);
    if (recvTid) {
      this.ctx.getGuardedTransfers().delete(recvTid);
      this.ctx.getRecvTransferIds().delete(msg.filename);
    }
    this.ctx.setTransferCancelled(true);
    this.emitProgress(msg.filename, 0, 0, 0, 0, status);
  }

  private isValidChunkFields(msg: FileChunkMessage): boolean {
    const { chunkIndex, totalChunks } = msg;
    if (!Number.isFinite(totalChunks) || !Number.isInteger(totalChunks!) || totalChunks! <= 0) {
      console.warn(`[REPLAY_OOB] invalid totalChunks=${totalChunks} — rejected`);
      return false;
    }
    if (!Number.isFinite(chunkIndex) || !Number.isInteger(chunkIndex!) || chunkIndex! < 0 || chunkIndex! >= totalChunks!) {
      console.warn(`[REPLAY_OOB] chunkIndex=${chunkIndex} out of range [0, ${totalChunks}) — rejected`);
      return false;
    }
    return true;
  }

  processChunk(msg: FileChunkMessage): void {
    if (!msg.chunk || typeof msg.chunkIndex !== 'number' || !msg.totalChunks || !msg.fileSize) return;
    if (!this.ctx.getRemotePublicKey()) return;

    // Bounds check applies to BOTH modes
    if (!this.isValidChunkFields(msg)) return;

    if (msg.transferId && this.ctx.isHelloComplete()) {
      this.processChunkGuarded(msg).catch((error) => {
        console.error(`[TRANSFER] Unhandled error in guarded path:`, error);
        this.ctx.onError(error instanceof WebRTCError ? error : new TransferError('Chunk processing failed', error));
      });
    } else {
      if (msg.transferId && !this.ctx.isHelloComplete()) {
        console.warn('[REPLAY_UNGUARDED] transferId present but HELLO incomplete — falling back to legacy');
      } else {
        console.warn('[REPLAY_UNGUARDED] chunk received without transferId — legacy peer');
      }
      this.processChunkLegacy(msg);
    }
  }

  private async processChunkGuarded(msg: FileChunkMessage): Promise<void> {
    const { filename, chunk, chunkIndex, totalChunks, fileSize, transferId } = msg;
    const remoteIdKey = this.ctx.getRemoteIdentityKey();
    const identityKey = remoteIdKey ? toBase64(remoteIdKey) : '';

    // Lookup or create guarded transfer
    const guardedTransfers = this.ctx.getGuardedTransfers();
    let transfer = guardedTransfers.get(transferId!);
    if (!transfer) {
      console.log(`[TRANSFER] Receiving ${filename} (${fileSize} bytes, ${totalChunks} chunks, tid=${transferId})`);
      // Store expectedHash from first chunk if bolt.file-hash negotiated
      const expectedHash = (this.ctx.hasCapability('bolt.file-hash') && msg.fileHash) ? msg.fileHash : undefined;
      transfer = {
        transferId: transferId!,
        filename: filename,
        totalChunks: totalChunks!,
        fileSize: fileSize!,
        buffer: new Array(totalChunks!).fill(null),
        receivedSet: new Set(),
        remoteIdentityKey: identityKey,
        expectedHash,
      };
      guardedTransfers.set(transferId!, transfer);
      this.ctx.getRecvTransferIds().set(filename, transferId!);
      this.ctx.setTransferStartTime(Date.now());
      this.ctx.setPauseDuration(0);
    } else if (transfer.remoteIdentityKey !== identityKey) {
      // Same transferId but different sender identity — cross-peer collision
      console.warn(`[REPLAY_XFER_MISMATCH] transferId=${transferId} bound to different sender identity — ignored`);
      return;
    }

    // Dedup check
    if (transfer.receivedSet.has(chunkIndex!)) {
      console.warn(`[REPLAY_DUP] chunkIndex=${chunkIndex} already received for tid=${transferId} — ignored`);
      return;
    }

    try {
      const keyPair = this.ctx.getKeyPair();
      if (!keyPair) throw new EncryptionError('No ephemeral key pair');
      const decrypted = openBoxPayload(chunk!, this.ctx.getRemotePublicKey()!, keyPair.secretKey);
      transfer.buffer[chunkIndex!] = new Blob([decrypted as BlobPart]);
      transfer.receivedSet.add(chunkIndex!);

      const received = transfer.receivedSet.size;
      this.emitProgress(filename, received, totalChunks!, received * (fileSize! / totalChunks!), fileSize!, 'transferring');

      // Check completion
      if (received === totalChunks!) {
        const assembledBlob = new Blob(transfer.buffer as Blob[]);

        // Verify file integrity if expectedHash was provided (bolt.file-hash negotiated)
        if (transfer.expectedHash) {
          try {
            const actual = await hashFile(assembledBlob);
            if (actual !== transfer.expectedHash) {
              console.error(`[INTEGRITY_MISMATCH] expected=${transfer.expectedHash} actual=${actual} (tid=${transferId})`);
              guardedTransfers.delete(transferId!);
              this.ctx.getRecvTransferIds().delete(filename);
              this.emitProgress(filename, 0, totalChunks!, 0, fileSize!, 'error');
              this.ctx.onError(new IntegrityError('File integrity check failed: hash mismatch'));
              this.ctx.sendMessage({
                type: 'error',
                code: 'INTEGRITY_FAILED',
                message: 'File integrity check failed: hash mismatch',
              });
              this.ctx.disconnect();
              return;
            }
            console.log(`[INTEGRITY_OK] hash verified for ${filename} (tid=${transferId})`);
          } catch (hashError) {
            console.error(`[INTEGRITY_ERROR] failed to compute hash for ${filename}:`, hashError);
            // Fail-closed: treat hash computation failure as integrity failure
            guardedTransfers.delete(transferId!);
            this.ctx.getRecvTransferIds().delete(filename);
            this.emitProgress(filename, 0, totalChunks!, 0, fileSize!, 'error');
            this.ctx.onError(new IntegrityError('File integrity check failed: hash computation error'));
            this.ctx.disconnect();
            return;
          }
        }

        console.log(`[TRANSFER] Completed receiving ${filename} (tid=${transferId})`);
        guardedTransfers.delete(transferId!);
        this.ctx.getRecvTransferIds().delete(filename);
        this.emitProgress(filename, totalChunks!, totalChunks!, fileSize!, fileSize!, 'completed');
        this.ctx.onReceiveFile(assembledBlob, filename);
      }
    } catch (error) {
      console.error(`[TRANSFER] Error processing chunk ${chunkIndex} of ${filename} (tid=${transferId}):`, error);
      guardedTransfers.delete(transferId!);
      this.ctx.getRecvTransferIds().delete(filename);
      this.emitProgress(filename, 0, totalChunks!, 0, fileSize!, 'error');
      this.ctx.onError(error instanceof WebRTCError ? error : new TransferError('Chunk processing failed', error));
    }
  }

  private processChunkLegacy(msg: FileChunkMessage): void {
    const { filename, chunk, chunkIndex, totalChunks, fileSize } = msg;

    const receiveBuffers = this.ctx.getReceiveBuffers();
    // Initialize buffer on first chunk
    if (!receiveBuffers.has(filename)) {
      console.log(`[TRANSFER] Receiving ${filename} (${fileSize} bytes, ${totalChunks} chunks) [legacy]`);
      receiveBuffers.set(filename, new Array(totalChunks!).fill(null));
      this.ctx.setTransferStartTime(Date.now());
      this.ctx.setPauseDuration(0);
    }

    try {
      const keyPair = this.ctx.getKeyPair();
      if (!keyPair) throw new EncryptionError('No ephemeral key pair');
      const decrypted = openBoxPayload(chunk!, this.ctx.getRemotePublicKey()!, keyPair.secretKey);
      const buffer = receiveBuffers.get(filename)!;
      buffer[chunkIndex!] = new Blob([decrypted as BlobPart]);

      const received = buffer.filter(Boolean).length;
      this.emitProgress(filename, received, totalChunks!, received * (fileSize! / totalChunks!), fileSize!, 'transferring');

      // Check completion
      if (received === totalChunks!) {
        console.log(`[TRANSFER] Completed receiving ${filename} [legacy]`);
        const file = new Blob(buffer as Blob[]);
        receiveBuffers.delete(filename);
        this.emitProgress(filename, totalChunks!, totalChunks!, fileSize!, fileSize!, 'completed');
        this.ctx.onReceiveFile(file, filename);
      }
    } catch (error) {
      console.error(`[TRANSFER] Error processing chunk ${chunkIndex} of ${filename}:`, error);
      receiveBuffers.delete(filename);
      this.emitProgress(filename, 0, totalChunks!, 0, fileSize!, 'error');
      this.ctx.onError(error instanceof WebRTCError ? error : new TransferError('Chunk processing failed', error));
    }
  }

  // ─── Transfer Control ──────────────────────────────────────────────

  pauseTransfer(filename: string): void {
    this.ctx.setTransferPaused(true);
    this.ctx.setLastPausedAt(Date.now());
    this.ctx.getMetricsCollector()?.markPaused();
    const transferId = this.ctx.getSendTransferIds().get(filename);
    this.sendControlMessage(filename, { paused: true, ...(transferId && { transferId }) });
    this.emitProgress(filename, 0, 0, 0, 0, 'paused');
  }

  resumeTransfer(filename: string): void {
    const lastPausedAt = this.ctx.getLastPausedAt();
    if (lastPausedAt) {
      this.ctx.setPauseDuration(this.ctx.getPauseDuration() + Date.now() - lastPausedAt);
      this.ctx.setLastPausedAt(null);
    }
    this.ctx.setTransferPaused(false);
    this.ctx.getMetricsCollector()?.markResumed();
    const transferId = this.ctx.getSendTransferIds().get(filename);
    this.sendControlMessage(filename, { resumed: true, ...(transferId && { transferId }) });
    this.emitProgress(filename, 0, 0, 0, 0, 'transferring');
  }

  cancelTransfer(filename: string, isReceiver: boolean = false): void {
    this.ctx.setTransferCancelled(true);
    const transferId = isReceiver
      ? this.ctx.getRecvTransferIds().get(filename)
      : this.ctx.getSendTransferIds().get(filename);
    this.sendControlMessage(filename, {
      cancelled: true,
      cancelledBy: isReceiver ? 'receiver' : 'sender',
      ...(transferId && { transferId }),
    });
    this.ctx.getReceiveBuffers().delete(filename);
    if (transferId) {
      this.ctx.getGuardedTransfers().delete(transferId);
      this.ctx.getRecvTransferIds().delete(filename);
      this.ctx.getSendTransferIds().delete(filename);
    }
    const status = isReceiver ? 'canceled_by_receiver' : 'canceled_by_sender';
    this.emitProgress(filename, 0, 0, 0, 0, status);
  }

  private sendControlMessage(filename: string, fields: Partial<FileChunkMessage>): void {
    if (!this.ctx.getDc() || this.ctx.getDc()!.readyState !== 'open') return;
    this.ctx.sendMessage({ type: 'file-chunk', filename, ...fields });
  }

  // ─── Progress ──────────────────────────────────────────────────────

  emitProgress(
    filename: string,
    currentChunk: number,
    totalChunks: number,
    loaded: number,
    total: number,
    status: TransferProgress['status']
  ): void {
    const collector = this.ctx.getMetricsCollector();
    if (collector && !this.ctx.getMetricsFirstProgressRecorded()) {
      collector.recordFirstProgress();
      this.ctx.setMetricsFirstProgressRecorded(true);
    }

    const callback = this.ctx.getOnProgressCallback();
    if (!callback) return;

    const elapsed = Math.max(1, Date.now() - this.ctx.getTransferStartTime() - this.ctx.getPauseDuration());
    const speed = loaded > 0 ? loaded / (elapsed / 1000) : 0;
    const remaining = speed > 0 ? (total - loaded) / speed : 0;

    callback({
      filename,
      currentChunk,
      totalChunks,
      loaded,
      total,
      status,
      stats: {
        speed,
        averageSpeed: speed,
        estimatedTimeRemaining: remaining,
        retryCount: 0,
        maxRetries: 0,
        startTime: this.ctx.getTransferStartTime(),
        pauseDuration: this.ctx.getPauseDuration(),
        lastPausedAt: this.ctx.getLastPausedAt() ?? undefined,
      },
    });
  }

}
