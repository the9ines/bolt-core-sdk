import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { openBoxPayload, sealBoxPayload, generateEphemeralKeyPair } from '../src/index.js';

const __dirname = dirname(fileURLToPath(import.meta.url));
const vectorPath = join(__dirname, 'vectors', 'envelope-open.vectors.json');
const vectors = JSON.parse(readFileSync(vectorPath, 'utf-8'));

function fromHex(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substring(i, i + 2), 16);
  }
  return bytes;
}

describe('envelope open golden vectors', () => {
  for (const c of vectors.cases) {
    it(`${c.name} — opens envelope and decodes inner`, () => {
      const senderPk = fromHex(c.sender_public_hex);
      const receiverSk = fromHex(c.receiver_secret_hex);
      const envelope = c.envelope_json;

      // Validate envelope frame structure
      expect(envelope.type).toBe('profile-envelope');
      expect(envelope.version).toBe(1);
      expect(envelope.encoding).toBe('base64');
      expect(typeof envelope.payload).toBe('string');

      // Open the sealed payload
      const decrypted = openBoxPayload(envelope.payload, senderPk, receiverSk);
      const inner = JSON.parse(new TextDecoder().decode(decrypted));

      expect(inner).toEqual(c.expected_inner);
    });
  }

  it('wrong key rejects all envelope vectors', () => {
    const wrongKey = new Uint8Array(32);
    wrongKey[0] = 0xFF;

    for (const c of vectors.cases) {
      const receiverSk = fromHex(c.receiver_secret_hex);
      expect(() => openBoxPayload(c.envelope_json.payload, wrongKey, receiverSk)).toThrow();
    }
  });
});

describe('envelope seal-then-open round-trip (non-golden)', () => {
  it('sealing then opening inside envelope structure recovers inner', () => {
    const kpA = generateEphemeralKeyPair();
    const kpB = generateEphemeralKeyPair();

    const inner = { type: 'ping', ts_ms: 1700000000000 };
    const innerBytes = new TextEncoder().encode(JSON.stringify(inner));

    const sealedPayload = sealBoxPayload(innerBytes, kpB.publicKey, kpA.secretKey);
    const envelope = {
      type: 'profile-envelope' as const,
      version: 1,
      encoding: 'base64' as const,
      payload: sealedPayload,
    };

    // Decode
    const decrypted = openBoxPayload(envelope.payload, kpA.publicKey, kpB.secretKey);
    const recovered = JSON.parse(new TextDecoder().decode(decrypted));

    expect(recovered).toEqual(inner);
  });

  it('round-trip works with all inner message types', () => {
    const kpA = generateEphemeralKeyPair();
    const kpB = generateEphemeralKeyPair();

    const messages = [
      { type: 'ping', ts_ms: 12345 },
      { type: 'pong', ts_ms: 12346, reply_to_ms: 12345 },
      { type: 'app_message', text: 'round trip test' },
    ];

    for (const msg of messages) {
      const innerBytes = new TextEncoder().encode(JSON.stringify(msg));
      const sealed = sealBoxPayload(innerBytes, kpB.publicKey, kpA.secretKey);
      const decrypted = openBoxPayload(sealed, kpA.publicKey, kpB.secretKey);
      const recovered = JSON.parse(new TextDecoder().decode(decrypted));
      expect(recovered).toEqual(msg);
    }
  });
});
