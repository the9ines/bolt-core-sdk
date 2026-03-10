//! BTR golden vector generator — Rust authority for cross-language parity.
//!
//! Generates deterministic JSON fixtures for 5 BTR vector categories.
//! Output path: `bolt-core-sdk/rust/bolt-core/test-vectors/btr/`
//! TS parity consumption: `bolt-core-sdk/ts/bolt-core/__tests__/vectors/btr/`
//!
//! TEST FIXTURES ONLY — all keys are deterministic and publicly known.

use serde::Serialize;

use crate::key_schedule::{chain_advance, derive_session_root, derive_transfer_root};
use crate::negotiate::{negotiate_btr, BtrMode};
use crate::ratchet::derive_ratcheted_session_root;

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
