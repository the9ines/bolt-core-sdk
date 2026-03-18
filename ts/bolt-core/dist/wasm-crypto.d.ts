/**
 * WASM-backed protocol adapter (RUSTIFY-BROWSER-CORE-1 RB3+RB4).
 *
 * RB3: crypto/session/SAS functions backed by Rust/WASM.
 * RB4: BTR state (BtrEngine, BtrTransferCtx) + transfer state (SendSession)
 *      backed by Rust/WASM opaque handles.
 *
 * TS tweetnacl/BTR implementations remain as fallback (PM-RB-03: condition-gated).
 */
export interface WasmCryptoAdapter {
    generateEphemeralKeyPair(): {
        publicKey: Uint8Array;
        secretKey: Uint8Array;
    };
    generateIdentityKeyPair(): {
        publicKey: Uint8Array;
        secretKey: Uint8Array;
    };
    sealBoxPayload(plaintext: Uint8Array, remotePublicKey: Uint8Array, senderSecretKey: Uint8Array): string;
    openBoxPayload(sealed: string, senderPublicKey: Uint8Array, receiverSecretKey: Uint8Array): Uint8Array;
    computeSas(identityA: Uint8Array, identityB: Uint8Array, ephemeralA: Uint8Array, ephemeralB: Uint8Array): string;
    generateSecurePeerCode(): string;
    isValidPeerCode(code: string): boolean;
    sha256Hex(data: Uint8Array): string;
}
/**
 * Get the WASM crypto adapter, or null if not initialized or init failed.
 * Callers should fall back to TS crypto if this returns null.
 */
export declare function getWasmCrypto(): WasmCryptoAdapter | null;
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
export declare function initWasmCryptoFromModule(wasmModule: any): boolean;
/**
 * Initialize WASM crypto. Legacy entry point — attempts dynamic import.
 * Prefer initProtocolWasm() from @the9ines/bolt-transport-web instead.
 */
export declare function initWasmCrypto(): Promise<boolean>;
/**
 * Get the raw WASM module for constructing BTR/transfer handles.
 * Returns null if WASM not initialized.
 */
export declare function getWasmModule(): any;
/**
 * Opaque BTR engine handle. Rust owns all key material.
 * Wraps WasmBtrEngine from bolt-protocol-wasm.
 *
 * Usage:
 *   const engine = createWasmBtrEngine(sharedSecret);
 *   const ctx = engine.beginTransferSend(transferId, remoteRatchetPub);
 *   const { chainIndex, sealed } = ctx.sealChunk(plaintext);
 *   engine.free();  // zeroize when done
 */
export interface WasmBtrEngineHandle {
    beginTransferSend(transferId: Uint8Array, remoteRatchetPub: Uint8Array): WasmBtrTransferCtxHandle;
    beginTransferReceive(transferId: Uint8Array, remoteRatchetPub: Uint8Array): WasmBtrTransferCtxHandle;
    ratchetGeneration(): number;
    endTransfer(): void;
    cleanupDisconnect(): void;
    free(): void;
}
export interface WasmBtrTransferCtxHandle {
    sealChunk(plaintext: Uint8Array): {
        chainIndex: number;
        sealed: Uint8Array;
    };
    openChunk(expectedIndex: number, sealed: Uint8Array): Uint8Array;
    chainIndex(): number;
    generation(): number;
    transferId(): Uint8Array;
    localRatchetPub(): Uint8Array;
    cleanupComplete(): void;
    cleanupCancel(): void;
    free(): void;
}
export interface WasmSendSessionHandle {
    beginSend(transferId: string, payload: Uint8Array, filename: string, fileHash?: string): {
        transferId: string;
        filename: string;
        size: number;
        totalChunks: number;
        chunkSize: number;
        fileHash?: string;
    };
    onAccept(transferId: string): void;
    onCancel(transferId: string): void;
    onPause(transferId: string): void;
    onResume(transferId: string): void;
    nextChunk(): {
        transferId: string;
        chunkIndex: number;
        totalChunks: number;
        data: Uint8Array;
    } | null;
    finish(): string;
    state(): string;
    isSendActive(): boolean;
    free(): void;
}
/**
 * Create a WASM-backed BTR engine. Returns null if WASM not available.
 * Caller must call .free() when done to zeroize key material.
 */
export declare function createWasmBtrEngine(sharedSecret: Uint8Array): WasmBtrEngineHandle | null;
/**
 * Create a WASM-backed send session. Returns null if WASM not available.
 * Rust owns transfer-state transitions. TS proposes events; Rust validates.
 */
export declare function createWasmSendSession(): WasmSendSessionHandle | null;
