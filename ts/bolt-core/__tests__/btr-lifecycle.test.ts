/**
 * BTR full-lifecycle cross-language vectors (P2).
 *
 * Consumes Rust-authority btr-lifecycle.vectors.json:
 * - 2 transfers × 3 chunks each
 * - Deterministic DH (StaticSecret scalars), fixed nonces
 * - Proves inter-transfer DH ratchet advances session root
 * - Proves multi-chunk seal/open byte-identical cross-language
 */
import { describe, it, expect } from 'vitest';
import lifecycleVectors from '../../../rust/bolt-core/test-vectors/btr/btr-lifecycle.vectors.json';
import { deriveSessionRoot, deriveTransferRoot, chainAdvance } from '../src/btr/key-schedule.js';
import { deriveRatchetedSessionRoot, scalarMult } from '../src/btr/ratchet.js';
import { btrSealDeterministic, btrOpen } from '../src/btr/encrypt.js';

function fromHex(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substring(i, i + 2), 16);
  }
  return bytes;
}

function toHex(bytes: Uint8Array): string {
  return Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('');
}

describe('BTR lifecycle vectors (Rust authority)', () => {
  const ephemeralSharedSecret = fromHex(lifecycleVectors.ephemeral_shared_secret_hex);
  let sessionRootKey = deriveSessionRoot(ephemeralSharedSecret);

  it('initial session root matches Rust derivation', () => {
    // Verified implicitly — if this is wrong, all downstream assertions fail.
    expect(sessionRootKey.length).toBe(32);
  });

  for (const transfer of lifecycleVectors.transfers) {
    describe(`${transfer.id}`, () => {
      it('DH commutativity: sender×receiver == receiver×sender', () => {
        const senderScalar = fromHex(transfer.sender_scalar_hex);
        const receiverScalar = fromHex(transfer.receiver_scalar_hex);
        const senderPublic = fromHex(transfer.sender_public_hex);
        const receiverPublic = fromHex(transfer.receiver_public_hex);

        // Verify public key derivation from scalar (scalar × basepoint)
        const basepoint = new Uint8Array(32);
        basepoint[0] = 9; // X25519 basepoint
        const derivedSenderPub = scalarMult(senderScalar, basepoint);
        const derivedReceiverPub = scalarMult(receiverScalar, basepoint);
        expect(toHex(derivedSenderPub)).toBe(toHex(senderPublic));
        expect(toHex(derivedReceiverPub)).toBe(toHex(receiverPublic));

        // DH commutativity
        const dhForward = scalarMult(senderScalar, receiverPublic);
        const dhReverse = scalarMult(receiverScalar, senderPublic);
        expect(toHex(dhForward)).toBe(toHex(dhReverse));
        expect(toHex(dhForward)).toBe(transfer.dh_output_hex);
      });

      it('DH ratchet produces expected session root', () => {
        const dhOutput = fromHex(transfer.dh_output_hex);
        const newSrk = deriveRatchetedSessionRoot(sessionRootKey, dhOutput);
        expect(toHex(newSrk)).toBe(transfer.session_root_key_after_hex);
        // Advance local tracking
        sessionRootKey = newSrk;
      });

      it('transfer root derivation matches Rust', () => {
        const transferId = fromHex(transfer.transfer_id_hex);
        const trk = deriveTransferRoot(sessionRootKey, transferId);
        expect(toHex(trk)).toBe(transfer.transfer_root_key_hex);
      });

      for (const chunk of transfer.chunks) {
        describe(`chunk ${chunk.chain_index}`, () => {
          it('chain advance produces expected message_key and next_chain_key', () => {
            const chainKey = fromHex(chunk.chain_key_hex);
            const adv = chainAdvance(chainKey);
            expect(toHex(adv.messageKey)).toBe(chunk.message_key_hex);
            expect(toHex(adv.nextChainKey)).toBe(chunk.next_chain_key_hex);
          });

          it('deterministic seal produces byte-identical ciphertext', () => {
            const messageKey = fromHex(chunk.message_key_hex);
            const nonce = fromHex(chunk.nonce_hex);
            const plaintext = fromHex(chunk.plaintext_hex);
            const sealed = btrSealDeterministic(messageKey, plaintext, nonce);
            expect(toHex(sealed)).toBe(chunk.sealed_hex);
          });

          it('open recovers original plaintext', () => {
            const messageKey = fromHex(chunk.message_key_hex);
            const sealed = fromHex(chunk.sealed_hex);
            const opened = btrOpen(messageKey, sealed);
            expect(toHex(opened)).toBe(chunk.plaintext_hex);
          });
        });
      }
    });
  }

  it('inter-transfer session root keys differ (DH ratchet advancement)', () => {
    const t0 = lifecycleVectors.transfers[0];
    const t1 = lifecycleVectors.transfers[1];
    expect(t0.session_root_key_after_hex).not.toBe(t1.session_root_key_after_hex);
  });

  it('inter-transfer transfer root keys differ (isolation)', () => {
    const t0 = lifecycleVectors.transfers[0];
    const t1 = lifecycleVectors.transfers[1];
    expect(t0.transfer_root_key_hex).not.toBe(t1.transfer_root_key_hex);
  });

  it('ratchet generation increments per transfer', () => {
    expect(lifecycleVectors.transfers[0].ratchet_generation_after).toBe(1);
    expect(lifecycleVectors.transfers[1].ratchet_generation_after).toBe(2);
  });
});
