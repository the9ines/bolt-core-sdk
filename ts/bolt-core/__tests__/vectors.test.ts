import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { openBoxPayload, fromBase64 } from '../src/index.js';
import { NONCE_LENGTH } from '../src/constants.js';

const __dirname = dirname(fileURLToPath(import.meta.url));
const vectorDir = join(__dirname, 'vectors');

const boxPayload = JSON.parse(readFileSync(join(vectorDir, 'box-payload.vectors.json'), 'utf-8'));
const framing = JSON.parse(readFileSync(join(vectorDir, 'framing.vectors.json'), 'utf-8'));

// ── Helpers ──────────────────────────────────────────────

function hexToBytes(hex: string): Uint8Array {
  if (hex.length === 0) return new Uint8Array(0);
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(hex.substring(i * 2, i * 2 + 2), 16);
  }
  return bytes;
}

function toHex(arr: Uint8Array): string {
  return Array.from(arr).map(b => b.toString(16).padStart(2, '0')).join('');
}

const receiverSecretKey = fromBase64(boxPayload.receiver.secretKey_base64);
const senderPublicKey = fromBase64(boxPayload.sender.publicKey_base64);
const evePublicKey = fromBase64(boxPayload.eve.publicKey_base64);

// ── Box payload: open vectors ────────────────────────────

describe('Deterministic box-payload vectors', () => {
  for (const vec of boxPayload.vectors) {
    it(`opens vector: ${vec.id}`, () => {
      const plaintext = openBoxPayload(
        vec.sealed_base64,
        senderPublicKey,
        receiverSecretKey,
      );
      expect(toHex(plaintext)).toBe(vec.plaintext_hex);
    });
  }

  it('hello-bolt decrypts to expected UTF-8 string', () => {
    const vec = boxPayload.vectors.find((v: { id: string }) => v.id === 'hello-bolt');
    const plaintext = openBoxPayload(vec.sealed_base64, senderPublicKey, receiverSecretKey);
    expect(new TextDecoder().decode(plaintext)).toBe('Hello, Bolt!');
  });
});

// ── Box payload: corrupt vectors ─────────────────────────

describe('Corrupt box-payload vectors', () => {
  for (const vec of boxPayload.corrupt_vectors) {
    it(`rejects: ${vec.id}`, () => {
      const useSenderPub = vec.use_eve_as_sender ? evePublicKey : senderPublicKey;
      expect(() => {
        openBoxPayload(vec.sealed_base64, useSenderPub, receiverSecretKey);
      }).toThrow(vec.expected_error);
    });
  }
});

// ── Framing: nonce||ciphertext layout ────────────────────

describe('Wire format framing vectors', () => {
  it('nonce length constant matches vector spec', () => {
    expect(NONCE_LENGTH).toBe(framing.constants.nonce_length);
  });

  for (const vec of framing.vectors) {
    describe(`framing: ${vec.id}`, () => {
      const decoded = fromBase64(vec.sealed_base64);

      it('decoded length matches expected', () => {
        expect(decoded.length).toBe(vec.expected_decoded_length);
      });

      it('first 24 bytes are the nonce', () => {
        const nonce = decoded.slice(0, 24);
        expect(toHex(nonce)).toBe(vec.expected_nonce_hex);
      });

      it('ciphertext length = plaintext_length + box_overhead(16)', () => {
        const ciphertext = decoded.slice(24);
        expect(ciphertext.length).toBe(vec.expected_ciphertext_length);
      });

      it('total = nonce(24) + plaintext + overhead(16)', () => {
        expect(decoded.length).toBe(24 + vec.plaintext_length + framing.constants.box_overhead);
      });
    });
  }
});

// ── Deterministic regeneration check ─────────────────────

describe('Vector file integrity', () => {
  it('box-payload vectors have 4 seal vectors', () => {
    expect(boxPayload.vectors.length).toBe(4);
  });

  it('box-payload vectors have 4 corrupt vectors', () => {
    expect(boxPayload.corrupt_vectors.length).toBe(4);
  });

  it('framing vectors have 4 entries', () => {
    expect(framing.vectors.length).toBe(4);
  });

  it('all sealed payloads are valid base64', () => {
    for (const vec of boxPayload.vectors) {
      expect(() => fromBase64(vec.sealed_base64)).not.toThrow();
    }
    for (const vec of boxPayload.corrupt_vectors) {
      expect(() => fromBase64(vec.sealed_base64)).not.toThrow();
    }
  });
});
