// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { SignalingProvider } from '../services/signaling/SignalingProvider.js';

// ─── Stub @the9ines/bolt-core ────────────────────────────────────────────────

class MockBoltError extends Error {
  details?: unknown;
  constructor(message: string, details?: unknown) {
    super(message);
    this.name = 'BoltError';
    this.details = details;
  }
}

class MockIntegrityError extends MockBoltError {
  constructor(m: string = 'File integrity check failed') {
    super(m);
    this.name = 'IntegrityError';
  }
}

/**
 * Deterministic mock hashFile: returns SHA-256-length hex derived from blob content.
 * For tests, we produce a predictable hash so we can control match/mismatch.
 * The mock uses a simple scheme: hash of blob text content zero-padded to 64 chars.
 */
const MOCK_HASH_MATCH = 'a'.repeat(64);
const MOCK_HASH_MISMATCH = 'b'.repeat(64);

let mockHashFileResult = MOCK_HASH_MATCH;

vi.mock('@the9ines/bolt-core', () => ({
  sealBoxPayload: (_data: Uint8Array) => 'encrypted-stub',
  openBoxPayload: (cipher: string) =>
    new Uint8Array([...new TextEncoder().encode(cipher)]),
  generateEphemeralKeyPair: () => ({
    publicKey: new Uint8Array(32),
    secretKey: new Uint8Array(32),
  }),
  generateIdentityKeyPair: () => ({
    publicKey: new Uint8Array(32),
    secretKey: new Uint8Array(64),
  }),
  toBase64: (arr: Uint8Array) =>
    Array.from(arr).map(b => b.toString(16).padStart(2, '0')).join(''),
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
  IntegrityError: MockIntegrityError,
  KeyMismatchError: class extends MockBoltError {
    constructor(m: string, d?: unknown) { super(m, d); this.name = 'KeyMismatchError'; }
  },
  computeSas: () => 'AABBCC',
  bufferToHex: (buffer: ArrayBuffer) =>
    Array.from(new Uint8Array(buffer)).map(b => b.toString(16).padStart(2, '0')).join(''),
  hashFile: async () => mockHashFileResult,
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

// ─── Tests ───────────────────────────────────────────────────────────────────

describe('Phase M2: File Hash Wiring', () => {
  let WebRTCService: any;

  beforeEach(async () => {
    mockHashFileResult = MOCK_HASH_MATCH;
    const mod = await import('../services/webrtc/WebRTCService.js');
    WebRTCService = mod.default;
  });

  function createService(
    onReceiveFile = vi.fn(),
    onError = vi.fn(),
    onProgress?: (p: any) => void,
    options?: any,
  ) {
    const signaling = createMockSignaling();
    const service = new WebRTCService(
      signaling, 'LOCAL', onReceiveFile, onError, onProgress,
      options ?? { identityPublicKey: new Uint8Array(32) },
    );
    (service as any).remotePublicKey = new Uint8Array(32);
    (service as any).keyPair = {
      publicKey: new Uint8Array(32),
      secretKey: new Uint8Array(32),
    };
    return service;
  }

  function attachDataChannel(service: any) {
    const sentMessages: string[] = [];
    const dc = {
      readyState: 'open',
      bufferedAmount: 0,
      bufferedAmountLowThreshold: 0,
      binaryType: 'arraybuffer',
      send: (data: string) => sentMessages.push(data),
      close: vi.fn(),
      onmessage: null as ((event: MessageEvent) => void) | null,
      onopen: null as (() => void) | null,
      onclose: null as (() => void) | null,
      onerror: null as ((e: Event) => void) | null,
      onbufferedamountlow: null as (() => void) | null,
    };
    (service as any).dc = dc;
    dc.onmessage = (event: MessageEvent) => {
      (service as any).handleMessage(event);
    };
    return { dc, sentMessages };
  }

  /** Simulate completed HELLO with both peers supporting bolt.file-hash. */
  function enableFileHash(service: any) {
    (service as any).helloComplete = true;
    (service as any).remoteIdentityKey = new Uint8Array(32);
    (service as any).localCapabilities = ['bolt.file-hash'];
    (service as any).negotiatedCapabilities = ['bolt.file-hash'];
  }

  /** Simulate completed HELLO with no file-hash capability negotiated. */
  function disableFileHash(service: any) {
    (service as any).helloComplete = true;
    (service as any).remoteIdentityKey = new Uint8Array(32);
    (service as any).localCapabilities = ['bolt.file-hash'];
    (service as any).negotiatedCapabilities = [];
  }

  // ──────────────────────────────────────────────────────────────────────────
  // Negotiation tests
  // ──────────────────────────────────────────────────────────────────────────

  describe('Negotiation', () => {
    it('both peers advertise bolt.file-hash → verification active', async () => {
      const onReceiveFile = vi.fn();
      const service = createService(onReceiveFile);
      enableFileHash(service);

      // Send 2 chunks with fileHash on first
      (service as any).processChunk(makeChunkMsg({
        chunkIndex: 0, totalChunks: 2, transferId: 'n1', fileHash: MOCK_HASH_MATCH,
      }));
      (service as any).processChunk(makeChunkMsg({
        chunkIndex: 1, totalChunks: 2, transferId: 'n1',
      }));

      // Wait for async hash verification
      await new Promise(r => setTimeout(r, 20));

      expect(onReceiveFile).toHaveBeenCalledTimes(1);
      service.disconnect();
    });

    it('only local advertises → verification inactive (no expectedHash stored)', async () => {
      const onReceiveFile = vi.fn();
      const service = createService(onReceiveFile);
      // local has bolt.file-hash but negotiated is empty (remote doesn't support it)
      disableFileHash(service);

      (service as any).processChunk(makeChunkMsg({
        chunkIndex: 0, totalChunks: 1, transferId: 'n2', fileHash: MOCK_HASH_MATCH,
      }));

      await new Promise(r => setTimeout(r, 20));

      // Should complete without verification
      expect(onReceiveFile).toHaveBeenCalledTimes(1);
      service.disconnect();
    });

    it('only remote advertises → verification inactive', async () => {
      const onReceiveFile = vi.fn();
      const service = createService(onReceiveFile);
      (service as any).helloComplete = true;
      (service as any).remoteIdentityKey = new Uint8Array(32);
      (service as any).localCapabilities = [];
      (service as any).negotiatedCapabilities = [];

      (service as any).processChunk(makeChunkMsg({
        chunkIndex: 0, totalChunks: 1, transferId: 'n3', fileHash: MOCK_HASH_MATCH,
      }));

      await new Promise(r => setTimeout(r, 20));

      expect(onReceiveFile).toHaveBeenCalledTimes(1);
      service.disconnect();
    });

    it('neither advertises → verification inactive', async () => {
      const onReceiveFile = vi.fn();
      const service = createService(onReceiveFile);
      (service as any).helloComplete = true;
      (service as any).remoteIdentityKey = new Uint8Array(32);
      (service as any).localCapabilities = [];
      (service as any).negotiatedCapabilities = [];

      (service as any).processChunk(makeChunkMsg({
        chunkIndex: 0, totalChunks: 1, transferId: 'n4',
      }));

      await new Promise(r => setTimeout(r, 20));

      expect(onReceiveFile).toHaveBeenCalledTimes(1);
      service.disconnect();
    });
  });

  // ──────────────────────────────────────────────────────────────────────────
  // Success tests
  // ──────────────────────────────────────────────────────────────────────────

  describe('Success', () => {
    it('verification active + hash matches → onReceiveFile called, no error', async () => {
      const onReceiveFile = vi.fn();
      const onError = vi.fn();
      const service = createService(onReceiveFile, onError);
      enableFileHash(service);

      mockHashFileResult = MOCK_HASH_MATCH;

      (service as any).processChunk(makeChunkMsg({
        chunkIndex: 0, totalChunks: 2, transferId: 's1', fileHash: MOCK_HASH_MATCH,
      }));
      (service as any).processChunk(makeChunkMsg({
        chunkIndex: 1, totalChunks: 2, transferId: 's1',
      }));

      await new Promise(r => setTimeout(r, 20));

      expect(onReceiveFile).toHaveBeenCalledTimes(1);
      expect(onError).not.toHaveBeenCalled();
      service.disconnect();
    });

    it('legacy path (no hash field) still completes without error', async () => {
      const onReceiveFile = vi.fn();
      const onError = vi.fn();
      const service = createService(onReceiveFile, onError);
      enableFileHash(service);

      // No fileHash on any chunk — legacy sender
      (service as any).processChunk(makeChunkMsg({
        chunkIndex: 0, totalChunks: 1, transferId: 's2',
      }));

      await new Promise(r => setTimeout(r, 20));

      expect(onReceiveFile).toHaveBeenCalledTimes(1);
      expect(onError).not.toHaveBeenCalled();
      service.disconnect();
    });
  });

  // ──────────────────────────────────────────────────────────────────────────
  // Failure tests
  // ──────────────────────────────────────────────────────────────────────────

  describe('Failure', () => {
    it('verification active + hash mismatch → IntegrityError emitted', async () => {
      const onReceiveFile = vi.fn();
      const onError = vi.fn();
      const service = createService(onReceiveFile, onError);
      attachDataChannel(service);
      enableFileHash(service);

      // hashFile will return MOCK_HASH_MISMATCH (different from expected)
      mockHashFileResult = MOCK_HASH_MISMATCH;

      (service as any).processChunk(makeChunkMsg({
        chunkIndex: 0, totalChunks: 2, transferId: 'f1', fileHash: MOCK_HASH_MATCH,
      }));
      (service as any).processChunk(makeChunkMsg({
        chunkIndex: 1, totalChunks: 2, transferId: 'f1',
      }));

      await new Promise(r => setTimeout(r, 20));

      expect(onReceiveFile).not.toHaveBeenCalled();
      expect(onError).toHaveBeenCalledTimes(1);
      expect(onError.mock.calls[0][0].name).toBe('IntegrityError');
      service.disconnect();
    });

    it('mismatch sends error control message with code INTEGRITY_FAILED', async () => {
      const onReceiveFile = vi.fn();
      const onError = vi.fn();
      const service = createService(onReceiveFile, onError);
      const { sentMessages } = attachDataChannel(service);
      enableFileHash(service);

      mockHashFileResult = MOCK_HASH_MISMATCH;

      (service as any).processChunk(makeChunkMsg({
        chunkIndex: 0, totalChunks: 1, transferId: 'f2', fileHash: MOCK_HASH_MATCH,
      }));

      await new Promise(r => setTimeout(r, 20));

      const errorMsgs = sentMessages.filter(m => {
        try {
          const parsed = JSON.parse(m);
          return parsed.type === 'error' && parsed.code === 'INTEGRITY_FAILED';
        } catch { return false; }
      });
      expect(errorMsgs.length).toBe(1);
      service.disconnect();
    });

    it('mismatch triggers disconnect()', async () => {
      const onReceiveFile = vi.fn();
      const onError = vi.fn();
      const service = createService(onReceiveFile, onError);
      attachDataChannel(service);
      enableFileHash(service);

      mockHashFileResult = MOCK_HASH_MISMATCH;

      (service as any).processChunk(makeChunkMsg({
        chunkIndex: 0, totalChunks: 1, transferId: 'f3', fileHash: MOCK_HASH_MATCH,
      }));

      await new Promise(r => setTimeout(r, 20));

      // After disconnect, helloComplete is reset to false
      expect((service as any).helloComplete).toBe(false);
    });

    it('mismatch clears receiver state maps', async () => {
      const onReceiveFile = vi.fn();
      const onError = vi.fn();
      const service = createService(onReceiveFile, onError);
      attachDataChannel(service);
      enableFileHash(service);

      mockHashFileResult = MOCK_HASH_MISMATCH;

      (service as any).processChunk(makeChunkMsg({
        chunkIndex: 0, totalChunks: 2, transferId: 'f4', fileHash: MOCK_HASH_MATCH,
      }));
      (service as any).processChunk(makeChunkMsg({
        chunkIndex: 1, totalChunks: 2, transferId: 'f4',
      }));

      await new Promise(r => setTimeout(r, 20));

      // All state cleared by disconnect()
      expect((service as any).guardedTransfers.size).toBe(0);
      expect((service as any).recvTransferIds.size).toBe(0);
      expect((service as any).sendTransferIds.size).toBe(0);
    });
  });

  // ──────────────────────────────────────────────────────────────────────────
  // Regression protection
  // ──────────────────────────────────────────────────────────────────────────

  describe('Regression protection', () => {
    it('strict handshake gating unchanged', () => {
      const onError = vi.fn();
      const service = createService(vi.fn(), onError);
      const { sentMessages } = attachDataChannel(service);
      // helloComplete is false by default

      const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

      (service as any).handleMessage({
        data: JSON.stringify({
          type: 'file-chunk', filename: 'evil.bin', chunk: 'x',
          chunkIndex: 0, totalChunks: 1, fileSize: 10,
        }),
      } as MessageEvent);

      const invalidState = warnSpy.mock.calls.filter(
        (a) => typeof a[0] === 'string' && a[0].includes('[INVALID_STATE]')
      );
      expect(invalidState.length).toBe(1);
      expect(onError).toHaveBeenCalled();

      warnSpy.mockRestore();
      service.disconnect();
    });

    it('replay protection unchanged (dedup + OOB)', () => {
      const service = createService();
      enableFileHash(service);

      const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

      // Duplicate
      (service as any).processChunk(makeChunkMsg({ chunkIndex: 0, transferId: 'r1' }));
      (service as any).processChunk(makeChunkMsg({ chunkIndex: 0, transferId: 'r1' }));

      // OOB
      (service as any).processChunk(makeChunkMsg({ chunkIndex: 5, totalChunks: 3, transferId: 'r2' }));

      const dup = warnSpy.mock.calls.filter(a => typeof a[0] === 'string' && a[0].includes('[REPLAY_DUP]'));
      const oob = warnSpy.mock.calls.filter(a => typeof a[0] === 'string' && a[0].includes('[REPLAY_OOB]'));
      expect(dup.length).toBe(1);
      expect(oob.length).toBe(1);

      warnSpy.mockRestore();
      service.disconnect();
    });

    it('pause/resume unchanged', () => {
      const service = createService();
      enableFileHash(service);
      attachDataChannel(service);

      // Should not throw
      service.pauseTransfer('test.bin');
      service.resumeTransfer('test.bin');

      service.disconnect();
    });

    it('cancel unchanged', () => {
      const service = createService();
      enableFileHash(service);
      attachDataChannel(service);

      // Seed a guarded transfer
      (service as any).processChunk(makeChunkMsg({ chunkIndex: 0, totalChunks: 3, transferId: 'c1' }));

      service.cancelTransfer('test.bin', true);
      expect((service as any).guardedTransfers.size).toBe(0);
      expect((service as any).recvTransferIds.size).toBe(0);

      service.disconnect();
    });
  });

  // ──────────────────────────────────────────────────────────────────────────
  // Sender behavior
  // ──────────────────────────────────────────────────────────────────────────

  describe('Sender behavior', () => {
    it('when verification active, sender includes fileHash on first chunk only', async () => {
      const service = createService();
      enableFileHash(service);

      const sentMessages: string[] = [];
      const mockDc = {
        readyState: 'open',
        bufferedAmount: 0,
        bufferedAmountLowThreshold: 0,
        send: (data: string) => sentMessages.push(data),
        onbufferedamountlow: null,
      };
      (service as any).dc = mockDc;

      const file = {
        name: 'hash-test.bin',
        size: 50000,
        arrayBuffer: () => Promise.resolve(new Uint8Array(50000).buffer),
        slice: (start: number, end: number) => ({
          arrayBuffer: () => Promise.resolve(new Uint8Array(end - start).buffer),
        }),
      } as unknown as File;

      await service.sendFile(file);

      const chunks = sentMessages.map(m => JSON.parse(m));
      expect(chunks.length).toBeGreaterThan(1);

      // First chunk has fileHash
      expect(chunks[0].fileHash).toBeDefined();
      expect(typeof chunks[0].fileHash).toBe('string');

      // Subsequent chunks do NOT have fileHash
      for (let i = 1; i < chunks.length; i++) {
        expect(chunks[i].fileHash).toBeUndefined();
      }

      service.disconnect();
    });

    it('fileHash format is 64 hex chars (sha256)', async () => {
      const service = createService();
      enableFileHash(service);

      const sentMessages: string[] = [];
      const mockDc = {
        readyState: 'open',
        bufferedAmount: 0,
        bufferedAmountLowThreshold: 0,
        send: (data: string) => sentMessages.push(data),
        onbufferedamountlow: null,
      };
      (service as any).dc = mockDc;

      const file = {
        name: 'format-test.bin',
        size: 100,
        arrayBuffer: () => Promise.resolve(new Uint8Array(100).buffer),
        slice: (start: number, end: number) => ({
          arrayBuffer: () => Promise.resolve(new Uint8Array(end - start).buffer),
        }),
      } as unknown as File;

      await service.sendFile(file);

      const firstChunk = JSON.parse(sentMessages[0]);
      expect(firstChunk.fileHash).toMatch(/^[0-9a-f]{64}$/);

      service.disconnect();
    });
  });
});
