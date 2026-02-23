import { toBase64, fromBase64, KeyMismatchError } from '@the9ines/bolt-core';

// ─── Pin Persistence Interface ──────────────────────────────────────────────

/** Abstract storage backend for TOFU peer pins. */
export interface PinPersistence {
  getPin(peerCode: string): Promise<Uint8Array | null>;
  setPin(peerCode: string, identityPublicKey: Uint8Array): Promise<void>;
  removePin(peerCode: string): Promise<void>;
}

// ─── IndexedDB Implementation ───────────────────────────────────────────────

const DB_NAME = 'bolt-pins';
const DB_VERSION = 1;
const STORE_NAME = 'pins';

function openPinDB(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, DB_VERSION);
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME);
      }
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

/** IndexedDB-backed pin persistence. */
export class IndexedDBPinStore implements PinPersistence {
  async getPin(peerCode: string): Promise<Uint8Array | null> {
    const db = await openPinDB();
    return new Promise((resolve, reject) => {
      const tx = db.transaction(STORE_NAME, 'readonly');
      const store = tx.objectStore(STORE_NAME);
      const req = store.get(peerCode);
      req.onsuccess = () => {
        const val = req.result;
        resolve(val ? fromBase64(val) : null);
      };
      req.onerror = () => reject(req.error);
    });
  }

  async setPin(peerCode: string, identityPublicKey: Uint8Array): Promise<void> {
    const db = await openPinDB();
    return new Promise((resolve, reject) => {
      const tx = db.transaction(STORE_NAME, 'readwrite');
      const store = tx.objectStore(STORE_NAME);
      store.put(toBase64(identityPublicKey), peerCode);
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error);
    });
  }

  async removePin(peerCode: string): Promise<void> {
    const db = await openPinDB();
    return new Promise((resolve, reject) => {
      const tx = db.transaction(STORE_NAME, 'readwrite');
      const store = tx.objectStore(STORE_NAME);
      store.delete(peerCode);
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error);
    });
  }
}

// ─── In-Memory Implementation (for tests) ───────────────────────────────────

/** In-memory pin persistence for testing. */
export class MemoryPinStore implements PinPersistence {
  private pins = new Map<string, Uint8Array>();

  async getPin(peerCode: string): Promise<Uint8Array | null> {
    return this.pins.get(peerCode) ?? null;
  }

  async setPin(peerCode: string, identityPublicKey: Uint8Array): Promise<void> {
    this.pins.set(peerCode, identityPublicKey);
  }

  async removePin(peerCode: string): Promise<void> {
    this.pins.delete(peerCode);
  }
}

// ─── Verification ───────────────────────────────────────────────────────────

function uint8Equal(a: Uint8Array, b: Uint8Array): boolean {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) return false;
  }
  return true;
}

/**
 * Verify a peer's identity against the pin store.
 *
 * - If no pin exists for this peer: pin the identity (first contact / TOFU)
 * - If pin matches: return normally (trusted)
 * - If pin mismatches: throw KeyMismatchError (fail-closed)
 *
 * @returns 'pinned' if this was first contact, 'verified' if existing pin matched
 */
export async function verifyPinnedIdentity(
  pinStore: PinPersistence,
  peerCode: string,
  identityPublicKey: Uint8Array,
): Promise<'pinned' | 'verified'> {
  const existing = await pinStore.getPin(peerCode);

  if (!existing) {
    await pinStore.setPin(peerCode, identityPublicKey);
    return 'pinned';
  }

  if (uint8Equal(existing, identityPublicKey)) {
    return 'verified';
  }

  throw new KeyMismatchError(peerCode, existing, identityPublicKey);
}
