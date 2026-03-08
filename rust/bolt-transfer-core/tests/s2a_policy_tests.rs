//! S2A transfer policy contract tests.
//!
//! These tests validate CONTRACTS and substantive scheduling behavior.
//! Migrated from bolt-core/tests/s2_policy_contracts.rs and extended
//! with S2A-specific tests for window sizing, effective chunk cap,
//! stall detection, and progress cadence.

use bolt_transfer_core::policy::{
    decide, detect_stall, progress_cadence, Backpressure, DeviceClass, FairnessMode, LinkStats,
    PolicyInput, PressureState, ProgressConfig, StallClassification, StallInput,
    TransferConstraints, MAX_PACING_DELAY_MS,
};

fn make_input(
    pending: Vec<u32>,
    max_parallel: u16,
    max_bytes: u32,
    pressure: PressureState,
) -> PolicyInput {
    PolicyInput {
        pending_chunk_ids: pending,
        link_stats: LinkStats {
            rtt_ms: 20,
            loss_ppm: 0,
        },
        device_class: DeviceClass::Desktop,
        constraints: TransferConstraints {
            max_parallel_chunks: max_parallel,
            max_in_flight_bytes: max_bytes,
            priority: 128,
            fairness_mode: FairnessMode::Balanced,
            configured_chunk_size: 16384,
            transport_max_message_size: 65536,
        },
        pressure,
    }
}

// ─── Determinism (migrated) ──────────────────────────────────────────

#[test]
fn determinism_identical_inputs_produce_identical_outputs() {
    let input = make_input(vec![10, 20, 30, 40, 50], 3, 65536, PressureState::Clear);
    let d1 = decide(&input);
    let d2 = decide(&input);
    assert_eq!(d1, d2, "identical inputs must produce identical decisions");
}

#[test]
fn determinism_ordering_preserved() {
    let input = make_input(vec![99, 42, 7, 1000, 3], 4, 65536, PressureState::Clear);
    let d1 = decide(&input);
    let d2 = decide(&input);
    assert_eq!(
        d1.next_chunk_ids, d2.next_chunk_ids,
        "chunk ID ordering must be deterministic"
    );
    assert_eq!(d1.next_chunk_ids, vec![99, 42, 7, 1000]);
}

#[test]
fn determinism_across_device_classes() {
    for class in [
        DeviceClass::Desktop,
        DeviceClass::Mobile,
        DeviceClass::LowPower,
        DeviceClass::Unknown,
    ] {
        let mut input = make_input(vec![1, 2, 3], 2, 65536, PressureState::Clear);
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
        let mut input = make_input(vec![1, 2, 3], 2, 65536, PressureState::Clear);
        input.constraints.fairness_mode = mode;
        let d1 = decide(&input);
        let d2 = decide(&input);
        assert_eq!(d1, d2, "determinism must hold for {:?}", mode);
    }
}

// ─── Bounds (migrated) ──────────────────────────────────────────────

#[test]
fn bound_next_chunk_ids_within_max_parallel() {
    let input = make_input(
        vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        4,
        65536,
        PressureState::Clear,
    );
    let d = decide(&input);
    assert!(
        d.next_chunk_ids.len() <= 4,
        "next_chunk_ids.len() ({}) must be <= max_parallel_chunks (4)",
        d.next_chunk_ids.len()
    );
}

#[test]
fn bound_window_suggestion_within_max_parallel() {
    let input = make_input(vec![0, 1, 2, 3, 4, 5], 3, 65536, PressureState::Clear);
    let d = decide(&input);
    assert!(
        d.window_suggestion_chunks <= 3,
        "window_suggestion_chunks ({}) must be <= max_parallel_chunks (3)",
        d.window_suggestion_chunks
    );
}

#[test]
fn bound_pacing_delay_within_max() {
    let input = make_input(vec![0, 1, 2], 3, 65536, PressureState::Clear);
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
    let input = make_input(vec![0, 1, 2, 3], 1, 65536, PressureState::Clear);
    let d = decide(&input);
    assert!(d.next_chunk_ids.len() <= 1);
    assert!(d.window_suggestion_chunks <= 1);
}

#[test]
fn bound_max_parallel_zero() {
    let input = make_input(vec![0, 1, 2], 0, 65536, PressureState::Clear);
    let d = decide(&input);
    assert!(d.next_chunk_ids.is_empty());
    assert_eq!(d.window_suggestion_chunks, 0);
}

// ─── Backpressure (migrated + updated for PressureState) ────────────

#[test]
fn backpressure_pause_when_pressured() {
    let input = make_input(vec![0, 1, 2], 3, 1000, PressureState::Pressured);
    let d = decide(&input);
    assert_eq!(
        d.backpressure,
        Backpressure::Pause,
        "must pause when pressure is Pressured"
    );
    assert!(
        d.next_chunk_ids.is_empty(),
        "must not schedule chunks when pressured"
    );
}

