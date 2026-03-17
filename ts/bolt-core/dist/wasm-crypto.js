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
 * Initialize WASM crypto. Call once at app startup.
 * Fails silently — returns false if WASM is not available.
 * PM-RB-03: TS fallback remains operational if this fails.
 */
export async function initWasmCrypto(wasmUrl) {
    if (_initAttempted)
        return _wasmCrypto !== null;
    _initAttempted = true;
    try {
        // Dynamic import — the WASM module is loaded at runtime, not bundle time.
        // The consuming app must make the bolt-protocol-wasm package or .wasm file available.
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const wasm = await (Function('return import("bolt-protocol-wasm")')());
        await wasm.default(wasmUrl);
        _wasmModule = wasm; // RB4: store for BTR/transfer handle construction
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
        console.log('[BOLT-WASM] Protocol authority initialized (Rust/WASM: crypto + BTR + transfer)');
        return true;
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
