// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { SignalingProvider } from '../services/signaling/SignalingProvider.js';

// ─── Stub @the9ines/bolt-core to isolate replay/bounds logic from crypto ─────

class MockBoltError extends Error {
  details?: unknown;
  constructor(message: string, details?: unknown) {
    super(message);
    this.name = 'BoltError';
    this.details = details;
  }
}

vi.mock('@the9ines/bolt-core', () => ({
  sealBoxPayload: (_data: Uint8Array) => 'encrypted-stub',
  openBoxPayload: (cipher: string) => {
    // Return deterministic plaintext from the cipher stub
    return new Uint8Array([...new TextEncoder().encode(cipher)]);
  },
  generateEphemeralKeyPair: () => ({
    publicKey: new Uint8Array(32),
    secretKey: new Uint8Array(32),
  }),
  generateIdentityKeyPair: () => ({
    publicKey: new Uint8Array(32),
    secretKey: new Uint8Array(64),
  }),
  toBase64: (arr: Uint8Array) => {
    return Array.from(arr).map(b => b.toString(16).padStart(2, '0')).join('');
  },
  fromBase64: () => new Uint8Array(32),
  DEFAULT_CHUNK_SIZE: 16384,
  BoltError: MockBoltError,
  EncryptionError: class extends MockBoltError {
    constructor(m: string, d?: unknown) { super(m, d); this.name = 'EncryptionError'; }
  },
  ConnectionError: class extends MockBoltError {
    constructor(m: string, d?: unknown) { super(m, d); this.name = 'ConnectionError'; }
  },
  TransferError: class extends MockBoltError {
    constructor(m: string, d?: unknown) { super(m, d); this.name = 'TransferError'; }
  },
  IntegrityError: class extends MockBoltError {
    constructor(m: string = 'File integrity check failed') { super(m); this.name = 'IntegrityError'; }
  },
  KeyMismatchError: class extends MockBoltError {
    constructor(m: string, d?: unknown) { super(m, d); this.name = 'KeyMismatchError'; }
  },
  computeSas: () => 'AABBCC',
  bufferToHex: (buffer: ArrayBuffer) =>
    Array.from(new Uint8Array(buffer)).map(b => b.toString(16).padStart(2, '0')).join(''),
  hashFile: async () => 'a'.repeat(64),
}));

// ─── Helpers ─────────────────────────────────────────────────────────────────

function createMockSignaling(): SignalingProvider {
  return {
    connect: vi.fn().mockResolvedValue(undefined),
    onSignal: vi.fn(),
    onPeerDiscovered: vi.fn(),
    onPeerLost: vi.fn(),
    sendSignal: vi.fn().mockResolvedValue(undefined),
    getPeers: vi.fn().mockReturnValue([]),
    disconnect: vi.fn(),
    name: 'mock',
  };
}

function makeChunkMsg(overrides: Record<string, unknown> = {}) {
  return {
    type: 'file-chunk',
    filename: 'test.bin',
    chunk: `chunk-data-${overrides.chunkIndex ?? 0}`,
    chunkIndex: 0,
    totalChunks: 3,
    fileSize: 48,
    ...overrides,
  };
}

const IDENTITY_A = '0000000000000000000000000000000000000000000000000000000000000000';
const IDENTITY_B = '1111111111111111111111111111111111111111111111111111111111111111';

// ─── Tests ───────────────────────────────────────────────────────────────────

