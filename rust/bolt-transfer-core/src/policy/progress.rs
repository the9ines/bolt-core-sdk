//! Progress cadence — pure function for progress event emission.
//!
//! Forward-investment for T-STREAM-1. No daemon consumer in S2A.
//! Pure function + tests only.

/// Configuration for progress cadence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressConfig {
    /// Minimum milliseconds between progress reports.
    pub min_interval_ms: u64,
    /// Minimum percentage change to trigger a report (0-100).
    pub min_percent_delta: u8,
}

impl Default for ProgressConfig {
    fn default() -> Self {
        Self {
            min_interval_ms: 250,
            min_percent_delta: 1,
        }
    }
}

/// A progress report to emit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressReport {
    /// Percentage complete (0-100).
    pub percent: u8,
    /// Bytes transferred so far.
    pub bytes_transferred: u64,
    /// Total bytes in transfer.
    pub total_bytes: u64,
}

/// Determine whether a progress event should be emitted.
///
/// Pure function. Returns `Some(ProgressReport)` when both the time interval
/// and percentage delta thresholds are met. Returns `None` otherwise.
///
/// # Parameters
///
/// - `bytes_transferred`: bytes sent/received so far.
/// - `total_bytes`: total transfer size. Must be > 0.
/// - `elapsed_since_last_report_ms`: ms since the last emitted report.
/// - `last_reported_percent`: the percent value of the last emitted report (0 if none).
/// - `config`: cadence configuration.
pub fn progress_cadence(
    bytes_transferred: u64,
    total_bytes: u64,
    elapsed_since_last_report_ms: u64,
    last_reported_percent: u8,
    config: &ProgressConfig,
) -> Option<ProgressReport> {
    if total_bytes == 0 {
        return None;
    }

    let percent = std::cmp::min(
        ((bytes_transferred as u128 * 100) / total_bytes as u128) as u8,
        100,
    );

    let delta = percent.saturating_sub(last_reported_percent);

    if elapsed_since_last_report_ms >= config.min_interval_ms && delta >= config.min_percent_delta {
        Some(ProgressReport {
            percent,
            bytes_transferred,
            total_bytes,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_when_thresholds_met() {
        let config = ProgressConfig::default();
        let report = progress_cadence(5000, 10000, 300, 0, &config);
        assert!(report.is_some());
        let r = report.unwrap();
        assert_eq!(r.percent, 50);
        assert_eq!(r.bytes_transferred, 5000);
        assert_eq!(r.total_bytes, 10000);
    }

    #[test]
    fn suppressed_when_too_soon() {
        let config = ProgressConfig::default();
        // Only 100ms elapsed, need 250ms
        let report = progress_cadence(5000, 10000, 100, 0, &config);
        assert!(report.is_none());
    }

    #[test]
    fn suppressed_when_delta_too_small() {
        let config = ProgressConfig {
            min_interval_ms: 0,
            min_percent_delta: 10,
        };
        // Only 5% change (50 → 55)
        let report = progress_cadence(5500, 10000, 1000, 50, &config);
        assert!(report.is_none());
    }

    #[test]
    fn emits_at_completion() {
        let config = ProgressConfig::default();
        let report = progress_cadence(10000, 10000, 300, 99, &config);
        assert!(report.is_some());
        assert_eq!(report.unwrap().percent, 100);
    }

    #[test]
    fn zero_total_returns_none() {
        let config = ProgressConfig::default();
        let report = progress_cadence(0, 0, 1000, 0, &config);
        assert!(report.is_none());
    }

    #[test]
    fn initial_report_at_zero_percent() {
        let config = ProgressConfig {
            min_interval_ms: 0,
            min_percent_delta: 1,
        };
        // 1% done, last reported 0
        let report = progress_cadence(100, 10000, 0, 0, &config);
        assert!(report.is_some());
        assert_eq!(report.unwrap().percent, 1);
    }

    #[test]
    fn no_report_when_no_delta() {
        let config = ProgressConfig::default();
        // Same percentage as last report
        let report = progress_cadence(5000, 10000, 500, 50, &config);
        assert!(report.is_none());
    }

    #[test]
    fn percent_caps_at_100() {
        let config = ProgressConfig {
            min_interval_ms: 0,
            min_percent_delta: 0,
        };
        // Over-acked
        let report = progress_cadence(20000, 10000, 1000, 99, &config);
        assert!(report.is_some());
        assert_eq!(report.unwrap().percent, 100);
    }
}
