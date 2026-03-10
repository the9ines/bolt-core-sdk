//! BTR golden vector generator — Rust authority for cross-language parity.
//!
//! Generates deterministic JSON fixtures for 8 BTR vector categories.
//! Output path: `bolt-core-sdk/rust/bolt-core/test-vectors/btr/`
//! TS parity consumption: `bolt-core-sdk/ts/bolt-core/__tests__/vectors/btr/`
//!
//! TEST FIXTURES ONLY — all keys are deterministic and publicly known.

use serde::Serialize;

use crate::key_schedule::{chain_advance, derive_session_root, derive_transfer_root};
use crate::negotiate::{negotiate_btr, BtrMode};
use crate::ratchet::derive_ratcheted_session_root;

use crypto_secretbox::aead::Aead;
use crypto_secretbox::{KeyInit, Nonce, XSalsa20Poly1305};
use x25519_dalek::{PublicKey, StaticSecret};

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn make_key(seed: u8) -> [u8; 32] {
    let mut k = [0u8; 32];
    for (i, b) in k.iter_mut().enumerate() {
        *b = (i as u8).wrapping_add(seed);
    }
    k
}

fn make_tid(seed: u8) -> [u8; 16] {
    let mut t = [0u8; 16];
    for (i, b) in t.iter_mut().enumerate() {
        *b = (i as u8).wrapping_add(seed);
    }
    t
}

// ── btr-key-schedule ──────────────────────────────────────────────────

#[derive(Serialize)]
struct KeyScheduleVectors {
    #[serde(rename = "_WARNING")]
    warning: String,
    description: String,
    vectors: Vec<KeyScheduleVector>,
}

#[derive(Serialize)]
struct KeyScheduleVector {
    id: String,
    ephemeral_shared_secret_hex: String,
    expected_session_root_key_hex: String,
}

pub fn generate_key_schedule_json() -> String {
    let vectors: Vec<KeyScheduleVector> = (0..3)
        .map(|i| {
            let seed = (i * 50) as u8;
            let secret = make_key(seed);
            let srk = derive_session_root(&secret);
            KeyScheduleVector {
                id: format!("session-root-{i}"),
                ephemeral_shared_secret_hex: to_hex(&secret),
                expected_session_root_key_hex: to_hex(&srk),
            }
        })
        .collect();

    let data = KeyScheduleVectors {
        warning: "TEST FIXTURES ONLY — deterministic keys, not for production.".into(),
        description: "BTR session root derivation via HKDF-SHA256 (§16.3).".into(),
        vectors,
    };
    serde_json::to_string_pretty(&data).unwrap() + "\n"
}

// ── btr-transfer-ratchet ──────────────────────────────────────────────

#[derive(Serialize)]
struct TransferRatchetVectors {
    #[serde(rename = "_WARNING")]
    warning: String,
    description: String,
    vectors: Vec<TransferRatchetVector>,
}

#[derive(Serialize)]
struct TransferRatchetVector {
    id: String,
    session_root_key_hex: String,
    transfer_id_hex: String,
    expected_transfer_root_key_hex: String,
}

pub fn generate_transfer_ratchet_json() -> String {
    let srk = make_key(0xAB);
    let vectors: Vec<TransferRatchetVector> = (0..4)
        .map(|i| {
            let tid = make_tid((i * 30) as u8);
            let trk = derive_transfer_root(&srk, &tid);
            TransferRatchetVector {
                id: format!("transfer-root-{i}"),
                session_root_key_hex: to_hex(&srk),
                transfer_id_hex: to_hex(&tid),
                expected_transfer_root_key_hex: to_hex(&trk),
            }
        })
        .collect();

    let data = TransferRatchetVectors {
        warning: "TEST FIXTURES ONLY — deterministic keys, not for production.".into(),
        description: "BTR transfer root derivation via HKDF-SHA256 (§16.3).".into(),
        vectors,
    };
    serde_json::to_string_pretty(&data).unwrap() + "\n"
}

// ── btr-chain-advance ─────────────────────────────────────────────────

#[derive(Serialize)]
struct ChainAdvanceVectors {
    #[serde(rename = "_WARNING")]
    warning: String,
    description: String,
    vectors: Vec<ChainAdvanceVector>,
}

