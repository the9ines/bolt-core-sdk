/**
 * Compute a 6-character SAS (Short Authentication String) per PROTOCOL.md.
 *
 * SAS_input = SHA-256( sort32(identityA, identityB) || sort32(ephemeralA, ephemeralB) )
 * Display first 6 hex chars uppercase.
 *
 * @param identityA - Raw 32-byte identity public key of peer A
 * @param identityB - Raw 32-byte identity public key of peer B
 * @param ephemeralA - Raw 32-byte ephemeral public key of peer A
 * @param ephemeralB - Raw 32-byte ephemeral public key of peer B
 * @returns 6-character uppercase hex string (24 bits of entropy)
 */
export declare function computeSas(identityA: Uint8Array, identityB: Uint8Array, ephemeralA: Uint8Array, ephemeralB: Uint8Array): Promise<string>;
