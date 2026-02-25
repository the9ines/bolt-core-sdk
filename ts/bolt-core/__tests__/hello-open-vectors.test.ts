import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { openBoxPayload, sealBoxPayload, generateEphemeralKeyPair } from '../src/index.js';

const __dirname = dirname(fileURLToPath(import.meta.url));
const vectorPath = join(__dirname, 'vectors', 'web-hello-open.vectors.json');
const vectors = JSON.parse(readFileSync(vectorPath, 'utf-8'));

function fromHex(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substring(i, i + 2), 16);
  }
  return bytes;
}

describe('HELLO open golden vectors', () => {
  for (const c of vectors.cases) {
    it(`${c.name} — opens and decodes to expected inner`, () => {
      const senderPk = fromHex(c.sender_public_hex);
      const receiverSk = fromHex(c.receiver_secret_hex);

      const decrypted = openBoxPayload(c.sealed_payload_base64, senderPk, receiverSk);
      const inner = JSON.parse(new TextDecoder().decode(decrypted));

      expect(inner).toEqual(c.expected_inner);
    });
  }

  it('wrong key rejects all HELLO vectors', () => {
    const wrongKey = new Uint8Array(32);
    wrongKey[0] = 0xFF;

    for (const c of vectors.cases) {
      const receiverSk = fromHex(c.receiver_secret_hex);
      expect(() => openBoxPayload(c.sealed_payload_base64, wrongKey, receiverSk)).toThrow();
    }
  });
});

describe('HELLO seal-then-open round-trip (non-golden)', () => {
  it('sealing then opening recovers original HELLO inner', () => {
    const kpA = generateEphemeralKeyPair();
    const kpB = generateEphemeralKeyPair();

    const inner = {
      type: 'hello',
      version: 1,
      identityPublicKey: 'dGVzdA==',
      capabilities: ['bolt.file-hash', 'bolt.profile-envelope-v1'],
    };
    const plaintext = new TextEncoder().encode(JSON.stringify(inner));

    const sealed = sealBoxPayload(plaintext, kpB.publicKey, kpA.secretKey);
    const decrypted = openBoxPayload(sealed, kpA.publicKey, kpB.secretKey);
    const recovered = JSON.parse(new TextDecoder().decode(decrypted));

    expect(recovered).toEqual(inner);
  });

  it('different seals produce different ciphertext (random nonce)', () => {
    const kpA = generateEphemeralKeyPair();
    const kpB = generateEphemeralKeyPair();

    const plaintext = new TextEncoder().encode('{"type":"hello","version":1}');
    const sealed1 = sealBoxPayload(plaintext, kpB.publicKey, kpA.secretKey);
    const sealed2 = sealBoxPayload(plaintext, kpB.publicKey, kpA.secretKey);

    expect(sealed1).not.toBe(sealed2);
  });
});
