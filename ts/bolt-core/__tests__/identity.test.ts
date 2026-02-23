import { describe, it, expect } from 'vitest';
import { generateIdentityKeyPair, KeyMismatchError, PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH } from '../src/index.js';

describe('Identity keypair generation', () => {
  it('returns publicKey and secretKey of correct length', () => {
    const kp = generateIdentityKeyPair();
    expect(kp.publicKey).toBeInstanceOf(Uint8Array);
    expect(kp.secretKey).toBeInstanceOf(Uint8Array);
    expect(kp.publicKey.length).toBe(PUBLIC_KEY_LENGTH);
    expect(kp.secretKey.length).toBe(SECRET_KEY_LENGTH);
  });

  it('produces distinct keypairs on each call', () => {
    const a = generateIdentityKeyPair();
    const b = generateIdentityKeyPair();
    expect(a.publicKey).not.toEqual(b.publicKey);
    expect(a.secretKey).not.toEqual(b.secretKey);
  });

  it('does not return all-zero keys', () => {
    const kp = generateIdentityKeyPair();
    expect(kp.publicKey.some((b) => b !== 0)).toBe(true);
    expect(kp.secretKey.some((b) => b !== 0)).toBe(true);
  });
});

describe('KeyMismatchError', () => {
  it('is an instance of Error and BoltError', () => {
    const expected = new Uint8Array(32).fill(1);
    const received = new Uint8Array(32).fill(2);
    const err = new KeyMismatchError('ABC123', expected, received);

    expect(err).toBeInstanceOf(Error);
    expect(err.name).toBe('KeyMismatchError');
    expect(err.peerCode).toBe('ABC123');
    expect(err.expected).toBe(expected);
    expect(err.received).toBe(received);
    expect(err.message).toContain('ABC123');
  });
});