describe('Phase 8A: Replay Protection + Chunk Bounds', () => {
  let WebRTCService: any;

  beforeEach(async () => {
    const mod = await import('../services/webrtc/WebRTCService.js');
    WebRTCService = mod.default;
  });

  function createService(onReceiveFile = vi.fn(), onError = vi.fn()) {
    const signaling = createMockSignaling();
    const service = new WebRTCService(signaling, 'LOCAL', onReceiveFile, onError);
    // Set up dummy crypto state so processChunk doesn't bail
    (service as any).remotePublicKey = new Uint8Array(32);
    (service as any).keyPair = {
      publicKey: new Uint8Array(32),
      secretKey: new Uint8Array(32),
    };
    return service;
  }

  // ─── Guarded mode (transferId present) ───────────────────────────────────

  describe('Guarded mode (transferId present)', () => {
    it('rejects duplicate chunkIndex with [REPLAY_DUP] warning', () => {
      const service = createService();
      // Enable guarded mode: set helloComplete + remoteIdentityKey
      (service as any).helloComplete = true;
      (service as any).remoteIdentityKey = new Uint8Array(32);

      const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

      const msg1 = makeChunkMsg({ chunkIndex: 0, transferId: 'aaa' });
      const msg2 = makeChunkMsg({ chunkIndex: 0, transferId: 'aaa', chunk: 'different-data' });

      (service as any).processChunk(msg1);
      (service as any).processChunk(msg2);

      // Should have logged [REPLAY_DUP]
      const dupWarnings = warnSpy.mock.calls.filter(
        (args) => typeof args[0] === 'string' && args[0].includes('[REPLAY_DUP]')
      );
      expect(dupWarnings.length).toBe(1);

      // Buffer should only have the first chunk's data
      const transfer = (service as any).guardedTransfers.get('aaa');
      expect(transfer.receivedSet.size).toBe(1);

      warnSpy.mockRestore();
      service.disconnect();
    });

    it('rejects out-of-range chunkIndex with [REPLAY_OOB] warning', () => {
      const service = createService();
      (service as any).helloComplete = true;
      (service as any).remoteIdentityKey = new Uint8Array(32);

      const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

      // chunkIndex >= totalChunks
      (service as any).processChunk(makeChunkMsg({ chunkIndex: 5, totalChunks: 3, transferId: 'bbb' }));
      // chunkIndex < 0
      (service as any).processChunk(makeChunkMsg({ chunkIndex: -1, totalChunks: 3, transferId: 'bbb' }));

      const oobWarnings = warnSpy.mock.calls.filter(
        (args) => typeof args[0] === 'string' && args[0].includes('[REPLAY_OOB]')
      );
      expect(oobWarnings.length).toBe(2);

      // No guarded transfer should have been created
      expect((service as any).guardedTransfers.size).toBe(0);

      warnSpy.mockRestore();
      service.disconnect();
    });

    it('rejects same transferId from different sender identity with [REPLAY_XFER_MISMATCH]', () => {
      const service = createService();
      (service as any).helloComplete = true;
      // Start with identity A
      (service as any).remoteIdentityKey = new Uint8Array(32).fill(0);

      const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

      // First chunk from identity A
      (service as any).processChunk(makeChunkMsg({ chunkIndex: 0, transferId: 'ccc' }));
      expect((service as any).guardedTransfers.has('ccc')).toBe(true);

      // Switch identity to B (simulating different peer)
      (service as any).remoteIdentityKey = new Uint8Array(32).fill(1);

      // Same transferId from identity B — should be rejected
      (service as any).processChunk(makeChunkMsg({ chunkIndex: 1, transferId: 'ccc' }));

      const mismatchWarnings = warnSpy.mock.calls.filter(
        (args) => typeof args[0] === 'string' && args[0].includes('[REPLAY_XFER_MISMATCH]')
      );
      expect(mismatchWarnings.length).toBe(1);

      // Transfer should still have only 1 chunk from identity A
      const transfer = (service as any).guardedTransfers.get('ccc');
      expect(transfer.receivedSet.size).toBe(1);

      warnSpy.mockRestore();
      service.disconnect();
    });

    it('creates new transfer for different transferId (no mismatch log)', () => {
      const service = createService();
      (service as any).helloComplete = true;
      (service as any).remoteIdentityKey = new Uint8Array(32);

      const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

      // Two different transferIds from same identity
      (service as any).processChunk(makeChunkMsg({ chunkIndex: 0, transferId: 'ddd' }));
      (service as any).processChunk(makeChunkMsg({ chunkIndex: 0, transferId: 'eee', filename: 'other.bin' }));

      // Both should exist as separate guarded transfers
      expect((service as any).guardedTransfers.size).toBe(2);
      expect((service as any).guardedTransfers.has('ddd')).toBe(true);
      expect((service as any).guardedTransfers.has('eee')).toBe(true);

      // No mismatch warning
      const mismatchWarnings = warnSpy.mock.calls.filter(
        (args) => typeof args[0] === 'string' && args[0].includes('[REPLAY_XFER_MISMATCH]')
      );
      expect(mismatchWarnings.length).toBe(0);

      warnSpy.mockRestore();
      service.disconnect();
    });

    it('reconstructs file correctly from out-of-order delivery', () => {
      const onReceiveFile = vi.fn();
      const service = createService(onReceiveFile);
      (service as any).helloComplete = true;
      (service as any).remoteIdentityKey = new Uint8Array(32);

      // Send chunks out of order: 2, 0, 1
      (service as any).processChunk(makeChunkMsg({ chunkIndex: 2, totalChunks: 3, transferId: 'fff', chunk: 'c2' }));
      (service as any).processChunk(makeChunkMsg({ chunkIndex: 0, totalChunks: 3, transferId: 'fff', chunk: 'c0' }));
      (service as any).processChunk(makeChunkMsg({ chunkIndex: 1, totalChunks: 3, transferId: 'fff', chunk: 'c1' }));

      // onReceiveFile should have been called
      expect(onReceiveFile).toHaveBeenCalledTimes(1);
      const [blob, filename] = onReceiveFile.mock.calls[0];
      expect(blob).toBeInstanceOf(Blob);
      expect(filename).toBe('test.bin');

      service.disconnect();
    });

    it('resets state after completion and accepts new transferId', () => {
      const onReceiveFile = vi.fn();
      const service = createService(onReceiveFile);
      (service as any).helloComplete = true;
      (service as any).remoteIdentityKey = new Uint8Array(32);

      // Complete a 1-chunk transfer
      (service as any).processChunk(makeChunkMsg({ chunkIndex: 0, totalChunks: 1, transferId: 'ggg' }));
      expect(onReceiveFile).toHaveBeenCalledTimes(1);
      expect((service as any).guardedTransfers.size).toBe(0);

      // Start a new transfer with different transferId
      (service as any).processChunk(makeChunkMsg({ chunkIndex: 0, totalChunks: 2, transferId: 'hhh' }));
      expect((service as any).guardedTransfers.has('hhh')).toBe(true);
      expect((service as any).guardedTransfers.get('hhh').receivedSet.size).toBe(1);

      service.disconnect();
    });
  });

  // ─── Legacy mode (no transferId) ─────────────────────────────────────────

  describe('Legacy mode (no transferId)', () => {
    it('accepts chunks without transferId and completes transfer', () => {
      const onReceiveFile = vi.fn();
      const service = createService(onReceiveFile);

      // No transferId — legacy path
      (service as any).processChunk(makeChunkMsg({ chunkIndex: 0, totalChunks: 2 }));
      (service as any).processChunk(makeChunkMsg({ chunkIndex: 1, totalChunks: 2 }));

      expect(onReceiveFile).toHaveBeenCalledTimes(1);

      // guardedTransfers should be untouched
      expect((service as any).guardedTransfers.size).toBe(0);

      service.disconnect();
    });

    it('emits [REPLAY_UNGUARDED] deprecation warning per chunk', () => {
      const service = createService();
      const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

      (service as any).processChunk(makeChunkMsg({ chunkIndex: 0, totalChunks: 2 }));
      (service as any).processChunk(makeChunkMsg({ chunkIndex: 1, totalChunks: 2 }));

      const unguardedWarnings = warnSpy.mock.calls.filter(
        (args) => typeof args[0] === 'string' && args[0].includes('[REPLAY_UNGUARDED]')
      );
      expect(unguardedWarnings.length).toBe(2);

      warnSpy.mockRestore();
      service.disconnect();
    });
  });

  // ─── Sender ──────────────────────────────────────────────────────────────

  describe('Sender', () => {
    it('sendFile includes transferId (32 hex chars, constant across all chunks)', async () => {
      const service = createService();
      (service as any).helloComplete = true;
      (service as any).remoteIdentityKey = new Uint8Array(32);

      // Create a mock data channel
      const sentMessages: string[] = [];
      const mockDc = {
        readyState: 'open',
        bufferedAmount: 0,
        bufferedAmountLowThreshold: 0,
        send: (data: string) => sentMessages.push(data),
        onbufferedamountlow: null,
      };
      (service as any).dc = mockDc;

      // Create a mock File with working slice().arrayBuffer()
      const fileData = new Uint8Array(100);
      const file = {
        name: 'sender-test.bin',
        size: 100,
        slice: (start: number, end: number) => ({
          arrayBuffer: () => Promise.resolve(fileData.slice(start, end).buffer),
        }),
      } as unknown as File;

      await service.sendFile(file);

      // Parse sent messages
      const chunks = sentMessages.map((m) => JSON.parse(m));
      expect(chunks.length).toBeGreaterThan(0);

      // All chunks should have the same transferId
      const transferIds = chunks.map((c: any) => c.transferId);
      const uniqueIds = new Set(transferIds);
      expect(uniqueIds.size).toBe(1);

      // transferId should be 32 hex chars
      const tid = transferIds[0];
      expect(tid).toMatch(/^[0-9a-f]{32}$/);

      service.disconnect();
    });
  });
});
