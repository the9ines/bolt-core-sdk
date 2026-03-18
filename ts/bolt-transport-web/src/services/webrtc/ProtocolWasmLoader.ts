/**
 * Protocol WASM loader — loads bolt-protocol-wasm from embedded artifact
 * and initializes bolt-core's WASM crypto adapter.
 *
 * BR2: This is the production entry point for WASM protocol authority.
 * Consumer apps should call initProtocolWasm() at startup instead of
 * bolt-core's initWasmCrypto() directly.
 *
 * Usage:
 *   import { initProtocolWasm } from '@the9ines/bolt-transport-web';
 *   await initProtocolWasm();  // call once before any crypto operations
 */

import { initWasmCryptoFromModule } from '@the9ines/bolt-core';

let _initAttempted = false;
let _initResult = false;

/**
 * Load and initialize the embedded protocol WASM module.
 *
 * Loads bolt_protocol_wasm from the wasm/ directory embedded in this package,
 * then passes the loaded module to bolt-core's initWasmCryptoFromModule().
 *
 * Falls back silently — returns false if WASM is not available.
 * PM-RB-03: TS fallback remains operational if this fails.
 */
export async function initProtocolWasm(): Promise<boolean> {
  if (_initAttempted) return _initResult;
  _initAttempted = true;

  try {
    // Relative import — Vite/webpack resolve this from the published package.
    // Same pattern as PolicyAdapter.ts loading policy WASM.
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const wasm = await import('../../../wasm/bolt_protocol_wasm.js' as any);
    await wasm.default();

    _initResult = initWasmCryptoFromModule(wasm);
    return _initResult;
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    console.warn(`[BOLT-WASM] Protocol WASM load failed (${msg}), using TS fallback`);
    _initResult = false;
    return false;
  }
}
