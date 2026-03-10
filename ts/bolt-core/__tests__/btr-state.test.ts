/**
 * BTR state engine tests — BtrEngine + BtrTransferContext parity with Rust.
 */
import { describe, it, expect } from 'vitest';
import { BtrEngine, BtrTransferContext } from '../src/btr/state.js';
import { deriveSessionRoot, deriveTransferRoot, chainAdvance } from '../src/btr/key-schedule.js';
import { deriveRatchetedSessionRoot, scalarMult, generateRatchetKeypair } from '../src/btr/ratchet.js';
import { btrOpen } from '../src/btr/encrypt.js';
import { BtrError } from '../src/btr/errors.js';

function toHex(bytes: Uint8Array): string {
  return Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('');
}

describe('BtrEngine', () => {
  const sharedSecret = new Uint8Array(32).fill(0xAB);

  it('creation: generation=0, non-zero session root', () => {
    const engine = new BtrEngine(sharedSecret);
    expect(engine.ratchetGeneration).toBe(0);
    expect(toHex(engine.getSessionRootKey())).not.toBe('00'.repeat(32));
  });

  it('deterministic session root from same secret', () => {
    const a = new BtrEngine(sharedSecret);
    const b = new BtrEngine(sharedSecret);
    expect(toHex(a.getSessionRootKey())).toBe(toHex(b.getSessionRootKey()));
  });

  it('generation increments on beginTransferSend', () => {
    const engine = new BtrEngine(sharedSecret);
    const remoteKp = generateRatchetKeypair();
    const tid = new Uint8Array(16).fill(0x01);
    engine.beginTransferSend(tid, remoteKp.publicKey);
    expect(engine.ratchetGeneration).toBe(1);
  });

  it('cleanupDisconnect zeroes state', () => {
    const engine = new BtrEngine(sharedSecret);
    engine.cleanupDisconnect();
    expect(toHex(engine.getSessionRootKey())).toBe('00'.repeat(32));
    expect(engine.ratchetGeneration).toBe(0);
  });
});

describe('BtrTransferContext', () => {
  it('seal/open chunk parity: sender and receiver derive same keys', () => {
    const sharedSecret = new Uint8Array(32).fill(0xAB);
    const srk = deriveSessionRoot(sharedSecret);
    const tid = new Uint8Array(16).fill(0x42);
    const trk = deriveTransferRoot(srk, tid);

    const sender = new BtrTransferContext(new Uint8Array(tid), 1, new Uint8Array(trk));
    const receiver = new BtrTransferContext(new Uint8Array(tid), 1, new Uint8Array(trk));

    for (let i = 0; i < 3; i++) {
      const plaintext = new TextEncoder().encode(`chunk ${i}`);
      const [idx, sealed] = sender.sealChunk(plaintext);
      expect(idx).toBe(i);
      const opened = receiver.openChunk(i, sealed);
      expect(new TextDecoder().decode(opened)).toBe(`chunk ${i}`);
    }
  });

  it('open_chunk wrong index rejected with RATCHET_CHAIN_ERROR', () => {
    const trk = new Uint8Array(32).fill(0xAB);
    const ctx = new BtrTransferContext(new Uint8Array(16).fill(1), 1, trk);
    try {
      ctx.openChunk(1, new Uint8Array(64)); // expected 0, got 1
      expect.unreachable('should throw');
    } catch (err) {
      expect(err).toBeInstanceOf(BtrError);
      expect((err as BtrError).wireCode).toBe('RATCHET_CHAIN_ERROR');
    }
  });

  it('different transfers produce different keys (ISOLATION-BTR)', () => {
    const sharedSecret = new Uint8Array(32).fill(0xAB);
    const srk = deriveSessionRoot(sharedSecret);
    const trk_a = deriveTransferRoot(srk, new Uint8Array(16).fill(1));
    const trk_b = deriveTransferRoot(srk, new Uint8Array(16).fill(2));
    expect(toHex(trk_a)).not.toBe(toHex(trk_b));
  });

  it('seal_chunk advances index', () => {
    const trk = new Uint8Array(32).fill(0xAB);
    const ctx = new BtrTransferContext(new Uint8Array(16).fill(1), 1, trk);
    const [idx0] = ctx.sealChunk(new Uint8Array([0x61]));
    expect(idx0).toBe(0);
    expect(ctx.chainIndex).toBe(1);

    const [idx1] = ctx.sealChunk(new Uint8Array([0x62]));
    expect(idx1).toBe(1);
    expect(ctx.chainIndex).toBe(2);
  });

  it('cleanupComplete zeroes transfer state', () => {
    const trk = new Uint8Array(32).fill(0xAB);
    const ctx = new BtrTransferContext(new Uint8Array(16).fill(1), 1, trk);
    ctx.cleanupComplete();
    expect(toHex(ctx.getChainKey())).toBe('00'.repeat(32));
    expect(toHex(ctx.transferId)).toBe('00'.repeat(16));
  });
});

describe('cross-language chain derivation parity', () => {
  it('two engines from same secret, same DH, produce identical keys', () => {
    const sharedSecret = new Uint8Array(32).fill(0xAB);
    const engineA = new BtrEngine(sharedSecret);
    const engineB = new BtrEngine(sharedSecret);

    // Generate keypairs externally for controlled DH
    const kpA = generateRatchetKeypair();
    const kpB = generateRatchetKeypair();

    // Both compute DH
    const dhA = scalarMult(kpA.secretKey, kpB.publicKey);
    const dhB = scalarMult(kpB.secretKey, kpA.publicKey);
    expect(toHex(dhA)).toBe(toHex(dhB));

    // Both derive new session root
    const newSrkA = deriveRatchetedSessionRoot(engineA.getSessionRootKey(), dhA);
    const newSrkB = deriveRatchetedSessionRoot(engineB.getSessionRootKey(), dhB);
    expect(toHex(newSrkA)).toBe(toHex(newSrkB));

    // Both derive transfer root
    const tid = new Uint8Array(16).fill(0x01);
    const trkA = deriveTransferRoot(newSrkA, tid);
    const trkB = deriveTransferRoot(newSrkB, tid);
    expect(toHex(trkA)).toBe(toHex(trkB));

    // Chain advance produces same keys
    const advA = chainAdvance(trkA);
    const advB = chainAdvance(trkB);
    expect(toHex(advA.messageKey)).toBe(toHex(advB.messageKey));
    expect(toHex(advA.nextChainKey)).toBe(toHex(advB.nextChainKey));
  });
});
