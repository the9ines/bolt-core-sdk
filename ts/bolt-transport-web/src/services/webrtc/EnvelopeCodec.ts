/**
 * EnvelopeCodec — stateless Profile Envelope v1 encode/decode + dcSendMessage.
 *
 * Extracted from WebRTCService (A2). Thin wrapper around existing
 * seal/open primitives — no new crypto, no behavior change.
 *
 * BTR-4: Extended with optional BTR envelope-level fields (§16.2).
 */
import { sealBoxPayload, openBoxPayload } from '@the9ines/bolt-core';
import type { ProfileEnvelopeV1 } from './types.js';
import type { BtrEnvelopeFields } from './BtrTransferAdapter.js';

/** Encrypt an inner message into a ProfileEnvelopeV1 wire object. */
export function encodeProfileEnvelopeV1(
  innerMsg: object,
  remotePublicKey: Uint8Array,
  secretKey: Uint8Array,
  btrFields?: BtrEnvelopeFields,
): ProfileEnvelopeV1 {
  const innerJson = JSON.stringify(innerMsg);
  const innerBytes = new TextEncoder().encode(innerJson);
  const payload = sealBoxPayload(innerBytes, remotePublicKey, secretKey);
  const envelope: ProfileEnvelopeV1 = { type: 'profile-envelope', version: 1, encoding: 'base64', payload };
  if (btrFields) {
    envelope.chain_index = btrFields.chain_index;
    if (btrFields.ratchet_public_key !== undefined) {
      envelope.ratchet_public_key = btrFields.ratchet_public_key;
    }
    if (btrFields.ratchet_generation !== undefined) {
      envelope.ratchet_generation = btrFields.ratchet_generation;
    }
  }
  return envelope;
}

/** Decrypt a ProfileEnvelopeV1 payload. Throws on failure. */
export function decodeProfileEnvelopeV1(
  payload: string,
  remotePublicKey: Uint8Array,
  secretKey: Uint8Array,
): Uint8Array {
  return openBoxPayload(payload, remotePublicKey, secretKey);
}

/**
 * Extract BTR fields from a profile-envelope if present.
 * Returns null if no BTR fields are present.
 */
export function extractBtrEnvelopeFields(msg: any): BtrEnvelopeFields | null {
  if (typeof msg.chain_index !== 'number') return null;
  const fields: BtrEnvelopeFields = { chain_index: msg.chain_index };
  if (typeof msg.ratchet_public_key === 'string') {
    fields.ratchet_public_key = msg.ratchet_public_key;
  }
  if (typeof msg.ratchet_generation === 'number') {
    fields.ratchet_generation = msg.ratchet_generation;
  }
  return fields;
}

/**
 * Send a message over the DataChannel, wrapping in profile-envelope when negotiated.
 * MUST only be called after helloComplete === true (except for pre-handshake error messages).
 */
export function dcSendMessage(
  dc: RTCDataChannel | null,
  innerMsg: any,
  negotiatedEnvelope: boolean,
  helloComplete: boolean,
  keyPair: { publicKey: Uint8Array; secretKey: Uint8Array } | null,
  remotePublicKey: Uint8Array | null,
  btrFields?: BtrEnvelopeFields,
): void {
  if (!dc || dc.readyState !== 'open') return;
  if (negotiatedEnvelope && helloComplete && keyPair && remotePublicKey) {
    dc.send(JSON.stringify(encodeProfileEnvelopeV1(innerMsg, remotePublicKey, keyPair.secretKey, btrFields)));
  } else {
    dc.send(JSON.stringify(innerMsg));
  }
}
