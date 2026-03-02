import { describe, it, expect } from 'vitest';
import {
  NONCE_LENGTH,
  PUBLIC_KEY_LENGTH,
  SECRET_KEY_LENGTH,
  DEFAULT_CHUNK_SIZE,
  PEER_CODE_LENGTH,
  PEER_CODE_ALPHABET,
  SAS_LENGTH,
  BOLT_VERSION,
  TRANSFER_ID_LENGTH,
  SAS_ENTROPY,
  FILE_HASH_ALGORITHM,
  FILE_HASH_LENGTH,
  CAPABILITY_NAMESPACE,
} from '../src/constants.js';

describe('§14 Protocol Constants', () => {
  it('NONCE_LENGTH = 24', () => {
    expect(NONCE_LENGTH).toBe(24);
  });

  it('PUBLIC_KEY_LENGTH = 32', () => {
    expect(PUBLIC_KEY_LENGTH).toBe(32);
  });

  it('SECRET_KEY_LENGTH = 32', () => {
    expect(SECRET_KEY_LENGTH).toBe(32);
  });

  it('DEFAULT_CHUNK_SIZE = 16384', () => {
    expect(DEFAULT_CHUNK_SIZE).toBe(16384);
  });

  it('TRANSFER_ID_LENGTH = 16', () => {
    expect(TRANSFER_ID_LENGTH).toBe(16);
  });

  it('PEER_CODE_LENGTH = 6', () => {
    expect(PEER_CODE_LENGTH).toBe(6);
  });

  it('PEER_CODE_ALPHABET has 31 unambiguous characters', () => {
    expect(PEER_CODE_ALPHABET).toBe('ABCDEFGHJKMNPQRSTUVWXYZ23456789');
    expect(PEER_CODE_ALPHABET).toHaveLength(31);
    expect(PEER_CODE_ALPHABET).not.toContain('0');
    expect(PEER_CODE_ALPHABET).not.toContain('O');
    expect(PEER_CODE_ALPHABET).not.toContain('1');
    expect(PEER_CODE_ALPHABET).not.toContain('I');
    expect(PEER_CODE_ALPHABET).not.toContain('L');
  });

  it('SAS_LENGTH = 6', () => {
    expect(SAS_LENGTH).toBe(6);
  });

  it('SAS_ENTROPY = 24', () => {
    expect(SAS_ENTROPY).toBe(24);
  });

  it('FILE_HASH_ALGORITHM = SHA-256', () => {
    expect(FILE_HASH_ALGORITHM).toBe('SHA-256');
  });

  it('FILE_HASH_LENGTH = 32', () => {
    expect(FILE_HASH_LENGTH).toBe(32);
  });

  it('BOLT_VERSION = 1', () => {
    expect(BOLT_VERSION).toBe(1);
  });

  it('CAPABILITY_NAMESPACE = bolt.', () => {
    expect(CAPABILITY_NAMESPACE).toBe('bolt.');
  });

  it('total §14 constant count = 13', () => {
    const constants = [
      NONCE_LENGTH, PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH,
      DEFAULT_CHUNK_SIZE, TRANSFER_ID_LENGTH, PEER_CODE_LENGTH,
      PEER_CODE_ALPHABET, SAS_LENGTH, SAS_ENTROPY,
      FILE_HASH_ALGORITHM, FILE_HASH_LENGTH, BOLT_VERSION,
      CAPABILITY_NAMESPACE,
    ];
    expect(constants).toHaveLength(13);
  });
});
