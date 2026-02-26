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

/**
 * Mock sealBoxPayload/openBoxPayload: deterministic round-trip via hex encoding.
 * sealBoxPayload hex-encodes plaintext bytes.
 * openBoxPayload reverses hex → Uint8Array.
 */
let mockOpenThrow = false;

vi.mock('@the9ines/bolt-core', () => ({
  sealBoxPayload: (data: Uint8Array) => {
    return Array.from(data).map(b => b.toString(16).padStart(2, '0')).join('');
  },
  openBoxPayload: (sealed: string) => {
    if (mockOpenThrow) throw new Error('Decryption failed');
    const bytes = new Uint8Array(sealed.length / 2);
    for (let i = 0; i < sealed.length; i += 2) {
      bytes[i / 2] = parseInt(sealed.substring(i, i + 2), 16);
    }
    return bytes;
  },
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

/** Hex-encode a UTF-8 string (matching mock sealBoxPayload format). */
function hexEncode(str: string): string {
  return Array.from(new TextEncoder().encode(str))
    .map(b => b.toString(16).padStart(2, '0')).join('');
}

/** Build a valid profile-envelope by hex-encoding an inner message. */
function buildEnvelope(innerMsg: Record<string, unknown>): Record<string, unknown> {
  const payload = hexEncode(JSON.stringify(innerMsg));
  return { type: 'profile-envelope', version: 1, encoding: 'base64', payload };
}

function makeChunkMsg(overrides: Record<string, unknown> = {}) {
  return {
    type: 'file-chunk',
    filename: 'test.bin',
    chunk: hexEncode(`chunk-data-${overrides.chunkIndex ?? 0}`),
    chunkIndex: 0,
    totalChunks: 3,
    fileSize: 48,
    ...overrides,
  };
}

/** Hex-decode a string to UTF-8 (reverses mock sealBoxPayload). */
function hexDecode(hex: string): string {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substring(i, i + 2), 16);
  }
  return new TextDecoder().decode(bytes);
}

/** Assert a specific error code was sent and service disconnected.
 *  Checks both plaintext errors and enveloped errors (I5 fix). */
function expectErrorAndDisconnect(
  sentMessages: string[],
  code: string,
  service: any,
) {
  const errorMsgs = sentMessages.filter(m => {
    try {
      const p = JSON.parse(m);
      // Plaintext error (pre-HELLO or no envelope)
      if (p.type === 'error' && p.code === code) return true;
      // Enveloped error (post-HELLO with envelope negotiated)
      if (p.type === 'profile-envelope' && p.payload) {
        const inner = JSON.parse(hexDecode(p.payload));
        return inner.type === 'error' && inner.code === code;
      }
      return false;
    } catch { return false; }
  });
  expect(errorMsgs.length).toBe(1);
  // Disconnected — keyPair zeroed and nulled
  expect((service as any).keyPair).toBe(null);
}

// ─── Tests ───────────────────────────────────────────────────────────────────

describe('H2: WebRTCService Enforcement Compliance', () => {
  let WebRTCService: any;

  beforeEach(async () => {
    mockOpenThrow = false;
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

  /** Post-HELLO state with envelope negotiated. */
  function enableEnvelope(service: any) {
    (service as any).sessionState = 'post_hello';
    (service as any).helloComplete = true;
    (service as any).remoteIdentityKey = new Uint8Array(32);
    (service as any).localCapabilities = ['bolt.file-hash', 'bolt.profile-envelope-v1'];
    (service as any).negotiatedCapabilities = ['bolt.file-hash', 'bolt.profile-envelope-v1'];
  }

  /** Post-HELLO state without envelope negotiated. */
  function disableEnvelope(service: any) {
    (service as any).sessionState = 'post_hello';
    (service as any).helloComplete = true;
    (service as any).remoteIdentityKey = new Uint8Array(32);
    (service as any).localCapabilities = ['bolt.file-hash', 'bolt.profile-envelope-v1'];
    (service as any).negotiatedCapabilities = ['bolt.file-hash'];
  }

  // ──────────────────────────────────────────────────────────────────────────
  // A) Duplicate HELLO after post_hello → DUPLICATE_HELLO + disconnect
  // ──────────────────────────────────────────────────────────────────────────

  it('A. duplicate HELLO after post_hello → DUPLICATE_HELLO + disconnect', () => {
    const service = createService();
    const { sentMessages, injectMessage } = attachDataChannel(service);
    enableEnvelope(service);

    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    // Inject a HELLO after handshake already complete
    injectMessage({
      type: 'hello',
      payload: hexEncode('{"type":"hello","version":1,"identityPublicKey":"abc"}'),
    });

    const dupWarn = warnSpy.mock.calls.filter(
      (a) => typeof a[0] === 'string' && a[0].includes('[DUPLICATE_HELLO]')
    );
    expect(dupWarn.length).toBe(1);
    expectErrorAndDisconnect(sentMessages, 'DUPLICATE_HELLO', service);

    warnSpy.mockRestore();
  });

  // ──────────────────────────────────────────────────────────────────────────
  // B) Non-HELLO before HELLO in HELLO-enabled session → INVALID_STATE
  // ──────────────────────────────────────────────────────────────────────────

  it('B. non-HELLO before HELLO → INVALID_STATE + disconnect', () => {
    const onError = vi.fn();
    const service = createService(vi.fn(), onError);
    const { sentMessages, injectMessage } = attachDataChannel(service);
    // sessionState is 'pre_hello' by default

    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    injectMessage({
      type: 'file-chunk',
      filename: 'evil.bin',
      chunk: 'data',
      chunkIndex: 0,
      totalChunks: 1,
      fileSize: 10,
    });

    const invalidState = warnSpy.mock.calls.filter(
      (a) => typeof a[0] === 'string' && a[0].includes('[INVALID_STATE]')
    );
    expect(invalidState.length).toBe(1);
    expect(onError).toHaveBeenCalledTimes(1);
    expectErrorAndDisconnect(sentMessages, 'INVALID_STATE', service);

    warnSpy.mockRestore();
  });

  // ──────────────────────────────────────────────────────────────────────────
  // C) HELLO parse fail → HELLO_PARSE_ERROR + disconnect
  // ──────────────────────────────────────────────────────────────────────────

  it('C. HELLO parse fail → HELLO_PARSE_ERROR + disconnect', () => {
    const service = createService();
    const { sentMessages, injectMessage } = attachDataChannel(service);

    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    // Payload decrypts to "not valid json" — JSON.parse will throw
    injectMessage({
      type: 'hello',
      payload: hexEncode('not valid json'),
    });

    const parseErr = warnSpy.mock.calls.filter(
      (a) => typeof a[0] === 'string' && a[0].includes('[HELLO_PARSE_ERROR]')
    );
    expect(parseErr.length).toBe(1);
    expectErrorAndDisconnect(sentMessages, 'HELLO_PARSE_ERROR', service);

    warnSpy.mockRestore();
  });

  // ──────────────────────────────────────────────────────────────────────────
  // D) HELLO decrypt fail → HELLO_DECRYPT_FAIL + disconnect
  // ──────────────────────────────────────────────────────────────────────────

  it('D. HELLO decrypt fail → HELLO_DECRYPT_FAIL + disconnect', () => {
    const service = createService();
    const { sentMessages, injectMessage } = attachDataChannel(service);

    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    mockOpenThrow = true;

    injectMessage({
      type: 'hello',
      payload: 'some-encrypted-payload',
    });

    const decryptFail = warnSpy.mock.calls.filter(
      (a) => typeof a[0] === 'string' && a[0].includes('[HELLO_DECRYPT_FAIL]')
    );
    expect(decryptFail.length).toBe(1);
    expectErrorAndDisconnect(sentMessages, 'HELLO_DECRYPT_FAIL', service);

    warnSpy.mockRestore();
  });

  // ──────────────────────────────────────────────────────────────────────────
  // E) Envelope negotiated + plaintext received → ENVELOPE_REQUIRED
  // ──────────────────────────────────────────────────────────────────────────

  it('E. plaintext in envelope-required session → ENVELOPE_REQUIRED + disconnect', () => {
    const service = createService();
    const { sentMessages, injectMessage } = attachDataChannel(service);
    enableEnvelope(service);

    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    // Send plaintext file-chunk when envelope is negotiated
    injectMessage({
      type: 'file-chunk',
      filename: 'test.bin',
      chunk: 'plaintext-data',
      chunkIndex: 0,
      totalChunks: 1,
      fileSize: 10,
    });

    const envRequired = warnSpy.mock.calls.filter(
      (a) => typeof a[0] === 'string' && a[0].includes('[ENVELOPE_REQUIRED]')
    );
    expect(envRequired.length).toBe(1);
    expectErrorAndDisconnect(sentMessages, 'ENVELOPE_REQUIRED', service);

    warnSpy.mockRestore();
  });

  // ──────────────────────────────────────────────────────────────────────────
  // F) Envelope invalid JSON/version/encoding → ENVELOPE_INVALID
  // ──────────────────────────────────────────────────────────────────────────

  it('F. invalid envelope version → ENVELOPE_INVALID + disconnect', () => {
    const service = createService();
    const { sentMessages, injectMessage } = attachDataChannel(service);
    enableEnvelope(service);

    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    injectMessage({
      type: 'profile-envelope',
      version: 2,
      encoding: 'base64',
      payload: 'some-data',
    });

    const invalid = warnSpy.mock.calls.filter(
      (a) => typeof a[0] === 'string' && a[0].includes('[ENVELOPE_INVALID]')
    );
    expect(invalid.length).toBe(1);
    expectErrorAndDisconnect(sentMessages, 'ENVELOPE_INVALID', service);

    warnSpy.mockRestore();
  });

  // ──────────────────────────────────────────────────────────────────────────
  // G) Envelope decrypt fail → ENVELOPE_DECRYPT_FAIL
  // ──────────────────────────────────────────────────────────────────────────

  it('G. envelope decrypt fail → ENVELOPE_DECRYPT_FAIL + disconnect', () => {
    const service = createService();
    const { sentMessages, injectMessage } = attachDataChannel(service);
    enableEnvelope(service);

    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    mockOpenThrow = true;

    injectMessage({
      type: 'profile-envelope',
      version: 1,
      encoding: 'base64',
      payload: 'encrypted-data-that-will-fail',
    });

    const decryptFail = warnSpy.mock.calls.filter(
      (a) => typeof a[0] === 'string' && a[0].includes('[ENVELOPE_DECRYPT_FAIL]')
    );
    expect(decryptFail.length).toBe(1);
    expectErrorAndDisconnect(sentMessages, 'ENVELOPE_DECRYPT_FAIL', service);

    warnSpy.mockRestore();
  });

  // ──────────────────────────────────────────────────────────────────────────
  // H) Inner parse fail → INVALID_MESSAGE
  // ──────────────────────────────────────────────────────────────────────────

  it('H. inner JSON parse fail → INVALID_MESSAGE + disconnect', () => {
    const service = createService();
    const { sentMessages, injectMessage } = attachDataChannel(service);
    enableEnvelope(service);

    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    // Envelope with payload that decrypts to non-JSON
    injectMessage({
      type: 'profile-envelope',
      version: 1,
      encoding: 'base64',
      payload: hexEncode('this is not json'),
    });

    const invalidMsg = warnSpy.mock.calls.filter(
      (a) => typeof a[0] === 'string' && a[0].includes('[INVALID_MESSAGE]')
    );
    expect(invalidMsg.length).toBe(1);
    expectErrorAndDisconnect(sentMessages, 'INVALID_MESSAGE', service);

    warnSpy.mockRestore();
  });

  // ──────────────────────────────────────────────────────────────────────────
  // I) Unknown inner type → UNKNOWN_MESSAGE_TYPE
  // ──────────────────────────────────────────────────────────────────────────

  it('I. unknown inner message type → UNKNOWN_MESSAGE_TYPE + disconnect', () => {
    const service = createService();
    const { sentMessages, injectMessage } = attachDataChannel(service);
    enableEnvelope(service);

    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    // Valid envelope wrapping an unknown message type
    injectMessage(buildEnvelope({ type: 'unknown-thing', data: 'whatever' }));

    const unknownType = warnSpy.mock.calls.filter(
      (a) => typeof a[0] === 'string' && a[0].includes('[UNKNOWN_MESSAGE_TYPE]')
    );
    expect(unknownType.length).toBe(1);
    expectErrorAndDisconnect(sentMessages, 'UNKNOWN_MESSAGE_TYPE', service);

    warnSpy.mockRestore();
  });

  // ──────────────────────────────────────────────────────────────────────────
  // J) Envelope received when not negotiated → ENVELOPE_UNNEGOTIATED
  // ──────────────────────────────────────────────────────────────────────────

  it('J. envelope when not negotiated → ENVELOPE_UNNEGOTIATED + disconnect', () => {
    const service = createService();
    const { sentMessages, injectMessage } = attachDataChannel(service);
    disableEnvelope(service);

    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    injectMessage({
      type: 'profile-envelope',
      version: 1,
      encoding: 'base64',
      payload: 'some-payload',
    });

    const unneg = warnSpy.mock.calls.filter(
      (a) => typeof a[0] === 'string' && a[0].includes('[ENVELOPE_UNNEGOTIATED]')
    );
    expect(unneg.length).toBe(1);
    expectErrorAndDisconnect(sentMessages, 'ENVELOPE_UNNEGOTIATED', service);

    warnSpy.mockRestore();
  });

  // ──────────────────────────────────────────────────────────────────────────
  // K) Skip-HELLO legacy path → sessionState=post_hello + plaintext routing
  // ──────────────────────────────────────────────────────────────────────────

  it('K. skip-HELLO legacy path sets post_hello and permits plaintext', async () => {
    const onReceiveFile = vi.fn();
    // No identityPublicKey → skip-HELLO path
    const service = createService(onReceiveFile, vi.fn(), undefined, {});
    const { injectMessage } = attachDataChannel(service);

    // Trigger initiateHello directly — it will skip HELLO (no identity keys)
    (service as any).initiateHello();

    // Verify state transition
    expect((service as any).sessionState).toBe('post_hello');
    expect((service as any).helloComplete).toBe(true);
    expect((service as any).sessionLegacy).toBe(true);

    // Plaintext file-chunk should route normally (no envelope negotiated)
    injectMessage({
      type: 'file-chunk',
      filename: 'legacy.bin',
      chunk: hexEncode('legacy-data'),
      chunkIndex: 0,
      totalChunks: 1,
      fileSize: 10,
      transferId: 'legacy-tid-1',
    });

    await new Promise(r => setTimeout(r, 20));

    expect(onReceiveFile).toHaveBeenCalledTimes(1);
    expect(onReceiveFile.mock.calls[0][1]).toBe('legacy.bin');

    service.disconnect();
  });

  // ──────────────────────────────────────────────────────────────────────────
  // Additional: HELLO schema error
  // ──────────────────────────────────────────────────────────────────────────

  it('HELLO schema error (missing identityPublicKey) → HELLO_SCHEMA_ERROR + disconnect', () => {
    const service = createService();
    const { sentMessages, injectMessage } = attachDataChannel(service);

    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    // Valid JSON but missing required identityPublicKey field
    injectMessage({
      type: 'hello',
      payload: hexEncode('{"type":"hello","version":1}'),
    });

    const schemaErr = warnSpy.mock.calls.filter(
      (a) => typeof a[0] === 'string' && a[0].includes('[HELLO_SCHEMA_ERROR]')
    );
    expect(schemaErr.length).toBe(1);
    expectErrorAndDisconnect(sentMessages, 'HELLO_SCHEMA_ERROR', service);

    warnSpy.mockRestore();
  });

  // ──────────────────────────────────────────────────────────────────────────
  // Additional: Top-level catch → PROTOCOL_VIOLATION
  // ──────────────────────────────────────────────────────────────────────────

  it('unparseable message → PROTOCOL_VIOLATION + disconnect', () => {
    const service = createService();
    const { dc, sentMessages } = attachDataChannel(service);
    enableEnvelope(service);

    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    // Inject raw non-JSON string — JSON.parse will throw
    if (dc.onmessage) {
      dc.onmessage({ data: 'not json at all' } as MessageEvent);
    }

    const violation = errorSpy.mock.calls.filter(
      (a) => typeof a[0] === 'string' && a[0].includes('[PROTOCOL_VIOLATION]')
    );
    expect(violation.length).toBe(1);
    expectErrorAndDisconnect(sentMessages, 'PROTOCOL_VIOLATION', service);

    errorSpy.mockRestore();
  });

  // ──────────────────────────────────────────────────────────────────────────
  // State model: sessionState transitions
  // ──────────────────────────────────────────────────────────────────────────

  it('sessionState transitions: pre_hello → post_hello → closed', () => {
    const service = createService();
    attachDataChannel(service);

    // Initial state
    expect((service as any).sessionState).toBe('pre_hello');

    // After HELLO complete
    (service as any).sessionState = 'post_hello';
    (service as any).helloComplete = true;
    expect((service as any).sessionState).toBe('post_hello');

    // After disconnect
    service.disconnect();
    expect((service as any).sessionState).toBe('closed');
  });

  // ──────────────────────────────────────────────────────────────────────────
  // Non-envelope session: plaintext allowed
  // ──────────────────────────────────────────────────────────────────────────

  it('non-envelope session permits plaintext routing after HELLO', async () => {
    const onReceiveFile = vi.fn();
    const service = createService(onReceiveFile);
    const { injectMessage } = attachDataChannel(service);
    disableEnvelope(service);

    // Plaintext file-chunk should route normally
    injectMessage({
      type: 'file-chunk',
      filename: 'plain.bin',
      chunk: hexEncode('plain-data'),
      chunkIndex: 0,
      totalChunks: 1,
      fileSize: 10,
      transferId: 'plain-tid-1',
    });

    await new Promise(r => setTimeout(r, 20));

    expect(onReceiveFile).toHaveBeenCalledTimes(1);
    expect(onReceiveFile.mock.calls[0][1]).toBe('plain.bin');

    service.disconnect();
  });
});
