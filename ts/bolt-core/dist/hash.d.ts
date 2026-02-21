/**
 * Compute SHA-256 hash of data.
 */
export declare function sha256(data: ArrayBuffer | Uint8Array): Promise<ArrayBuffer>;
/**
 * Convert ArrayBuffer to hex string.
 */
export declare function bufferToHex(buffer: ArrayBuffer): string;
/**
 * Compute SHA-256 hash of a File and return hex string.
 */
export declare function hashFile(file: File): Promise<string>;
