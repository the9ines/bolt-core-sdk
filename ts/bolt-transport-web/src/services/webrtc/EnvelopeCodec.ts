/**
 * EnvelopeCodec — stateless Profile Envelope v1 encode/decode + dcSendMessage.
 *
 * Extracted from WebRTCService (A2). Thin wrapper around existing
 * seal/open primitives — no new crypto, no behavior change.
 */
import { sealBoxPayload, openBoxPayload } from '@the9ines/bolt-core';
import type { ProfileEnvelopeV1 } from './types.js';

/** Encrypt an inner message into a ProfileEnvelopeV1 wire object. */
export function encodeProfileEnvelopeV1(
  innerMsg: object,
  remotePublicKey: Uint8Array,
  secretKey: Uint8Array,
): ProfileEnvelopeV1 {
  const innerJson = JSON.stringify(innerMsg);
  const innerBytes = new TextEncoder().encode(innerJson);
  const payload = sealBoxPayload(innerBytes, remotePublicKey, secretKey);
  return { type: 'profile-envelope', version: 1, encoding: 'base64', payload };
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
): void {
  if (!dc || dc.readyState !== 'open') return;
  if (negotiatedEnvelope && helloComplete && keyPair && remotePublicKey) {
    dc.send(JSON.stringify(encodeProfileEnvelopeV1(innerMsg, remotePublicKey, keyPair.secretKey)));
  } else {
    dc.send(JSON.stringify(innerMsg));
  }
}
