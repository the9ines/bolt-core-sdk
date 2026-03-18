/**
 * WASM-backed protocol adapter (RUSTIFY-BROWSER-CORE-1 RB3+RB4).
 *
 * RB3: crypto/session/SAS functions backed by Rust/WASM.
 * RB4: BTR state (BtrEngine, BtrTransferCtx) + transfer state (SendSession)
 *      backed by Rust/WASM opaque handles.
 *
 * TS tweetnacl/BTR implementations remain as fallback (PM-RB-03: condition-gated).
 */
/** WASM adapter instance. null until initWasmCrypto() succeeds. */
let _wasmCrypto = null;
/** Whether WASM init has been attempted. */
let _initAttempted = false;
/**
 * Get the WASM crypto adapter, or null if not initialized or init failed.
 * Callers should fall back to TS crypto if this returns null.
 */
export function getWasmCrypto() {
    return _wasmCrypto;
}
/**
 * Initialize WASM crypto from a pre-loaded WASM module.
 *
 * BR2: Accepts an already-loaded+initialized WASM module (provided by
 * transport-web's initProtocolWasm()). This avoids bare module specifier
 * issues — the loader lives in transport-web where the artifact is embedded.
 *
 * PM-RB-03: TS fallback remains operational if this is never called.
 *
 * @param wasmModule - The loaded bolt-protocol-wasm module (with exported functions)
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function initWasmCryptoFromModule(wasmModule) {
    if (_wasmCrypto)
        return true;
    try {
        _wasmModule = wasmModule;
        _wasmCrypto = {
            generateEphemeralKeyPair: () => wasmModule.generateEphemeralKeyPair(),
            generateIdentityKeyPair: () => wasmModule.generateIdentityKeyPair(),
            sealBoxPayload: (p, rpk, sk) => wasmModule.sealBoxPayload(p, rpk, sk),
            openBoxPayload: (s, spk, rsk) => wasmModule.openBoxPayload(s, spk, rsk),
            computeSas: (ia, ib, ea, eb) => wasmModule.computeSas(ia, ib, ea, eb),
            generateSecurePeerCode: () => wasmModule.generateSecurePeerCode(),
            isValidPeerCode: (c) => wasmModule.isValidPeerCode(c),
            sha256Hex: (d) => wasmModule.sha256Hex(d),
        };
        console.log('[BOLT-WASM] Protocol authority initialized (Rust/WASM: crypto + BTR + transfer)');
        return true;
    }
    catch (e) {
        console.warn('[BOLT-WASM] Failed to initialize from module:', e);
        _wasmCrypto = null;
        _wasmModule = null;
        return false;
    }
}
/**
 * Initialize WASM crypto. Legacy entry point — attempts dynamic import.
 * Prefer initProtocolWasm() from @the9ines/bolt-transport-web instead.
 */
export async function initWasmCrypto() {
    if (_initAttempted)
        return _wasmCrypto !== null;
    _initAttempted = true;
    try {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const wasm = await (Function('return import("bolt-protocol-wasm")')());
        await wasm.default();
        return initWasmCryptoFromModule(wasm);
    }
    catch (e) {
        console.warn('[BOLT-WASM] Failed to initialize — falling back to TS protocol:', e);
        _wasmCrypto = null;
        return false;
    }
}
// ══════════════════════════════════════════════════════════════════
// RB4: BTR + Transfer State Authority
// ══════════════════════════════════════════════════════════════════
/** WASM module reference for constructing opaque handles. */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let _wasmModule = null;
/**
 * Get the raw WASM module for constructing BTR/transfer handles.
 * Returns null if WASM not initialized.
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function getWasmModule() {
    return _wasmModule;
}
/**
 * Create a WASM-backed BTR engine. Returns null if WASM not available.
 * Caller must call .free() when done to zeroize key material.
 */
export function createWasmBtrEngine(sharedSecret) {
    if (!_wasmModule)
        return null;
    try {
        return new _wasmModule.WasmBtrEngine(sharedSecret);
    }
    catch {
        return null;
    }
}
/**
 * Create a WASM-backed send session. Returns null if WASM not available.
 * Rust owns transfer-state transitions. TS proposes events; Rust validates.
 */
export function createWasmSendSession() {
    if (!_wasmModule)
        return null;
    try {
        return new _wasmModule.WasmSendSession();
    }
    catch {
        return null;
    }
}