#[test]
fn backpressure_no_pause_when_clear() {
    let input = make_input(vec![0, 1], 2, 1000, PressureState::Clear);
    let d = decide(&input);
    assert_ne!(d.backpressure, Backpressure::Pause);
}

#[test]
fn backpressure_no_pause_when_elevated() {
    let input = make_input(vec![0, 1], 2, 1000, PressureState::Elevated);
    let d = decide(&input);
    assert_ne!(
        d.backpressure,
        Backpressure::Pause,
        "elevated should reduce window, not pause"
    );
}

// ─── Sanity (migrated) ─────────────────────────────────────────────

#[test]
fn sanity_empty_pending_empty_result() {
    let input = make_input(vec![], 10, 65536, PressureState::Clear);
    let d = decide(&input);
    assert!(d.next_chunk_ids.is_empty());
}

#[test]
fn sanity_single_chunk() {
    let input = make_input(vec![42], 10, 65536, PressureState::Clear);
    let d = decide(&input);
    assert_eq!(d.next_chunk_ids, vec![42]);
}

#[test]
fn sanity_max_pacing_constant_is_reasonable() {
    assert_eq!(MAX_PACING_DELAY_MS, 5_000);
}

// ─── S2A: Effective Chunk Cap (AC-S2A-02) ───────────────────────────

#[test]
fn effective_chunk_caps_to_transport_max() {
    let mut input = make_input(vec![0, 1], 2, 65536, PressureState::Clear);
    input.constraints.configured_chunk_size = 16384;
    input.constraints.transport_max_message_size = 8192;
    let d = decide(&input);
    assert_eq!(
        d.effective_chunk_size, 8192,
        "effective chunk size must be min(configured, transport_max)"
    );
}

#[test]
fn effective_chunk_uses_configured_when_smaller() {
    let mut input = make_input(vec![0, 1], 2, 65536, PressureState::Clear);
    input.constraints.configured_chunk_size = 4096;
    input.constraints.transport_max_message_size = 65536;
    let d = decide(&input);
    assert_eq!(d.effective_chunk_size, 4096);
}

#[test]
fn effective_chunk_equal_values() {
    let mut input = make_input(vec![0], 1, 65536, PressureState::Clear);
    input.constraints.configured_chunk_size = 16384;
    input.constraints.transport_max_message_size = 16384;
    let d = decide(&input);
    assert_eq!(d.effective_chunk_size, 16384);
}

// ─── S2A: Window Sizing (AC-S2A-03) ────────────────────────────────

#[test]
fn window_shrinks_under_elevated_pressure() {
    let clear = make_input(vec![0, 1, 2, 3], 4, 65536, PressureState::Clear);
    let elevated = make_input(vec![0, 1, 2, 3], 4, 65536, PressureState::Elevated);
    let dc = decide(&clear);
    let de = decide(&elevated);
    assert!(
        de.window_suggestion_chunks < dc.window_suggestion_chunks,
        "elevated ({}) must produce smaller window than clear ({})",
        de.window_suggestion_chunks,
        dc.window_suggestion_chunks
    );
}

#[test]
fn window_full_when_clear() {
    let input = make_input(vec![0, 1, 2, 3], 4, 65536, PressureState::Clear);
    let d = decide(&input);
    assert_eq!(d.window_suggestion_chunks, 4);
}

#[test]
fn latency_mode_minimizes_window() {
    let mut input = make_input(vec![0, 1, 2, 3], 4, 65536, PressureState::Clear);
    input.constraints.fairness_mode = FairnessMode::Latency;
    let d = decide(&input);
    assert_eq!(d.window_suggestion_chunks, 1);
}

#[test]
fn throughput_mode_maximizes_window() {
    let mut input = make_input(vec![0, 1, 2, 3], 4, 65536, PressureState::Clear);
    input.constraints.fairness_mode = FairnessMode::Throughput;
    let d = decide(&input);
    assert_eq!(d.window_suggestion_chunks, 4);
}

#[test]
fn low_power_reduces_window() {
    let mut clear = make_input(vec![0, 1, 2, 3], 4, 65536, PressureState::Clear);
    clear.device_class = DeviceClass::Desktop;
    let dd = decide(&clear);

    clear.device_class = DeviceClass::LowPower;
    let dl = decide(&clear);

    assert!(
        dl.window_suggestion_chunks < dd.window_suggestion_chunks,
        "LowPower ({}) must produce smaller window than Desktop ({})",
        dl.window_suggestion_chunks,
        dd.window_suggestion_chunks
    );
}

#[test]
fn mobile_reduces_window() {
    let mut input = make_input(vec![0, 1, 2, 3], 4, 65536, PressureState::Clear);
    input.device_class = DeviceClass::Desktop;
    let dd = decide(&input);

    input.device_class = DeviceClass::Mobile;
    let dm = decide(&input);

    assert!(
        dm.window_suggestion_chunks < dd.window_suggestion_chunks,
        "Mobile ({}) must produce smaller window than Desktop ({})",
        dm.window_suggestion_chunks,
        dd.window_suggestion_chunks
    );
}

