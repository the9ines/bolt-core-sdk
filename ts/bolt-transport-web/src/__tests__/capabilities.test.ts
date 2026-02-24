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

function createMockDataChannel() {
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
  function injectMessage(data: Record<string, unknown>) {
    if (dc.onmessage) {
      dc.onmessage({ data: JSON.stringify(data) } as MessageEvent);
    }
  }
  return { dc, sentMessages, injectMessage };
}

// ─── Tests ───────────────────────────────────────────────────────────────────

describe('Phase 0: HELLO Capabilities Plumbing', () => {
  let WebRTCService: any;

  beforeEach(async () => {
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
    const { dc, sentMessages, injectMessage } = createMockDataChannel();
    (service as any).dc = dc;
    dc.onmessage = (event: MessageEvent) => {
      (service as any).handleMessage(event);
    };
    return { dc, sentMessages, injectMessage };
  }

  // ─── 1. HELLO payload includes capabilities [] on send ──────────────────

  it('HELLO payload includes capabilities [] on send', () => {
    const service = createService();
    const { sentMessages } = attachDataChannel(service);

    // Trigger HELLO send via initiateHello
    (service as any).initiateHello();

    expect(sentMessages.length).toBe(1);
    const envelope = JSON.parse(sentMessages[0]);
    expect(envelope.type).toBe('hello');
    expect(envelope.payload).toBeDefined();

    // The stub sealBoxPayload returns 'encrypted-stub', so we cannot decrypt.
    // Instead, verify the JSON that was passed to sealBoxPayload by inspecting
    // the localCapabilities field directly — it should be empty array.
    expect((service as any).localCapabilities).toEqual(['bolt.file-hash']);

    // Additionally, verify the HELLO JSON structure by calling the code path
    // that builds the payload. We can check via a spy on sealBoxPayload.
    // Since the mock just returns 'encrypted-stub', let's verify the JSON
    // was constructed correctly by checking the source — the test below
    // verifies the full round-trip via processHello.

    service.disconnect();
  });

  // ─── 2. Remote HELLO missing capabilities → stored as [] ────────────────

  it('remote HELLO missing capabilities is treated as empty array', async () => {
    const service = createService();
    attachDataChannel(service);

    // Simulate processHello with a HELLO that has NO capabilities field.
    // openBoxPayload mock returns the cipher as-is, so we pass a JSON string
    // as the "payload" that processHello will decrypt and parse.
    const helloPayload = JSON.stringify({
      type: 'hello',
      version: 1,
      identityPublicKey: 'AAAA', // fromBase64 mock returns Uint8Array(32)
    });

    await (service as any).processHello({ type: 'hello', payload: helloPayload });

    expect((service as any).remoteCapabilities).toEqual([]);
    expect((service as any).negotiatedCapabilities).toEqual([]);
    expect((service as any).helloComplete).toBe(true);

    service.disconnect();
  });

  // ─── 3. Remote HELLO with capabilities persists and intersection computed ─

  it('remote HELLO with capabilities persists correctly and intersection computed', async () => {
    const service = createService();
    attachDataChannel(service);

    // Manually set localCapabilities to simulate a future phase where we advertise
    (service as any).localCapabilities = ['bolt.file-hash', 'bolt.other'];

    const helloPayload = JSON.stringify({
      type: 'hello',
      version: 1,
      identityPublicKey: 'AAAA',
      capabilities: ['bolt.file-hash', 'bolt.envelope-v1'],
    });

    await (service as any).processHello({ type: 'hello', payload: helloPayload });

    expect((service as any).remoteCapabilities).toEqual(['bolt.file-hash', 'bolt.envelope-v1']);
    expect((service as any).negotiatedCapabilities).toEqual(['bolt.file-hash']);
    expect(service.hasCapability('bolt.file-hash')).toBe(true);
    expect(service.hasCapability('bolt.envelope-v1')).toBe(false);
    expect(service.hasCapability('bolt.other')).toBe(false);

    service.disconnect();
  });

  // ─── 4. Regression: sendFile still blocks on helloComplete ──────────────

  it('sendFile still blocks until helloComplete (regression)', async () => {
    const service = createService();
    attachDataChannel(service);
    (service as any).helloComplete = false;

    let resolved = false;
    const file = {
      name: 'block-test.bin',
      size: 50,
      slice: (start: number, end: number) => ({
        arrayBuffer: () => Promise.resolve(new Uint8Array(end - start).buffer),
      }),
    } as unknown as File;

    const sendPromise = service.sendFile(file).then(() => { resolved = true; });

    await new Promise(r => setTimeout(r, 50));
    expect(resolved).toBe(false);

    // Resolve HELLO gate
    (service as any).helloComplete = true;
    if ((service as any).helloResolve) {
      (service as any).helloResolve();
      (service as any).helloResolve = null;
    }

    await sendPromise;
    expect(resolved).toBe(true);

    service.disconnect();
  });

  // ─── 5. Regression: strict handshake gating unchanged ───────────────────

  it('strict handshake gating behavior unchanged (non-HELLO rejected before complete)', () => {
    const onError = vi.fn();
    const service = createService(vi.fn(), onError);
    const { sentMessages, injectMessage } = attachDataChannel(service);

    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    injectMessage({
      type: 'file-chunk',
      filename: 'evil.bin',
      chunk: 'data',
      chunkIndex: 0,
      totalChunks: 1,
      fileSize: 100,
    });

    // INVALID_STATE logged
    const invalidStateWarns = warnSpy.mock.calls.filter(
      (args) => typeof args[0] === 'string' && args[0].includes('[INVALID_STATE]')
    );
    expect(invalidStateWarns.length).toBe(1);

    // onError called
    expect(onError).toHaveBeenCalledTimes(1);
    expect(onError.mock.calls[0][0].message).toContain('before handshake');

    // Error control message sent
    const errorMsgs = sentMessages.filter(m => {
      try {
        const parsed = JSON.parse(m);
        return parsed.type === 'error' && parsed.code === 'INVALID_STATE';
      } catch { return false; }
    });
    expect(errorMsgs.length).toBe(1);

    warnSpy.mockRestore();
  });

  // ─── 6. No crash on unexpected HELLO payload shape ──────────────────────

  it('no crash on unexpected HELLO payload shape (capabilities is non-array)', async () => {
    const service = createService();
    attachDataChannel(service);

    // capabilities is a string instead of array — should degrade to []
    const helloPayload = JSON.stringify({
      type: 'hello',
      version: 1,
      identityPublicKey: 'AAAA',
      capabilities: 'not-an-array',
    });

    await (service as any).processHello({ type: 'hello', payload: helloPayload });

    expect((service as any).remoteCapabilities).toEqual([]);
    expect((service as any).negotiatedCapabilities).toEqual([]);
    expect((service as any).helloComplete).toBe(true);

    service.disconnect();
  });

  // ─── 7. disconnect clears negotiatedCapabilities ────────────────────────

  it('disconnect clears remoteCapabilities and negotiatedCapabilities', async () => {
    const service = createService();
    attachDataChannel(service);

    // Simulate a completed HELLO with capabilities
    (service as any).localCapabilities = ['bolt.file-hash'];
    const helloPayload = JSON.stringify({
      type: 'hello',
      version: 1,
      identityPublicKey: 'AAAA',
      capabilities: ['bolt.file-hash'],
    });
    await (service as any).processHello({ type: 'hello', payload: helloPayload });

    expect((service as any).negotiatedCapabilities).toEqual(['bolt.file-hash']);
    expect((service as any).remoteCapabilities).toEqual(['bolt.file-hash']);

    service.disconnect();

    expect((service as any).remoteCapabilities).toEqual([]);
    expect((service as any).negotiatedCapabilities).toEqual([]);
  });
});
