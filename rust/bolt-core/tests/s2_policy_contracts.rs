//! S2 transfer policy contract tests.
//!
//! These tests validate CONTRACTS, not scheduling quality.
//! The stub policy is acceptable if it satisfies these contracts.
//! A real implementation must also pass every test here.

use bolt_core::transfer_policy::{
    decide, Backpressure, DeviceClass, FairnessMode, LinkStats, PolicyInput, ScheduleDecision,
    TransferConstraints, MAX_PACING_DELAY_MS,
};

fn make_input(pending: Vec<u32>, max_parallel: u16, max_bytes: u32, in_flight: u32) -> PolicyInput {
    PolicyInput {
        pending_chunk_ids: pending,
        link_stats: LinkStats {
            rtt_ms: 20,
            loss_ppm: 0,
            in_flight_bytes: in_flight,
        },
        device_class: DeviceClass::Desktop,
        constraints: TransferConstraints {
            max_parallel_chunks: max_parallel,
            max_in_flight_bytes: max_bytes,
            priority: 128,
            fairness_mode: FairnessMode::Balanced,
        },
    }
}

// ─── Determinism ───────────────────────────────────────────────────────

#[test]
fn determinism_identical_inputs_produce_identical_outputs() {
    let input = make_input(vec![10, 20, 30, 40, 50], 3, 65536, 0);
    let d1 = decide(&input);
    let d2 = decide(&input);
    assert_eq!(d1, d2, "identical inputs must produce identical decisions");
}

#[test]
fn determinism_ordering_preserved() {
    let input = make_input(vec![99, 42, 7, 1000, 3], 4, 65536, 0);
    let d1 = decide(&input);
    let d2 = decide(&input);
    assert_eq!(
        d1.next_chunk_ids, d2.next_chunk_ids,
        "chunk ID ordering must be deterministic"
    );
    // Also verify order matches input order (first N from pending).
    assert_eq!(d1.next_chunk_ids, vec![99, 42, 7, 1000]);
}

#[test]
fn determinism_across_device_classes() {
    // Same constraints, different device class — both must be deterministic.
    for class in [
        DeviceClass::Desktop,
        DeviceClass::Mobile,
        DeviceClass::LowPower,
        DeviceClass::Unknown,
    ] {
        let mut input = make_input(vec![1, 2, 3], 2, 65536, 0);
        input.device_class = class;
        let d1 = decide(&input);
        let d2 = decide(&input);
        assert_eq!(d1, d2, "determinism must hold for {:?}", class);
    }
}

#[test]
fn determinism_across_fairness_modes() {
    for mode in [
        FairnessMode::Balanced,
        FairnessMode::Throughput,
        FairnessMode::Latency,
    ] {
        let mut input = make_input(vec![1, 2, 3], 2, 65536, 0);
        input.constraints.fairness_mode = mode;
        let d1 = decide(&input);
        let d2 = decide(&input);
        assert_eq!(d1, d2, "determinism must hold for {:?}", mode);
    }
}

// ─── Bounds ────────────────────────────────────────────────────────────

#[test]
fn bound_next_chunk_ids_within_max_parallel() {
    let input = make_input(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9], 4, 65536, 0);
    let d = decide(&input);
    assert!(
        d.next_chunk_ids.len() <= 4,
        "next_chunk_ids.len() ({}) must be <= max_parallel_chunks (4)",
        d.next_chunk_ids.len()
    );
}

#[test]
fn bound_window_suggestion_within_max_parallel() {
    let input = make_input(vec![0, 1, 2, 3, 4, 5], 3, 65536, 0);
    let d = decide(&input);
    assert!(
        d.window_suggestion_chunks <= 3,
        "window_suggestion_chunks ({}) must be <= max_parallel_chunks (3)",
        d.window_suggestion_chunks
    );
}

#[test]
fn bound_pacing_delay_within_max() {
    let input = make_input(vec![0, 1, 2], 3, 65536, 0);
    let d = decide(&input);
    assert!(
        d.pacing_delay_ms <= MAX_PACING_DELAY_MS,
        "pacing_delay_ms ({}) must be <= MAX_PACING_DELAY_MS ({})",
        d.pacing_delay_ms,
        MAX_PACING_DELAY_MS
    );
}

#[test]
fn bound_max_parallel_one() {
    let input = make_input(vec![0, 1, 2, 3], 1, 65536, 0);
    let d = decide(&input);
    assert!(d.next_chunk_ids.len() <= 1);
    assert!(d.window_suggestion_chunks <= 1);
}

#[test]
fn bound_max_parallel_zero() {
    let input = make_input(vec![0, 1, 2], 0, 65536, 0);
    let d = decide(&input);
    assert!(d.next_chunk_ids.is_empty());
    assert_eq!(d.window_suggestion_chunks, 0);
}

// ─── Backpressure ──────────────────────────────────────────────────────

#[test]
fn backpressure_pause_when_over_budget() {
    let input = make_input(vec![0, 1, 2], 3, 1000, 1001);
    let d = decide(&input);
    assert_eq!(
        d.backpressure,
        Backpressure::Pause,
        "must pause when in_flight_bytes > max_in_flight_bytes"
    );
    assert!(
        d.next_chunk_ids.is_empty(),
        "must not schedule chunks when over budget"
    );
}

#[test]
fn backpressure_pause_at_exact_boundary() {
    // At exact boundary (equal), should NOT pause.
    let input = make_input(vec![0, 1], 2, 1000, 1000);
    let d = decide(&input);
    assert_ne!(
        d.backpressure,
        Backpressure::Pause,
        "at exact boundary (equal), should not pause"
    );
}

#[test]
fn backpressure_pause_one_byte_over() {
    let input = make_input(vec![0, 1], 2, 1000, 1001);
    let d = decide(&input);
    assert_eq!(d.backpressure, Backpressure::Pause);
    assert!(d.next_chunk_ids.is_empty());
}

// ─── Sanity ────────────────────────────────────────────────────────────

#[test]
fn sanity_empty_pending_empty_result() {
    let input = make_input(vec![], 10, 65536, 0);
    let d = decide(&input);
    assert!(d.next_chunk_ids.is_empty());
}

#[test]
fn sanity_single_chunk() {
    let input = make_input(vec![42], 10, 65536, 0);
    let d = decide(&input);
    assert_eq!(d.next_chunk_ids, vec![42]);
}

#[test]
fn sanity_max_pacing_constant_is_reasonable() {
    // Document: max 5 seconds. This guards against accidental changes.
    assert_eq!(MAX_PACING_DELAY_MS, 5_000);
}
