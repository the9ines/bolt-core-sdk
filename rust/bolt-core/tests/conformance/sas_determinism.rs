//! Conformance: SAS Determinism
//!
//! Invariants under test:
//! - PROTO-06: SAS MUST be computed over raw 32-byte keys, not encoded
//!   representations
//! - SAS MUST be commutative (both peers compute identical SAS)
//! - SAS MUST be deterministic (identical inputs → identical output)
//! - SAS output MUST be exactly 6 uppercase hex characters
//!
//! Uses H3 golden vectors from ts/bolt-core/__tests__/vectors/sas.vectors.json.
//! Does NOT import bolt-rendezvous-protocol or any cross-repo dependency.

use serde::Deserialize;
use std::path::PathBuf;

// ── Vector schema ───────────────────────────────────────────────

#[derive(Deserialize)]
struct SasVectors {
    version: u32,
    cases: Vec<SasCase>,
}

#[derive(Deserialize)]
struct SasCase {
    name: String,
    identity_a_hex: String,
    identity_b_hex: String,
    ephemeral_a_hex: String,
    ephemeral_b_hex: String,
    expected_sas: String,
}

// ── Helpers ─────────────────────────────────────────────────────

fn vectors_dir() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .join("..")
        .join("..")
        .join("ts")
        .join("bolt-core")
        .join("__tests__")
        .join("vectors")
}

fn hex_to_32(hex: &str) -> [u8; 32] {
    bolt_core::encoding::from_hex(hex)
        .expect("invalid hex")
        .try_into()
        .expect("expected 32 bytes")
}

fn load_sas_vectors() -> SasVectors {
    let path = vectors_dir().join("sas.vectors.json");
    let data = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    serde_json::from_str(&data).expect("sas vectors parse failed")
}

// ── Conformance: Golden Vector Match ────────────────────────────

/// PROTO-06: Every SAS golden vector produces the expected output.
#[test]
fn conformance_sas_golden_vectors_match() {
    let vecs = load_sas_vectors();

    assert_eq!(vecs.version, 1, "unexpected SAS vector schema version");
    assert!(
        vecs.cases.len() >= 3,
        "expected at least 3 SAS cases, got {}",
        vecs.cases.len()
    );

    for case in &vecs.cases {
        let id_a = hex_to_32(&case.identity_a_hex);
        let id_b = hex_to_32(&case.identity_b_hex);
        let eph_a = hex_to_32(&case.ephemeral_a_hex);
        let eph_b = hex_to_32(&case.ephemeral_b_hex);

        let sas = bolt_core::sas::compute_sas(&id_a, &id_b, &eph_a, &eph_b);
        assert_eq!(
            sas, case.expected_sas,
            "SAS mismatch for case '{}': got '{sas}', expected '{}'",
            case.name, case.expected_sas
        );
    }
}

// ── Conformance: Commutativity ──────────────────────────────────

/// PROTO-06: Swapping A/B positions MUST produce identical SAS.
/// Both peers compute the same SAS regardless of who is "A" or "B".
#[test]
fn conformance_sas_commutative_all_vectors() {
    let vecs = load_sas_vectors();

    for case in &vecs.cases {
        let id_a = hex_to_32(&case.identity_a_hex);
        let id_b = hex_to_32(&case.identity_b_hex);
        let eph_a = hex_to_32(&case.ephemeral_a_hex);
        let eph_b = hex_to_32(&case.ephemeral_b_hex);

        let sas_ab = bolt_core::sas::compute_sas(&id_a, &id_b, &eph_a, &eph_b);
        let sas_ba = bolt_core::sas::compute_sas(&id_b, &id_a, &eph_b, &eph_a);

        assert_eq!(
            sas_ab, sas_ba,
            "SAS not commutative for case '{}': A→B='{}', B→A='{}'",
            case.name, sas_ab, sas_ba
        );
    }
}

// ── Conformance: Determinism (No Entropy Drift) ─────────────────

/// PROTO-06: Same inputs MUST always produce the same SAS output.
/// Verifies no hidden entropy source (no randomness in SAS computation).
#[test]
fn conformance_sas_deterministic_100_rounds() {
    let id_a = hex_to_32("07a37cbc142093c8b755dc1b10e86cb426374ad16aa853ed0bdfc0b2b86d1c7c");
    let id_b = hex_to_32("5869aff450549732cbaaed5e5df9b30a6da31cb0e5742bad5ad4a1a768f1a67b");
    let eph_a = hex_to_32("64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d48466");
    let eph_b = hex_to_32("01000000000000000000000000000000000000000000000000000000000000aa");

    let reference = bolt_core::sas::compute_sas(&id_a, &id_b, &eph_a, &eph_b);

    for round in 1..=100 {
        let sas = bolt_core::sas::compute_sas(&id_a, &id_b, &eph_a, &eph_b);
        assert_eq!(
            sas, reference,
            "SAS drift at round {round}: got '{sas}', expected '{reference}'"
        );
    }
}

// ── Conformance: Output Format ──────────────────────────────────

/// SAS output MUST be exactly 6 uppercase hex characters for all inputs.
#[test]
fn conformance_sas_output_format_all_vectors() {
    let vecs = load_sas_vectors();

    for case in &vecs.cases {
        let id_a = hex_to_32(&case.identity_a_hex);
        let id_b = hex_to_32(&case.identity_b_hex);
        let eph_a = hex_to_32(&case.ephemeral_a_hex);
        let eph_b = hex_to_32(&case.ephemeral_b_hex);

        let sas = bolt_core::sas::compute_sas(&id_a, &id_b, &eph_a, &eph_b);

        assert_eq!(sas.len(), 6, "SAS length != 6 for case '{}'", case.name);
        assert!(
            sas.chars().all(|c| c.is_ascii_hexdigit()),
            "SAS contains non-hex chars for case '{}': '{sas}'",
            case.name
        );
        assert_eq!(
            sas,
            sas.to_uppercase(),
            "SAS not uppercase for case '{}': '{sas}'",
            case.name
        );
    }
}

// ── Conformance: Distinct Keys → Distinct SAS ──────────────────

/// Different key sets in the golden vectors MUST produce different SAS values.
/// Detects catastrophic hash collision or constant-output regression.
#[test]
fn conformance_sas_distinct_inputs_produce_distinct_outputs() {
    let vecs = load_sas_vectors();
    let mut sas_set = std::collections::HashSet::new();

    for case in &vecs.cases {
        let id_a = hex_to_32(&case.identity_a_hex);
        let id_b = hex_to_32(&case.identity_b_hex);
        let eph_a = hex_to_32(&case.ephemeral_a_hex);
        let eph_b = hex_to_32(&case.ephemeral_b_hex);

        let sas = bolt_core::sas::compute_sas(&id_a, &id_b, &eph_a, &eph_b);
        sas_set.insert(sas);
    }

    // All golden vector cases use distinct key combinations.
    assert_eq!(
        sas_set.len(),
        vecs.cases.len(),
        "SAS collision detected among golden vector cases ({} unique out of {})",
        sas_set.len(),
        vecs.cases.len()
    );
}
