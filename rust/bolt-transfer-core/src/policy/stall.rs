//! Stall detection — pure threshold-based classification.
//!
//! Decision only: no actions, no retries, no side effects.
//! Callers interpret the classification and take appropriate action.

/// Input to stall detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StallInput {
    /// Total bytes acknowledged so far in this transfer.
    pub bytes_acked: u64,
    /// Total bytes expected for this transfer.
    pub total_bytes: u64,
    /// Milliseconds since last forward progress (new bytes acked).
    pub ms_since_progress: u64,
    /// Stall threshold in milliseconds. No progress beyond this → Stalled.
    pub stall_threshold_ms: u64,
    /// Warning threshold in milliseconds. No progress beyond this → Warning.
    /// Must be less than stall_threshold_ms.
    pub warn_threshold_ms: u64,
}

/// Stall classification result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StallClassification {
    /// Transfer is making forward progress. No concern.
    Healthy,
    /// No progress for longer than warn threshold but less than stall threshold.
    Warning { ms_since_progress: u64 },
    /// No progress for longer than stall threshold.
    Stalled { ms_since_progress: u64 },
    /// Transfer is complete (bytes_acked >= total_bytes). No stall possible.
    Complete,
}

/// Classify the current transfer stall state.
///
/// Pure function. No side effects.
///
/// # Contract
///
/// - If `bytes_acked >= total_bytes`, always returns `Complete`.
/// - If `ms_since_progress >= stall_threshold_ms`, returns `Stalled`.
/// - If `ms_since_progress >= warn_threshold_ms`, returns `Warning`.
/// - Otherwise returns `Healthy`.
pub fn detect_stall(input: &StallInput) -> StallClassification {
    if input.bytes_acked >= input.total_bytes {
        return StallClassification::Complete;
    }

    if input.ms_since_progress >= input.stall_threshold_ms {
        return StallClassification::Stalled {
            ms_since_progress: input.ms_since_progress,
        };
    }

    if input.ms_since_progress >= input.warn_threshold_ms {
        return StallClassification::Warning {
            ms_since_progress: input.ms_since_progress,
        };
    }

    StallClassification::Healthy
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_input() -> StallInput {
        StallInput {
            bytes_acked: 1000,
            total_bytes: 10000,
            ms_since_progress: 0,
            stall_threshold_ms: 10_000,
            warn_threshold_ms: 5_000,
        }
    }

    #[test]
    fn healthy_when_recent_progress() {
        let input = base_input();
        assert_eq!(detect_stall(&input), StallClassification::Healthy);
    }

    #[test]
    fn warning_at_threshold() {
        let mut input = base_input();
        input.ms_since_progress = 5_000;
        assert!(matches!(
            detect_stall(&input),
            StallClassification::Warning {
                ms_since_progress: 5_000
            }
        ));
    }

    #[test]
    fn warning_between_thresholds() {
        let mut input = base_input();
        input.ms_since_progress = 7_500;
        assert!(matches!(
            detect_stall(&input),
            StallClassification::Warning {
                ms_since_progress: 7_500
            }
        ));
    }

    #[test]
    fn stalled_at_threshold() {
        let mut input = base_input();
        input.ms_since_progress = 10_000;
        assert!(matches!(
            detect_stall(&input),
            StallClassification::Stalled {
                ms_since_progress: 10_000
            }
        ));
    }

    #[test]
    fn stalled_beyond_threshold() {
        let mut input = base_input();
        input.ms_since_progress = 30_000;
        assert!(matches!(
            detect_stall(&input),
            StallClassification::Stalled {
                ms_since_progress: 30_000
            }
        ));
    }

    #[test]
    fn complete_overrides_stall() {
        let mut input = base_input();
        input.bytes_acked = 10_000;
        input.total_bytes = 10_000;
        input.ms_since_progress = 999_999;
        assert_eq!(detect_stall(&input), StallClassification::Complete);
    }

    #[test]
    fn complete_when_over_acked() {
        let mut input = base_input();
        input.bytes_acked = 20_000;
        input.total_bytes = 10_000;
        assert_eq!(detect_stall(&input), StallClassification::Complete);
    }

    #[test]
    fn zero_progress_zero_threshold_stalls() {
        let mut input = base_input();
        input.ms_since_progress = 0;
        input.stall_threshold_ms = 0;
        input.warn_threshold_ms = 0;
        assert!(matches!(
            detect_stall(&input),
            StallClassification::Stalled { .. }
        ));
    }
}
