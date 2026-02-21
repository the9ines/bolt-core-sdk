import { describe, it, expect } from 'vitest';
import { box } from 'tweetnacl';
import {
  sealBoxPayload,
  openBoxPayload,
  generateEphemeralKeyPair,
  NONCE_LENGTH,
  PUBLIC_KEY_LENGTH,
} from '../src/index.js';

describe('generateEphemeralKeyPair', () => {
  it('returns 32-byte public and secret keys', () => {
    const kp = generateEphemeralKeyPair();
    expect(kp.publicKey.length).toBe(PUBLIC_KEY_LENGTH);
    expect(kp.secretKey.length).toBe(PUBLIC_KEY_LENGTH);
  });

  it('generates unique keypairs each time', () => {
    const kp1 = generateEphemeralKeyPair();
    const kp2 = generateEphemeralKeyPair();
    expect(kp1.publicKey).not.toEqual(kp2.publicKey);
    expect(kp1.secretKey).not.toEqual(kp2.secretKey);
  });
});

describe('sealBoxPayload / openBoxPayload', () => {
  it('round-trips plaintext correctly', () => {
    const alice = generateEphemeralKeyPair();
    const bob = generateEphemeralKeyPair();

    const plaintext = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]);
    const sealed = sealBoxPayload(plaintext, bob.publicKey, alice.secretKey);
    const opened = openBoxPayload(sealed, alice.publicKey, bob.secretKey);

    expect(opened).toEqual(plaintext);
  });

  it('produces different ciphertext for same plaintext (random nonce)', () => {
    const alice = generateEphemeralKeyPair();
    const bob = generateEphemeralKeyPair();

    const plaintext = new Uint8Array([10, 20, 30]);
    const sealed1 = sealBoxPayload(plaintext, bob.publicKey, alice.secretKey);
    const sealed2 = sealBoxPayload(plaintext, bob.publicKey, alice.secretKey);

    expect(sealed1).not.toEqual(sealed2);
  });

  it('fails to decrypt with wrong key', () => {
    const alice = generateEphemeralKeyPair();
    const bob = generateEphemeralKeyPair();
    const eve = generateEphemeralKeyPair();

    const plaintext = new Uint8Array([42, 43, 44]);
    const sealed = sealBoxPayload(plaintext, bob.publicKey, alice.secretKey);

    expect(() => {
      openBoxPayload(sealed, eve.publicKey, bob.secretKey);
    }).toThrow('Decryption failed');
  });

  it('handles empty payload', () => {
    const alice = generateEphemeralKeyPair();
    const bob = generateEphemeralKeyPair();

    const plaintext = new Uint8Array([]);
    const sealed = sealBoxPayload(plaintext, bob.publicKey, alice.secretKey);
    const opened = openBoxPayload(sealed, alice.publicKey, bob.secretKey);

    expect(opened).toEqual(plaintext);
  });

  it('handles 16KB payload (actual transfer chunk size)', () => {
    const alice = generateEphemeralKeyPair();
    const bob = generateEphemeralKeyPair();

    const plaintext = new Uint8Array(16384);
    for (let i = 0; i < plaintext.length; i++) {
      plaintext[i] = i % 256;
    }

    const sealed = sealBoxPayload(plaintext, bob.publicKey, alice.secretKey);
    const opened = openBoxPayload(sealed, alice.publicKey, bob.secretKey);

    expect(opened).toEqual(plaintext);
  });

  it('nonce length is 24 bytes', () => {
    expect(box.nonceLength).toBe(NONCE_LENGTH);
  });
});