#[derive(Serialize)]
struct ChainAdvanceVector {
    id: String,
    chain_key_hex: String,
    expected_message_key_hex: String,
    expected_next_chain_key_hex: String,
}

pub fn generate_chain_advance_json() -> String {
    let mut ck = make_key(0x01);
    let mut vectors = Vec::with_capacity(5);
    for i in 0..5 {
        let out = chain_advance(&ck);
        vectors.push(ChainAdvanceVector {
            id: format!("chain-step-{i}"),
            chain_key_hex: to_hex(&ck),
            expected_message_key_hex: to_hex(&out.message_key),
            expected_next_chain_key_hex: to_hex(&out.next_chain_key),
        });
        ck = out.next_chain_key;
    }

    let data = ChainAdvanceVectors {
        warning: "TEST FIXTURES ONLY — deterministic keys, not for production.".into(),
        description: "BTR per-chunk symmetric chain KDF (§16.3).".into(),
        vectors,
    };
    serde_json::to_string_pretty(&data).unwrap() + "\n"
}

// ── btr-replay-reject ─────────────────────────────────────────────────

#[derive(Serialize)]
struct ReplayRejectVectors {
    #[serde(rename = "_WARNING")]
    warning: String,
    description: String,
    vectors: Vec<ReplayRejectVector>,
}

#[derive(Serialize)]
struct ReplayRejectVector {
    id: String,
    description: String,
    transfer_id_hex: String,
    ratchet_generation: u32,
    chain_index: u32,
    prior_accepted: Vec<ReplayPrior>,
    expected_reject: bool,
    expected_error_code: Option<String>,
}

#[derive(Serialize)]
struct ReplayPrior {
    transfer_id_hex: String,
    ratchet_generation: u32,
    chain_index: u32,
}

pub fn generate_replay_reject_json() -> String {
    let tid = make_tid(0x01);
    let tid_hex = to_hex(&tid);
    let tid2 = make_tid(0x02);
    let tid2_hex = to_hex(&tid2);

    let vectors = vec![
        ReplayRejectVector {
            id: "accept-first-chunk".into(),
            description: "First chunk of first transfer — must accept.".into(),
            transfer_id_hex: tid_hex.clone(),
            ratchet_generation: 0,
            chain_index: 0,
            prior_accepted: vec![],
            expected_reject: false,
            expected_error_code: None,
        },
        ReplayRejectVector {
            id: "reject-duplicate".into(),
            description: "Same (tid, gen, idx) replayed — must reject.".into(),
            transfer_id_hex: tid_hex.clone(),
            ratchet_generation: 0,
            chain_index: 0,
            prior_accepted: vec![ReplayPrior {
                transfer_id_hex: tid_hex.clone(),
                ratchet_generation: 0,
                chain_index: 0,
            }],
            expected_reject: true,
            expected_error_code: Some("RATCHET_CHAIN_ERROR".into()),
        },
        ReplayRejectVector {
            id: "reject-skipped-index".into(),
            description: "chain_index=2 when expected=1 — ORDER-BTR violation.".into(),
            transfer_id_hex: tid_hex.clone(),
            ratchet_generation: 0,
            chain_index: 2,
            prior_accepted: vec![ReplayPrior {
                transfer_id_hex: tid_hex.clone(),
                ratchet_generation: 0,
                chain_index: 0,
            }],
            expected_reject: true,
            expected_error_code: Some("RATCHET_CHAIN_ERROR".into()),
        },
        ReplayRejectVector {
            id: "reject-wrong-generation".into(),
            description: "Generation mismatch — cross-generation replay attempt.".into(),
            transfer_id_hex: tid2_hex.clone(),
            ratchet_generation: 1,
            chain_index: 0,
            prior_accepted: vec![ReplayPrior {
                transfer_id_hex: tid_hex.clone(),
                ratchet_generation: 0,
                chain_index: 0,
            }],
            expected_reject: true,
            expected_error_code: Some("RATCHET_STATE_ERROR".into()),
        },
    ];

    let data = ReplayRejectVectors {
        warning: "TEST FIXTURES ONLY — deterministic test data.".into(),
        description:
            "BTR replay rejection for (transfer_id, ratchet_generation, chain_index) (§11).".into(),
        vectors,
    };
    serde_json::to_string_pretty(&data).unwrap() + "\n"
}

