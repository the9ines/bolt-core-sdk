import { PEER_CODE_ALPHABET } from './constants.js';
/**
 * Generate a cryptographically secure 6-character peer code.
 * Uses crypto.getRandomValues() for secure random generation.
 */
export function generateSecurePeerCode() {
    const array = new Uint8Array(6);
    crypto.getRandomValues(array);
    let code = '';
    for (let i = 0; i < 6; i++) {
        code += PEER_CODE_ALPHABET[array[i] % PEER_CODE_ALPHABET.length];
    }
    return code;
}
/**
 * Generate a longer peer code with dash separator.
 * Format: XXXX-XXXX (~40 bits of entropy)
 */
export function generateLongPeerCode() {
    const array = new Uint8Array(8);
    crypto.getRandomValues(array);
    let code = '';
    for (let i = 0; i < 8; i++) {
        code += PEER_CODE_ALPHABET[array[i] % PEER_CODE_ALPHABET.length];
        if (i === 3)
            code += '-';
    }
    return code;
}
/**
 * Validate peer code format.
 * Accepts 6-char or 8-char (with optional dash) codes using the unambiguous alphabet.
 */
export function isValidPeerCode(code) {
    const normalized = code.replace(/-/g, '').toUpperCase();
    if (normalized.length !== 6 && normalized.length !== 8) {
        return false;
    }
    return normalized.split('').every(char => PEER_CODE_ALPHABET.includes(char));
}
/**
 * Normalize peer code for comparison (remove dashes, uppercase).
 */
export function normalizePeerCode(code) {
    return code.replace(/-/g, '').toUpperCase();
}
