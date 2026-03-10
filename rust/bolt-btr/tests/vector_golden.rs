//! Golden vector generation + determinism tests.
//!
//! Generates BTR test vector JSON files and verifies they are deterministic
//! (running twice produces identical output).

#![cfg(feature = "vectors")]

use bolt_btr::vectors;
use std::path::PathBuf;

fn vector_dir() -> PathBuf {
    // bolt-btr/tests/../../../bolt-core/test-vectors/btr/
    let manifest = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest)
        .parent()
        .unwrap()
        .join("bolt-core")
        .join("test-vectors")
        .join("btr")
}

fn write_and_verify(filename: &str, generator: fn() -> String) {
    let path = vector_dir().join(filename);

    // Generate twice and verify determinism
    let first = generator();
    let second = generator();
    assert_eq!(
        first, second,
        "vector generation is not deterministic: {filename}"
    );

    // Write to disk
    std::fs::write(&path, &first).unwrap_or_else(|e| {
        panic!("failed to write {}: {e}", path.display());
    });

    // Read back and verify
    let read_back = std::fs::read_to_string(&path).unwrap();
    assert_eq!(read_back, first, "write/read mismatch for {filename}");
}

#[test]
fn generate_key_schedule_vectors() {
    write_and_verify(
        "btr-key-schedule.vectors.json",
        vectors::generate_key_schedule_json,
    );
}

#[test]
fn generate_transfer_ratchet_vectors() {
    write_and_verify(
        "btr-transfer-ratchet.vectors.json",
        vectors::generate_transfer_ratchet_json,
    );
}

#[test]
fn generate_chain_advance_vectors() {
    write_and_verify(
        "btr-chain-advance.vectors.json",
        vectors::generate_chain_advance_json,
    );
}

#[test]
fn generate_replay_reject_vectors() {
    write_and_verify(
        "btr-replay-reject.vectors.json",
        vectors::generate_replay_reject_json,
    );
}

#[test]
fn generate_downgrade_negotiate_vectors() {
    write_and_verify(
        "btr-downgrade-negotiate.vectors.json",
        vectors::generate_downgrade_negotiate_json,
    );
}

#[test]
fn generate_dh_ratchet_vectors() {
    write_and_verify(
        "btr-dh-ratchet.vectors.json",
        vectors::generate_dh_ratchet_json,
    );
}

#[test]
fn generate_encrypt_decrypt_vectors() {
    write_and_verify(
        "btr-encrypt-decrypt.vectors.json",
        vectors::generate_encrypt_decrypt_json,
    );
}

#[test]
fn generate_dh_sanity_vectors() {
    write_and_verify(
        "btr-dh-sanity.vectors.json",
        vectors::generate_dh_sanity_json,
    );
}

#[test]
fn generate_lifecycle_vectors() {
    write_and_verify(
        "btr-lifecycle.vectors.json",
        vectors::generate_lifecycle_json,
    );
}

#[test]
fn generate_adversarial_vectors() {
    write_and_verify(
        "btr-adversarial.vectors.json",
        vectors::generate_adversarial_json,
    );
}

/// Generates all vector files and then verifies they all exist.
#[test]
fn all_vector_files_present() {
    let dir = vector_dir();
    let generators: Vec<(&str, fn() -> String)> = vec![
        (
            "btr-key-schedule.vectors.json",
            vectors::generate_key_schedule_json,
        ),
        (
            "btr-transfer-ratchet.vectors.json",
            vectors::generate_transfer_ratchet_json,
        ),
        (
            "btr-chain-advance.vectors.json",
            vectors::generate_chain_advance_json,
        ),
        (
            "btr-replay-reject.vectors.json",
            vectors::generate_replay_reject_json,
        ),
        (
            "btr-downgrade-negotiate.vectors.json",
            vectors::generate_downgrade_negotiate_json,
        ),
        (
            "btr-dh-ratchet.vectors.json",
            vectors::generate_dh_ratchet_json,
        ),
        (
            "btr-encrypt-decrypt.vectors.json",
            vectors::generate_encrypt_decrypt_json,
        ),
        (
            "btr-dh-sanity.vectors.json",
            vectors::generate_dh_sanity_json,
        ),
        (
            "btr-lifecycle.vectors.json",
            vectors::generate_lifecycle_json,
        ),
        (
            "btr-adversarial.vectors.json",
            vectors::generate_adversarial_json,
        ),
    ];
    for (filename, gen) in &generators {
        let path = dir.join(filename);
        std::fs::write(&path, gen()).unwrap_or_else(|e| {
            panic!("failed to write {}: {e}", path.display());
        });
    }
    for (filename, _) in &generators {
        let path = dir.join(filename);
        assert!(path.exists(), "missing vector file: {}", path.display());
    }
}