// ── btr-downgrade-negotiate ───────────────────────────────────────────

#[derive(Serialize)]
struct DowngradeNegotiateVectors {
    #[serde(rename = "_WARNING")]
    warning: String,
    description: String,
    vectors: Vec<DowngradeNegotiateVector>,
}

#[derive(Serialize)]
struct DowngradeNegotiateVector {
    id: String,
    description: String,
    local_supports_btr: bool,
    remote_supports_btr: bool,
    remote_well_formed: bool,
    expected_mode: String,
    expected_log_token: Option<String>,
}

fn mode_str(mode: BtrMode) -> &'static str {
    match mode {
        BtrMode::FullBtr => "FULL_BTR",
        BtrMode::Downgrade => "DOWNGRADE",
        BtrMode::StaticEphemeral => "STATIC_EPHEMERAL",
        BtrMode::Reject => "REJECT",
    }
}

pub fn generate_downgrade_negotiate_json() -> String {
    let cases = [
        (
            "both-support-ok",
            "Both peers support BTR, well-formed.",
            true,
            true,
            true,
        ),
        ("local-only", "Only local supports BTR.", true, false, true),
        (
            "remote-only",
            "Only remote supports BTR.",
            false,
            true,
            true,
        ),
        ("neither", "Neither peer supports BTR.", false, false, true),
        (
            "both-malformed-remote",
            "Both support but remote is malformed.",
            true,
            true,
            false,
        ),
        (
            "local-no-remote-malformed",
            "Local does not support, remote malformed — downgrade (malformation irrelevant).",
            false,
            true,
            false,
        ),
    ];

    let vectors: Vec<DowngradeNegotiateVector> = cases
        .iter()
        .map(|(id, desc, local, remote, wf)| {
            let mode = negotiate_btr(*local, *remote, *wf);
            let log = crate::negotiate::btr_log_token(mode).map(String::from);
            DowngradeNegotiateVector {
                id: id.to_string(),
                description: desc.to_string(),
                local_supports_btr: *local,
                remote_supports_btr: *remote,
                remote_well_formed: *wf,
                expected_mode: mode_str(mode).to_string(),
                expected_log_token: log,
            }
        })
        .collect();

    let data = DowngradeNegotiateVectors {
        warning: "TEST FIXTURES ONLY — deterministic test data.".into(),
        description: "BTR capability negotiation matrix (§4, 6 cells).".into(),
        vectors,
    };
    serde_json::to_string_pretty(&data).unwrap() + "\n"
}

// ── DH ratchet step vector ────────────────────────────────────────────

#[derive(Serialize)]
struct DhRatchetVectors {
    #[serde(rename = "_WARNING")]
    warning: String,
    description: String,
    vectors: Vec<DhRatchetVector>,
}

#[derive(Serialize)]
struct DhRatchetVector {
    id: String,
    current_session_root_key_hex: String,
    dh_output_hex: String,
    expected_new_session_root_key_hex: String,
}

pub fn generate_dh_ratchet_json() -> String {
    let vectors: Vec<DhRatchetVector> = (0..3)
        .map(|i| {
            let srk = make_key((i * 40) as u8);
            let dh = make_key((i * 40 + 100) as u8);
            let new_srk = derive_ratcheted_session_root(&srk, &dh);
            DhRatchetVector {
                id: format!("dh-ratchet-step-{i}"),
                current_session_root_key_hex: to_hex(&srk),
                dh_output_hex: to_hex(&dh),
                expected_new_session_root_key_hex: to_hex(&new_srk),
            }
        })
        .collect();

    let data = DhRatchetVectors {
        warning: "TEST FIXTURES ONLY — deterministic keys, not for production.".into(),
        description: "BTR inter-transfer DH ratchet step via HKDF-SHA256 (§16.3).".into(),
        vectors,
    };
    serde_json::to_string_pretty(&data).unwrap() + "\n"
}

// ── btr-encrypt-decrypt (deterministic fixed-nonce) ─────────────────

#[derive(Serialize)]
struct EncryptDecryptVectors {
    #[serde(rename = "_WARNING")]
    warning: String,
    description: String,
    vectors: Vec<EncryptDecryptVector>,
}

