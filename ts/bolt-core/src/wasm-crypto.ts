/**
 * WASM-backed crypto/session adapter (RUSTIFY-BROWSER-CORE-1 RB3).
 *
 * Provides the same function signatures as the TS crypto/identity/sas modules,
 * but backed by Rust/WASM (bolt-protocol-wasm). The TS tweetnacl implementations
 * remain as fallback (PM-RB-03: condition-gated dual-path).
 *
 * Usage:
 *   import { initWasmCrypto, wasmCrypto } from './wasm-crypto.js';
 *   await initWasmCrypto();  // call once at app startup
 *   if (wasmCrypto) {
 *     const kp = wasmCrypto.generateEphemeralKeyPair();
 *   }
 */

// Type-only — matches TS crypto.ts signatures exactly
export interface WasmCryptoAdapter {
  generateEphemeralKeyPair(): { publicKey: Uint8Array; secretKey: Uint8Array };
  generateIdentityKeyPair(): { publicKey: Uint8Array; secretKey: Uint8Array };
  sealBoxPayload(plaintext: Uint8Array, remotePublicKey: Uint8Array, senderSecretKey: Uint8Array): string;
  openBoxPayload(sealed: string, senderPublicKey: Uint8Array, receiverSecretKey: Uint8Array): Uint8Array;
  computeSas(identityA: Uint8Array, identityB: Uint8Array, ephemeralA: Uint8Array, ephemeralB: Uint8Array): string;
  generateSecurePeerCode(): string;
  isValidPeerCode(code: string): boolean;
  sha256Hex(data: Uint8Array): string;
}

/** WASM adapter instance. null until initWasmCrypto() succeeds. */
let _wasmCrypto: WasmCryptoAdapter | null = null;

/** Whether WASM init has been attempted. */
let _initAttempted = false;

/**
 * Get the WASM crypto adapter, or null if not initialized or init failed.
 * Callers should fall back to TS crypto if this returns null.
 */
export function getWasmCrypto(): WasmCryptoAdapter | null {
  return _wasmCrypto;
}

/**
 * Initialize WASM crypto. Call once at app startup.
 * Fails silently — returns false if WASM is not available.
 * PM-RB-03: TS fallback remains operational if this fails.
 */
export async function initWasmCrypto(wasmUrl?: string): Promise<boolean> {
  if (_initAttempted) return _wasmCrypto !== null;
  _initAttempted = true;

  try {
    // Dynamic import — the WASM module is loaded at runtime, not bundle time.
    // The consuming app must make the bolt-protocol-wasm package or .wasm file available.
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const wasm: any = await (Function('return import("bolt-protocol-wasm")')());
    await wasm.default(wasmUrl);

    _wasmCrypto = {
      generateEphemeralKeyPair: () => wasm.generateEphemeralKeyPair(),
      generateIdentityKeyPair: () => wasm.generateIdentityKeyPair(),
      sealBoxPayload: (p, rpk, sk) => wasm.sealBoxPayload(p, rpk, sk),
      openBoxPayload: (s, spk, rsk) => wasm.openBoxPayload(s, spk, rsk),
      computeSas: (ia, ib, ea, eb) => wasm.computeSas(ia, ib, ea, eb),
      generateSecurePeerCode: () => wasm.generateSecurePeerCode(),
      isValidPeerCode: (c) => wasm.isValidPeerCode(c),
      sha256Hex: (d) => wasm.sha256Hex(d),
    };

    console.log('[BOLT-WASM] Protocol crypto initialized (Rust/WASM authority)');
    return true;
  } catch (e) {
    console.warn('[BOLT-WASM] Failed to initialize — falling back to TS crypto:', e);
    _wasmCrypto = null;
    return false;
  }
}