// ─── S2A: Pacing (substantive) ─────────────────────────────────────

#[test]
fn elevated_introduces_pacing() {
    let input = make_input(vec![0, 1, 2], 3, 65536, PressureState::Elevated);
    let d = decide(&input);
    assert!(
        d.pacing_delay_ms > 0,
        "elevated pressure must introduce pacing"
    );
}

#[test]
fn clear_no_pacing_in_throughput_mode() {
    let mut input = make_input(vec![0, 1, 2], 3, 65536, PressureState::Clear);
    input.constraints.fairness_mode = FairnessMode::Throughput;
    let d = decide(&input);
    assert_eq!(d.pacing_delay_ms, 0);
}

#[test]
fn latency_mode_rtt_proportional_pacing() {
    let mut low_rtt = make_input(vec![0], 1, 65536, PressureState::Clear);
    low_rtt.constraints.fairness_mode = FairnessMode::Latency;
    low_rtt.link_stats.rtt_ms = 10;

    let mut high_rtt = low_rtt.clone();
    high_rtt.link_stats.rtt_ms = 200;

    let dl = decide(&low_rtt);
    let dh = decide(&high_rtt);

    assert!(
        dh.pacing_delay_ms > dl.pacing_delay_ms,
        "higher RTT ({}) must produce more pacing than lower RTT ({})",
        dh.pacing_delay_ms,
        dl.pacing_delay_ms
    );
}

// ─── S2A: Stall Detection (AC-S2A-06) ──────────────────────────────

#[test]
fn stall_healthy() {
    let input = StallInput {
        bytes_acked: 5000,
        total_bytes: 10000,
        ms_since_progress: 100,
        stall_threshold_ms: 10000,
        warn_threshold_ms: 5000,
    };
    assert_eq!(detect_stall(&input), StallClassification::Healthy);
}

#[test]
fn stall_warning() {
    let input = StallInput {
        bytes_acked: 5000,
        total_bytes: 10000,
        ms_since_progress: 5000,
        stall_threshold_ms: 10000,
        warn_threshold_ms: 5000,
    };
    assert!(matches!(
        detect_stall(&input),
        StallClassification::Warning { .. }
    ));
}

#[test]
fn stall_detected() {
    let input = StallInput {
        bytes_acked: 0,
        total_bytes: 10000,
        ms_since_progress: 10000,
        stall_threshold_ms: 10000,
        warn_threshold_ms: 5000,
    };
    match detect_stall(&input) {
        StallClassification::Stalled {
            ms_since_progress: ms,
        } => assert_eq!(ms, 10000),
        other => panic!("expected Stalled, got {:?}", other),
    }
}

#[test]
fn stall_complete_overrides() {
    let input = StallInput {
        bytes_acked: 10000,
        total_bytes: 10000,
        ms_since_progress: 999_999,
        stall_threshold_ms: 1000,
        warn_threshold_ms: 500,
    };
    assert_eq!(detect_stall(&input), StallClassification::Complete);
}

// ─── S2A: Progress Cadence (AC-S2A-07) ─────────────────────────────

#[test]
fn progress_emits_when_thresholds_met() {
    let config = ProgressConfig::default();
    let r = progress_cadence(5000, 10000, 300, 0, &config);
    assert!(r.is_some());
    assert_eq!(r.unwrap().percent, 50);
}

#[test]
fn progress_suppressed_too_soon() {
    let config = ProgressConfig::default();
    let r = progress_cadence(5000, 10000, 100, 0, &config);
    assert!(r.is_none());
}

#[test]
fn progress_suppressed_small_delta() {
    let config = ProgressConfig {
        min_interval_ms: 0,
        min_percent_delta: 10,
    };
    let r = progress_cadence(5500, 10000, 1000, 50, &config);
    assert!(r.is_none());
}

#[test]
fn progress_emits_at_completion() {
    let config = ProgressConfig::default();
    let r = progress_cadence(10000, 10000, 300, 99, &config);
    assert!(r.is_some());
    assert_eq!(r.unwrap().percent, 100);
}

// ─── S2A: No Dual-Path (AC-S2A-04) ─────────────────────────────────

#[test]
fn unified_backpressure_signal_type() {
    // Verify that BackpressureController and policy use the same Backpressure type.
    // This is a compile-time guarantee — if this test compiles, the types are unified.
    use bolt_transfer_core::backpressure::BackpressureController;
    use bolt_transfer_core::policy::types::Backpressure as PolicyBackpressure;

    let config = bolt_transfer_core::backpressure::BackpressureConfig::default();
    let mut ctrl = BackpressureController::new(config);

    struct MockTx;
    impl bolt_transfer_core::TransportQuery for MockTx {
        fn is_open(&self) -> bool {
            true
        }
        fn buffered_bytes(&self) -> usize {
            100_000
        }
        fn max_message_size(&self) -> usize {
            65536
        }
    }

    let signal: PolicyBackpressure = ctrl.evaluate(&MockTx);
    assert_eq!(signal, PolicyBackpressure::Pause);
}
