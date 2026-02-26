//! Transfer policy implementation — deterministic scheduling decisions.
//!
//! The policy function is pure: identical inputs produce identical outputs.
//! No IO, no clocks, no global state.

use super::types::{Backpressure, PolicyInput, ScheduleDecision};

/// Compute a scheduling decision from the given input.
///
/// # Contract
///
/// - **Deterministic**: identical `input` always produces identical output
///   (including `next_chunk_ids` ordering).
/// - **Bounded**: `next_chunk_ids.len() <= input.constraints.max_parallel_chunks`.
/// - **Bounded**: `window_suggestion_chunks <= input.constraints.max_parallel_chunks`.
/// - **Bounded**: `pacing_delay_ms <= MAX_PACING_DELAY_MS`.
/// - **Backpressure**: if `input.link_stats.in_flight_bytes > input.constraints.max_in_flight_bytes`,
///   backpressure MUST be `Pause` (no new chunks scheduled).
///
/// # Current Implementation
///
/// This is a deterministic stub. It satisfies all contracts but performs
/// no intelligent scheduling. A real implementation will replace this
/// in a future S2 sub-phase.
pub fn decide(input: &PolicyInput) -> ScheduleDecision {
    // Contract: if over budget, pause and schedule nothing.
    if input.link_stats.in_flight_bytes > input.constraints.max_in_flight_bytes {
        return ScheduleDecision {
            next_chunk_ids: Vec::new(),
            pacing_delay_ms: 0,
            window_suggestion_chunks: 0,
            backpressure: Backpressure::Pause,
        };
    }

    // Stub: schedule up to max_parallel_chunks from the pending list.
    let limit = input.constraints.max_parallel_chunks as usize;
    let next_chunk_ids: Vec<u32> = input
        .pending_chunk_ids
        .iter()
        .take(limit)
        .copied()
        .collect();

    let window = next_chunk_ids.len() as u16;

    ScheduleDecision {
        next_chunk_ids,
        pacing_delay_ms: 0,
        window_suggestion_chunks: window,
        backpressure: Backpressure::NoChange,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transfer_policy::types::{
        DeviceClass, FairnessMode, LinkStats, TransferConstraints, MAX_PACING_DELAY_MS,
    };

    fn default_input() -> PolicyInput {
        PolicyInput {
            pending_chunk_ids: vec![0, 1, 2, 3, 4],
            link_stats: LinkStats {
                rtt_ms: 10,
                loss_ppm: 0,
                in_flight_bytes: 0,
            },
            device_class: DeviceClass::Desktop,
            constraints: TransferConstraints {
                max_parallel_chunks: 3,
                max_in_flight_bytes: 65536,
                priority: 128,
                fairness_mode: FairnessMode::Balanced,
            },
        }
    }

    #[test]
    fn stub_returns_valid_decision() {
        let input = default_input();
        let decision = decide(&input);

        assert!(!decision.next_chunk_ids.is_empty());
        assert!(decision.next_chunk_ids.len() <= input.constraints.max_parallel_chunks as usize);
        assert!(decision.pacing_delay_ms <= MAX_PACING_DELAY_MS);
        assert!(decision.window_suggestion_chunks <= input.constraints.max_parallel_chunks);
        assert_eq!(decision.backpressure, Backpressure::NoChange);
    }

    #[test]
    fn stub_respects_parallel_chunk_limit() {
        let input = default_input();
        let decision = decide(&input);
        // 5 pending but max_parallel_chunks = 3
        assert_eq!(decision.next_chunk_ids.len(), 3);
        assert_eq!(decision.next_chunk_ids, vec![0, 1, 2]);
    }

    #[test]
    fn empty_pending_returns_empty_schedule() {
        let mut input = default_input();
        input.pending_chunk_ids = vec![];
        let decision = decide(&input);
        assert!(decision.next_chunk_ids.is_empty());
        assert_eq!(decision.window_suggestion_chunks, 0);
    }

    #[test]
    fn over_budget_triggers_pause() {
        let mut input = default_input();
        input.link_stats.in_flight_bytes = 100_000; // > 65536
        let decision = decide(&input);
        assert!(decision.next_chunk_ids.is_empty());
        assert_eq!(decision.backpressure, Backpressure::Pause);
    }
}
