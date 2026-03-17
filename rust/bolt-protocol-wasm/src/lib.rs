//! WASM bindings for Bolt protocol crypto/session authority.
//!
//! RUSTIFY-BROWSER-CORE-1 RB3: Exposes bolt-core crypto, session, SAS,
//! and peer-code functions for browser consumption via wasm-bindgen.
//!
//! These bindings wrap the canonical Rust implementations. The browser
//! path calls these instead of the TS tweetnacl/noble-hashes equivalents.
//!
//! Scope: crypto + session only. BTR + transfer SM deferred to RB4.

use wasm_bindgen::prelude::*;

// ── Initialization ────────────────────────────────────────────────

/// Initialize panic hook for browser console error reporting.
/// Called once at WASM module load time.
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

// ── Key Generation ────────────────────────────────────────────────

/// Generate an ephemeral X25519 keypair for session use.
///
/// Returns a JsValue containing { publicKey: Uint8Array, secretKey: Uint8Array }.
/// Parity: TS `generateEphemeralKeyPair()` (tweetnacl `box.keyPair()`).
#[wasm_bindgen(js_name = "generateEphemeralKeyPair")]
pub fn generate_ephemeral_keypair() -> Result<JsValue, JsValue> {
    let kp = bolt_core::crypto::generate_ephemeral_keypair();
    let obj = js_sys::Object::new();
    let pk = js_sys::Uint8Array::from(&kp.public_key[..]);
    let sk = js_sys::Uint8Array::from(&kp.secret_key[..]);
    js_sys::Reflect::set(&obj, &"publicKey".into(), &pk)?;
    js_sys::Reflect::set(&obj, &"secretKey".into(), &sk)?;
    Ok(obj.into())
}

/// Generate a persistent identity X25519 keypair.
///
/// Returns a JsValue containing { publicKey: Uint8Array, secretKey: Uint8Array }.
/// Parity: TS `generateIdentityKeyPair()`.
#[wasm_bindgen(js_name = "generateIdentityKeyPair")]
pub fn generate_identity_keypair() -> Result<JsValue, JsValue> {
    let kp = bolt_core::identity::generate_identity_keypair();
    let obj = js_sys::Object::new();
    let pk = js_sys::Uint8Array::from(&kp.public_key[..]);
    let sk = js_sys::Uint8Array::from(&kp.secret_key[..]);
    js_sys::Reflect::set(&obj, &"publicKey".into(), &pk)?;
    js_sys::Reflect::set(&obj, &"secretKey".into(), &sk)?;
    Ok(obj.into())
}

// ── NaCl Box (XSalsa20-Poly1305) ─────────────────────────────────

