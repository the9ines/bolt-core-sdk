import { encodeBase64, decodeBase64 } from 'tweetnacl-util';
/** Encode a Uint8Array to a base64 string */
export function toBase64(data) {
    return encodeBase64(data);
}
/** Decode a base64 string to a Uint8Array */
export function fromBase64(base64) {
    return decodeBase64(base64);
}
