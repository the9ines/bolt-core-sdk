//! Transfer policy implementation — deterministic scheduling decisions.
//!
//! The policy function is pure: identical inputs produce identical outputs.
//! No IO, no clocks, no global state.

use super::types::{
    Backpressure, DeviceClass, FairnessMode, PolicyInput, PressureState, ScheduleDecision,
    MAX_PACING_DELAY_MS,
};

/// Base pacing delay (ms) per fairness mode when pressure is clear.
const PACING_CLEAR_MS: u32 = 0;
/// Pacing delay (ms) under elevated pressure.
const PACING_ELEVATED_MS: u32 = 25;
/// Pacing delay (ms) under full pressure (before Pause takes effect).
const PACING_PRESSURED_MS: u32 = 0; // Pressured → Pause, no pacing needed

/// Window fraction to use under elevated pressure (numerator/denominator).
const ELEVATED_WINDOW_NUM: u16 = 1;
const ELEVATED_WINDOW_DEN: u16 = 2;

/// Compute a scheduling decision from the given input.
///
/// # Contract
///
/// - **Deterministic**: identical `input` always produces identical output
///   (including `next_chunk_ids` ordering).
/// - **Bounded**: `next_chunk_ids.len() <= input.constraints.max_parallel_chunks`.
/// - **Bounded**: `window_suggestion_chunks <= input.constraints.max_parallel_chunks`.
/// - **Bounded**: `pacing_delay_ms <= MAX_PACING_DELAY_MS`.
/// - **Backpressure**: if `input.pressure == Pressured`, backpressure MUST be
///   `Pause` (no new chunks scheduled).
/// - **Effective chunk size**: `effective_chunk_size = min(configured_chunk_size,
///   transport_max_message_size)`. Caps effective chunk size used for
///   scheduling/sending decisions.
pub fn decide(input: &PolicyInput) -> ScheduleDecision {
    let effective_chunk_size = std::cmp::min(
        input.constraints.configured_chunk_size,
        input.constraints.transport_max_message_size,
    );

    // Contract: if pressured, pause and schedule nothing.
    if input.pressure == PressureState::Pressured {
        return ScheduleDecision {
            next_chunk_ids: Vec::new(),
            pacing_delay_ms: PACING_PRESSURED_MS,
            window_suggestion_chunks: 0,
            backpressure: Backpressure::Pause,
            effective_chunk_size,
        };
    }

    let max_parallel = input.constraints.max_parallel_chunks;

    // Compute effective window based on pressure and fairness mode.
    let base_window = match input.pressure {
        PressureState::Elevated => {
            // Halve window under elevated pressure.
            let reduced =
                (max_parallel as u32 * ELEVATED_WINDOW_NUM as u32) / ELEVATED_WINDOW_DEN as u32;
            std::cmp::max(reduced as u16, 1.min(max_parallel))
        }
        PressureState::Clear => max_parallel,
        PressureState::Pressured => unreachable!(),
    };

    // Apply fairness mode adjustments.
    let adjusted_window = match input.constraints.fairness_mode {
        FairnessMode::Latency => {
            // Latency mode: cap window to 1 (minimize in-flight).
            1u16.min(base_window)
        }
        FairnessMode::Throughput => base_window,
        FairnessMode::Balanced => base_window,
    };

    // Apply device class adjustments.
    let device_window = match input.device_class {
        DeviceClass::LowPower => {
            // LowPower: cap to half of adjusted, minimum 1.
            let reduced = adjusted_window / 2;
            std::cmp::max(reduced, 1.min(adjusted_window))
        }
        DeviceClass::Mobile => {
            // Mobile: reduce by 25%, minimum 1.
            let reduced = adjusted_window * 3 / 4;
            std::cmp::max(reduced, 1.min(adjusted_window))
        }
        DeviceClass::Desktop | DeviceClass::Unknown => adjusted_window,
    };

    // Schedule chunks up to the effective window.
    let limit = device_window as usize;
    let next_chunk_ids: Vec<u32> = input
        .pending_chunk_ids
        .iter()
        .take(limit)
        .copied()
        .collect();

    let window = next_chunk_ids.len() as u16;

    // Compute pacing delay.
    let base_pacing = match input.pressure {
        PressureState::Elevated => PACING_ELEVATED_MS,
        PressureState::Clear => PACING_CLEAR_MS,
        PressureState::Pressured => unreachable!(),
    };

    // Latency mode adds RTT-proportional pacing.
    let pacing = match input.constraints.fairness_mode {
        FairnessMode::Latency => {
            let rtt_pacing = input.link_stats.rtt_ms / 4;
            std::cmp::min(base_pacing + rtt_pacing, MAX_PACING_DELAY_MS)
        }
        FairnessMode::Balanced => {
            // Small RTT-aware bump when elevated.
            if input.pressure == PressureState::Elevated {
                let rtt_bump = input.link_stats.rtt_ms / 8;
                std::cmp::min(base_pacing + rtt_bump, MAX_PACING_DELAY_MS)
            } else {
                base_pacing
            }
        }
        FairnessMode::Throughput => base_pacing,
    };

    // Backpressure output: Resume if we were pressured and now cleared.
    // Clear → NoChange (caller wasn't paused).
    // Elevated → NoChange (not yet at pause threshold).
    let backpressure = Backpressure::NoChange;

    ScheduleDecision {
        next_chunk_ids,
        pacing_delay_ms: pacing,
        window_suggestion_chunks: window,
        backpressure,
        effective_chunk_size,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::types::{DeviceClass, LinkStats, TransferConstraints};

    fn default_input() -> PolicyInput {
        PolicyInput {
            pending_chunk_ids: vec![0, 1, 2, 3, 4],
            link_stats: LinkStats {
                rtt_ms: 10,
                loss_ppm: 0,
            },
            device_class: DeviceClass::Desktop,
            constraints: TransferConstraints {
                max_parallel_chunks: 4,
                max_in_flight_bytes: 65536,
                priority: 128,
                fairness_mode: FairnessMode::Balanced,
                configured_chunk_size: 16384,
                transport_max_message_size: 65536,
            },
            pressure: PressureState::Clear,
        }
    }

    #[test]
    fn clear_pressure_schedules_full_window() {
        let input = default_input();
        let d = decide(&input);
        assert_eq!(d.next_chunk_ids, vec![0, 1, 2, 3]);
        assert_eq!(d.window_suggestion_chunks, 4);
        assert_eq!(d.backpressure, Backpressure::NoChange);
        assert_eq!(d.pacing_delay_ms, 0);
    }

    #[test]
    fn pressured_pauses_and_schedules_nothing() {
        let mut input = default_input();
        input.pressure = PressureState::Pressured;
        let d = decide(&input);
        assert!(d.next_chunk_ids.is_empty());
        assert_eq!(d.window_suggestion_chunks, 0);
        assert_eq!(d.backpressure, Backpressure::Pause);
    }

    #[test]
    fn elevated_halves_window() {
        let mut input = default_input();
        input.pressure = PressureState::Elevated;
        let d = decide(&input);
        // 4 / 2 = 2
        assert_eq!(d.window_suggestion_chunks, 2);
        assert_eq!(d.next_chunk_ids, vec![0, 1]);
        assert!(d.pacing_delay_ms > 0);
    }

    #[test]
    fn effective_chunk_caps_to_transport_max() {
        let mut input = default_input();
        input.constraints.configured_chunk_size = 16384;
        input.constraints.transport_max_message_size = 8192;
        let d = decide(&input);
        assert_eq!(d.effective_chunk_size, 8192);
    }

    #[test]
    fn effective_chunk_uses_configured_when_smaller() {
        let mut input = default_input();
        input.constraints.configured_chunk_size = 4096;
        input.constraints.transport_max_message_size = 65536;
        let d = decide(&input);
        assert_eq!(d.effective_chunk_size, 4096);
    }

    #[test]
    fn latency_mode_caps_window_to_one() {
        let mut input = default_input();
        input.constraints.fairness_mode = FairnessMode::Latency;
        let d = decide(&input);
        assert_eq!(d.window_suggestion_chunks, 1);
        assert_eq!(d.next_chunk_ids, vec![0]);
    }

    #[test]
    fn latency_mode_adds_rtt_pacing() {
        let mut input = default_input();
        input.constraints.fairness_mode = FairnessMode::Latency;
        input.link_stats.rtt_ms = 100;
        let d = decide(&input);
        // base 0 + rtt/4 = 25
        assert_eq!(d.pacing_delay_ms, 25);
    }

    #[test]
    fn throughput_mode_full_window_no_pacing() {
        let mut input = default_input();
        input.constraints.fairness_mode = FairnessMode::Throughput;
        let d = decide(&input);
        assert_eq!(d.window_suggestion_chunks, 4);
        assert_eq!(d.pacing_delay_ms, 0);
    }

    #[test]
    fn low_power_device_halves_window() {
        let mut input = default_input();
        input.device_class = DeviceClass::LowPower;
        let d = decide(&input);
        // 4 / 2 = 2
        assert_eq!(d.window_suggestion_chunks, 2);
    }

    #[test]
    fn mobile_device_reduces_window() {
        let mut input = default_input();
        input.device_class = DeviceClass::Mobile;
        let d = decide(&input);
        // 4 * 3/4 = 3
        assert_eq!(d.window_suggestion_chunks, 3);
    }

    #[test]
    fn empty_pending_returns_empty() {
        let mut input = default_input();
        input.pending_chunk_ids = vec![];
        let d = decide(&input);
        assert!(d.next_chunk_ids.is_empty());
        assert_eq!(d.window_suggestion_chunks, 0);
    }

    #[test]
    fn max_parallel_zero() {
        let mut input = default_input();
        input.constraints.max_parallel_chunks = 0;
        let d = decide(&input);
        assert!(d.next_chunk_ids.is_empty());
        assert_eq!(d.window_suggestion_chunks, 0);
    }

    #[test]
    fn pacing_never_exceeds_max() {
        let mut input = default_input();
        input.constraints.fairness_mode = FairnessMode::Latency;
        input.link_stats.rtt_ms = 100_000; // Very high RTT
        let d = decide(&input);
        assert!(d.pacing_delay_ms <= MAX_PACING_DELAY_MS);
    }

    #[test]
    fn elevated_balanced_adds_rtt_bump() {
        let mut input = default_input();
        input.pressure = PressureState::Elevated;
        input.link_stats.rtt_ms = 80;
        let d = decide(&input);
        // base 25 + rtt/8 = 25 + 10 = 35
        assert_eq!(d.pacing_delay_ms, 35);
    }
}
