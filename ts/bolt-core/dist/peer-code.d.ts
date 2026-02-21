/**
 * Generate a cryptographically secure 6-character peer code.
 * Uses crypto.getRandomValues() for secure random generation.
 */
export declare function generateSecurePeerCode(): string;
/**
 * Generate a longer peer code with dash separator.
 * Format: XXXX-XXXX (~40 bits of entropy)
 */
export declare function generateLongPeerCode(): string;
/**
 * Validate peer code format.
 * Accepts 6-char or 8-char (with optional dash) codes using the unambiguous alphabet.
 */
export declare function isValidPeerCode(code: string): boolean;
/**
 * Normalize peer code for comparison (remove dashes, uppercase).
 */
export declare function normalizePeerCode(code: string): string;