#[derive(Serialize)]
struct EncryptDecryptVector {
    id: String,
    description: String,
    message_key_hex: String,
    nonce_hex: String,
    plaintext_hex: String,
    expected_ciphertext_hex: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    expect_error: Option<String>,
}

/// Generate deterministic encrypt/decrypt vectors using fixed nonces.
///
/// Uses NaCl secretbox (XSalsa20-Poly1305) directly with known nonces
/// so that TS can reproduce byte-identical ciphertext.
pub fn generate_encrypt_decrypt_json() -> String {
    let key = make_key(0xE0);
    let nonce_bytes: [u8; 24] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
        0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
    ];

    let cipher = XSalsa20Poly1305::new((&key).into());
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Helper: encrypt with fixed nonce, return nonce || ciphertext hex
    let seal = |plaintext: &[u8]| -> String {
        let ct = cipher.encrypt(nonce, plaintext).unwrap();
        let mut combined = Vec::with_capacity(24 + ct.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ct);
        to_hex(&combined)
    };

    // Vector 1: empty plaintext
    let empty_pt = vec![];
    let empty_ct = seal(&empty_pt);

    // Vector 2: small plaintext
    let small_pt = b"hello";
    let small_ct = seal(small_pt);

    // Vector 3: chunk-sized (256 bytes — representative, not 16K for file size)
    let chunk_pt: Vec<u8> = (0..256).map(|i| (i & 0xFF) as u8).collect();
    let chunk_ct = seal(&chunk_pt);

    // Vector 4: multi-byte random-ish plaintext
    let multi_pt: Vec<u8> = (0..73).map(|i| ((i * 7 + 13) & 0xFF) as u8).collect();
    let multi_ct = seal(&multi_pt);

    // Vector 5: tampered ciphertext — flip last byte of a valid seal
    let valid_sealed = {
        let ct = cipher.encrypt(nonce, b"tamper-test".as_slice()).unwrap();
        let mut combined = Vec::with_capacity(24 + ct.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ct);
        combined
    };
    let mut tampered = valid_sealed.clone();
    let last = tampered.len() - 1;
    tampered[last] ^= 0x01;

    let vectors = vec![
        EncryptDecryptVector {
            id: "empty-plaintext".into(),
            description: "Empty plaintext — secretbox must handle zero-length.".into(),
            message_key_hex: to_hex(&key),
            nonce_hex: to_hex(&nonce_bytes),
            plaintext_hex: to_hex(&empty_pt),
            expected_ciphertext_hex: empty_ct,
            expect_error: None,
        },
        EncryptDecryptVector {
            id: "small-plaintext".into(),
            description: "5-byte ASCII plaintext.".into(),
            message_key_hex: to_hex(&key),
            nonce_hex: to_hex(&nonce_bytes),
            plaintext_hex: to_hex(small_pt),
            expected_ciphertext_hex: small_ct,
            expect_error: None,
        },
        EncryptDecryptVector {
            id: "chunk-sized-plaintext".into(),
            description: "256-byte sequential plaintext (representative chunk).".into(),
            message_key_hex: to_hex(&key),
            nonce_hex: to_hex(&nonce_bytes),
            plaintext_hex: to_hex(&chunk_pt),
            expected_ciphertext_hex: chunk_ct,
            expect_error: None,
        },
        EncryptDecryptVector {
            id: "multi-byte-plaintext".into(),
            description: "73-byte pseudo-random plaintext.".into(),
            message_key_hex: to_hex(&key),
            nonce_hex: to_hex(&nonce_bytes),
            plaintext_hex: to_hex(&multi_pt),
            expected_ciphertext_hex: multi_ct,
            expect_error: None,
        },
        EncryptDecryptVector {
            id: "tampered-ciphertext".into(),
            description: "Last byte of ciphertext flipped — MAC must reject.".into(),
            message_key_hex: to_hex(&key),
            nonce_hex: to_hex(&nonce_bytes),
            plaintext_hex: "".into(),
            expected_ciphertext_hex: to_hex(&tampered),
            expect_error: Some("RATCHET_DECRYPT_FAIL".into()),
        },
        EncryptDecryptVector {
            id: "truncated-ciphertext".into(),
            description: "Sealed payload shorter than nonce+MAC — must reject.".into(),
            message_key_hex: to_hex(&key),
            nonce_hex: to_hex(&nonce_bytes),
            plaintext_hex: "".into(),
            expected_ciphertext_hex: to_hex(&[0u8; 10]),
            expect_error: Some("RATCHET_DECRYPT_FAIL".into()),
        },
    ];

    let data = EncryptDecryptVectors {
        warning: "TEST FIXTURES ONLY — fixed nonces, not for production.".into(),
        description: "BTR deterministic encrypt/decrypt via NaCl secretbox (§16.4). Fixed nonces for cross-language parity.".into(),
        vectors,
    };
    serde_json::to_string_pretty(&data).unwrap() + "\n"
}

