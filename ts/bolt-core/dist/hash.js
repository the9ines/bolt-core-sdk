/**
 * Compute SHA-256 hash of data.
 */
export async function sha256(data) {
    if (data instanceof Uint8Array) {
        return await crypto.subtle.digest('SHA-256', data);
    }
    return await crypto.subtle.digest('SHA-256', data);
}
/**
 * Convert ArrayBuffer to hex string.
 */
export function bufferToHex(buffer) {
    return Array.from(new Uint8Array(buffer))
        .map(b => b.toString(16).padStart(2, '0'))
        .join('');
}
/**
 * Compute SHA-256 hash of a File or Blob and return hex string.
 */
export async function hashFile(file) {
    const buffer = await file.arrayBuffer();
    const hash = await sha256(buffer);
    return bufferToHex(hash);
}
