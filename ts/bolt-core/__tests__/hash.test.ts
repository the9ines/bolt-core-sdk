import { describe, it, expect } from 'vitest';
import { sha256, bufferToHex } from '../src/index.js';

describe('bufferToHex', () => {
  it('converts empty buffer', () => {
    expect(bufferToHex(new Uint8Array([]).buffer)).toBe('');
  });

  it('converts known bytes', () => {
    expect(bufferToHex(new Uint8Array([0, 1, 255]).buffer)).toBe('0001ff');
  });

  it('pads single-digit hex values', () => {
    expect(bufferToHex(new Uint8Array([0x0a]).buffer)).toBe('0a');
  });
});

describe('sha256', () => {
  it('produces a 32-byte hash', async () => {
    const data = new TextEncoder().encode('hello');
    const hash = await sha256(data);
    expect(new Uint8Array(hash)).toHaveLength(32);
  });

  it('produces consistent output', async () => {
    const data = new TextEncoder().encode('test');
    const hash1 = bufferToHex(await sha256(data));
    const hash2 = bufferToHex(await sha256(data));
    expect(hash1).toBe(hash2);
  });

  it('produces known hash for empty input', async () => {
    const hash = bufferToHex(await sha256(new Uint8Array([])));
    expect(hash).toBe('e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855');
  });
});