// ── btr-dh-sanity (X25519 cross-library) ────────────────────────────

#[derive(Serialize)]
struct DhSanityVectors {
    #[serde(rename = "_WARNING")]
    warning: String,
    description: String,
    vectors: Vec<DhSanityVector>,
}

#[derive(Serialize)]
struct DhSanityVector {
    id: String,
    description: String,
    secret_scalar_hex: String,
    remote_public_hex: String,
    expected_shared_secret_hex: String,
}

/// Generate DH sanity check vectors for tweetnacl.scalarMult vs x25519-dalek.
///
/// Uses deterministic secret scalars (NOT CSPRNG) and derives public keys
/// from them, then computes DH outputs for cross-library verification.
pub fn generate_dh_sanity_json() -> String {
    // Use StaticSecret for deterministic scalar injection.
    // EphemeralSecret cannot be constructed from known bytes.
    let scalars: Vec<[u8; 32]> = vec![
        make_key(0xA0),
        make_key(0xB0),
        make_key(0xC0),
    ];

    let secrets: Vec<StaticSecret> = scalars
        .iter()
        .map(|s| StaticSecret::from(*s))
        .collect();

    let publics: Vec<PublicKey> = secrets
        .iter()
        .map(PublicKey::from)
        .collect();

    let vectors = vec![
        // Alice (A0) × Bob (B0)
        {
            let shared = secrets[0].diffie_hellman(&publics[1]);
            DhSanityVector {
                id: "dh-a0-b0".into(),
                description: "scalar A0 × public B0 — basic DH.".into(),
                secret_scalar_hex: to_hex(&scalars[0]),
                remote_public_hex: to_hex(publics[1].as_bytes()),
                expected_shared_secret_hex: to_hex(shared.as_bytes()),
            }
        },
        // Bob (B0) × Alice (A0) — must equal A0×B0 (commutativity)
        {
            let shared = secrets[1].diffie_hellman(&publics[0]);
            DhSanityVector {
                id: "dh-b0-a0".into(),
                description: "scalar B0 × public A0 — commutativity check (must equal dh-a0-b0).".into(),
                secret_scalar_hex: to_hex(&scalars[1]),
                remote_public_hex: to_hex(publics[0].as_bytes()),
                expected_shared_secret_hex: to_hex(shared.as_bytes()),
            }
        },
        // Charlie (C0) × Alice (A0)
        {
            let shared = secrets[2].diffie_hellman(&publics[0]);
            DhSanityVector {
                id: "dh-c0-a0".into(),
                description: "scalar C0 × public A0 — different pair yields different output.".into(),
                secret_scalar_hex: to_hex(&scalars[2]),
                remote_public_hex: to_hex(publics[0].as_bytes()),
                expected_shared_secret_hex: to_hex(shared.as_bytes()),
            }
        },
        // Scalar-to-basepoint: derive public key from scalar A0
        {
            DhSanityVector {
                id: "scalar-to-public-a0".into(),
                description: "scalar A0 × basepoint — public key derivation sanity.".into(),
                secret_scalar_hex: to_hex(&scalars[0]),
                remote_public_hex: to_hex(&[
                    0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ]), // X25519 basepoint
                expected_shared_secret_hex: to_hex(publics[0].as_bytes()),
            }
        },
    ];

    let data = DhSanityVectors {
        warning: "TEST FIXTURES ONLY — deterministic scalars, not for production.".into(),
        description: "X25519 DH cross-library sanity: x25519-dalek (Rust) vs tweetnacl.scalarMult (TS).".into(),
        vectors,
    };
    serde_json::to_string_pretty(&data).unwrap() + "\n"
}