/// Seal plaintext using NaCl box. Returns base64(nonce || ciphertext).
///
/// Parity: TS `sealBoxPayload(plaintext, remotePublicKey, senderSecretKey)`.
/// Identical wire format. Random nonce generated internally via CSPRNG.
#[wasm_bindgen(js_name = "sealBoxPayload")]
pub fn seal_box_payload(
    plaintext: &[u8],
    remote_public_key: &[u8],
    sender_secret_key: &[u8],
) -> Result<String, JsValue> {
    let rpk: [u8; 32] = remote_public_key
        .try_into()
        .map_err(|_| JsValue::from_str("remote_public_key must be 32 bytes"))?;
    let ssk: [u8; 32] = sender_secret_key
        .try_into()
        .map_err(|_| JsValue::from_str("sender_secret_key must be 32 bytes"))?;

    bolt_core::crypto::seal_box_payload(plaintext, &rpk, &ssk)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Open a sealed NaCl box payload. Expects base64(nonce || ciphertext).
///
/// Parity: TS `openBoxPayload(sealed, senderPublicKey, receiverSecretKey)`.
#[wasm_bindgen(js_name = "openBoxPayload")]
pub fn open_box_payload(
    sealed: &str,
    sender_public_key: &[u8],
    receiver_secret_key: &[u8],
) -> Result<Vec<u8>, JsValue> {
    let spk: [u8; 32] = sender_public_key
        .try_into()
        .map_err(|_| JsValue::from_str("sender_public_key must be 32 bytes"))?;
    let rsk: [u8; 32] = receiver_secret_key
        .try_into()
        .map_err(|_| JsValue::from_str("receiver_secret_key must be 32 bytes"))?;

    bolt_core::crypto::open_box_payload(sealed, &spk, &rsk)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

// ── SAS (Short Authentication String) ─────────────────────────────

/// Compute 6-character SAS from identity + ephemeral public keys.
///
/// Parity: TS `computeSas(identityA, identityB, ephemeralA, ephemeralB)`.
/// Identical algorithm: SHA-256(sort32(id_a, id_b) || sort32(eph_a, eph_b)),
/// first 6 hex chars, uppercase.
#[wasm_bindgen(js_name = "computeSas")]
pub fn compute_sas(
    identity_a: &[u8],
    identity_b: &[u8],
    ephemeral_a: &[u8],
    ephemeral_b: &[u8],
) -> Result<String, JsValue> {
    let id_a: [u8; 32] = identity_a
        .try_into()
        .map_err(|_| JsValue::from_str("identity_a must be 32 bytes"))?;
    let id_b: [u8; 32] = identity_b
        .try_into()
        .map_err(|_| JsValue::from_str("identity_b must be 32 bytes"))?;
    let eph_a: [u8; 32] = ephemeral_a
        .try_into()
        .map_err(|_| JsValue::from_str("ephemeral_a must be 32 bytes"))?;
    let eph_b: [u8; 32] = ephemeral_b
        .try_into()
        .map_err(|_| JsValue::from_str("ephemeral_b must be 32 bytes"))?;

    Ok(bolt_core::sas::compute_sas(&id_a, &id_b, &eph_a, &eph_b))
}

// ── Peer Code ─────────────────────────────────────────────────────

/// Generate a 6-character secure peer code.
///
/// Parity: TS `generateSecurePeerCode()`.
#[wasm_bindgen(js_name = "generateSecurePeerCode")]
pub fn generate_peer_code() -> String {
    bolt_core::peer_code::generate_secure_peer_code()
}

/// Validate a peer code (6 or 8 chars, optional hyphens, unambiguous alphabet).
///
/// Parity: TS `isValidPeerCode(code)`.
#[wasm_bindgen(js_name = "isValidPeerCode")]
pub fn is_valid_peer_code(code: &str) -> bool {
    bolt_core::peer_code::is_valid_peer_code(code)
}

// ── Hashing ───────────────────────────────────────────────────────

/// Compute SHA-256 hex digest of data.
///
/// Parity: TS `sha256Hex(data)`.
#[wasm_bindgen(js_name = "sha256Hex")]
pub fn sha256_hex(data: &[u8]) -> String {
    bolt_core::hash::sha256_hex(data)
}

// ── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keypair_roundtrip() {
        let kp = bolt_core::crypto::generate_ephemeral_keypair();
        let plaintext = b"RB3 test";
        let sealed = bolt_core::crypto::seal_box_payload(
            plaintext,
            &kp.public_key,
            &kp.secret_key,
        )
        .unwrap();
        let opened = bolt_core::crypto::open_box_payload(
            &sealed,
            &kp.public_key,
            &kp.secret_key,
        )
        .unwrap();
        assert_eq!(opened, plaintext);
    }

    #[test]
    fn sas_deterministic() {
        let a = [1u8; 32];
        let b = [2u8; 32];
        let sas1 = bolt_core::sas::compute_sas(&a, &b, &a, &b);
        let sas2 = bolt_core::sas::compute_sas(&a, &b, &a, &b);
        assert_eq!(sas1, sas2);
        assert_eq!(sas1.len(), 6);
    }

    #[test]
    fn peer_code_valid() {
        let code = bolt_core::peer_code::generate_secure_peer_code();
        assert!(bolt_core::peer_code::is_valid_peer_code(&code));
        assert!(!bolt_core::peer_code::is_valid_peer_code(""));
    }

    #[test]
    fn sha256_known() {
        let hash = bolt_core::hash::sha256_hex(b"hello");
        assert_eq!(hash.len(), 64);
        assert!(hash.starts_with("2cf24dba"));
    }

    #[test]
    fn golden_vector_hello_bolt() {
        let sender_pk = bolt_core::encoding::from_hex(
            "07a37cbc142093c8b755dc1b10e86cb426374ad16aa853ed0bdfc0b2b86d1c7c",
        )
        .unwrap();
        let sender_pk: [u8; 32] = sender_pk.try_into().unwrap();
        let receiver_sk: [u8; 32] = core::array::from_fn(|i| (i as u8) + 33);
        let sealed = "AAECAwQFBgcICQoLDA0ODxAREhMUFRYXvjBFLvx0BRI+SIiwhwJMy1qQtzU1EV2Qlp41Ig==";
        let result = bolt_core::crypto::open_box_payload(sealed, &sender_pk, &receiver_sk).unwrap();
        assert_eq!(result, b"Hello, Bolt!");
    }
}
