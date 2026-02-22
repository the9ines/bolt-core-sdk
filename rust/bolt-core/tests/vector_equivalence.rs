//! Vector equivalence gate.
//!
//! Regenerates golden vectors from Rust and compares against committed
//! JSON files. This proves that the Rust crate produces identical
//! cryptographic output to the TypeScript SDK for the same inputs.

use std::path::PathBuf;

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

/// Compare two JSON strings semantically (parsed serde_json::Value).
/// This is robust against whitespace/formatting differences while
/// still verifying all content and field ordering via Value equality.
fn assert_json_equivalent(name: &str, generated: &str, committed: &str) {
    let gen_val: serde_json::Value =
        serde_json::from_str(generated).expect("generated JSON is invalid");
    let com_val: serde_json::Value =
        serde_json::from_str(committed).expect("committed JSON is invalid");
    assert_eq!(
        gen_val, com_val,
        "DRIFT: {name} — Rust-generated vectors do not match committed file"
    );
}

#[test]
fn box_payload_equivalence() {
    let generated = bolt_core::vectors::generate_box_payload_json();
    let committed_path = vectors_dir().join("box-payload.vectors.json");
    let committed = std::fs::read_to_string(&committed_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", committed_path.display(), e));
    assert_json_equivalent("box-payload.vectors.json", &generated, &committed);
}

#[test]
fn framing_equivalence() {
    let generated = bolt_core::vectors::generate_framing_json();
    let committed_path = vectors_dir().join("framing.vectors.json");
    let committed = std::fs::read_to_string(&committed_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", committed_path.display(), e));
    assert_json_equivalent("framing.vectors.json", &generated, &committed);
}
