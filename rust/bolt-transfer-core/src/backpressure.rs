//! Backpressure — high/low watermark pause-resume behavior (AC-TC-05).
//!
//! Provides basic flow control using a watermark model:
//! - When buffered bytes exceed `high_watermark`, signal Pause.
//! - When buffered bytes drop below `low_watermark`, signal Resume.
//! - Otherwise, no change.
//!
//! # S2A Unified Authority
//!
//! This controller is the SOLE evaluator of transport pressure. Its output
//! feeds into `PolicyInput::pressure` as a `PressureState`. The policy
//! module emits the final `Backpressure` signal — no dual-path overlap.

use crate::policy::types::{Backpressure, PressureState};
use crate::transport::TransportQuery;

/// Configuration for watermark-based backpressure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackpressureConfig {
    /// Bytes buffered above this threshold trigger Pause.
    pub high_watermark: usize,
    /// Bytes buffered below this threshold trigger Resume (after pause).
    pub low_watermark: usize,
}

impl BackpressureConfig {
    /// Create a new config. Panics if low >= high (invalid).
    pub fn new(high_watermark: usize, low_watermark: usize) -> Self {
        assert!(
            low_watermark < high_watermark,
            "low_watermark must be less than high_watermark"
        );
        Self {
            high_watermark,
            low_watermark,
        }
    }
}

/// Default config: 64 KiB high, 16 KiB low.
impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            high_watermark: 65_536,
            low_watermark: 16_384,
        }
    }
}

/// Backpressure controller — evaluates transport state against watermarks.
///
/// Single authoritative source for pressure evaluation. Callers use
/// [`pressure_state()`](Self::pressure_state) to obtain a [`PressureState`]
/// for [`PolicyInput`](crate::policy::types::PolicyInput).
#[derive(Debug)]
pub struct BackpressureController {
    config: BackpressureConfig,
    is_paused: bool,
}

impl BackpressureController {
    pub fn new(config: BackpressureConfig) -> Self {
        Self {
            config,
            is_paused: false,
        }
    }

    /// Evaluate backpressure based on current transport state.
    ///
    /// Returns the unified `Backpressure` signal.
    pub fn evaluate(&mut self, transport: &dyn TransportQuery) -> Backpressure {
        let buffered = transport.buffered_bytes();

        if !self.is_paused && buffered >= self.config.high_watermark {
            self.is_paused = true;
            Backpressure::Pause
        } else if self.is_paused && buffered <= self.config.low_watermark {
            self.is_paused = false;
            Backpressure::Resume
        } else {
            Backpressure::NoChange
        }
    }

    /// Current pressure state for use in `PolicyInput::pressure`.
    ///
    /// Maps the controller's internal state + transport observation
    /// to a `PressureState` suitable for the policy decision function.
    pub fn pressure_state(&self, transport: &dyn TransportQuery) -> PressureState {
        let buffered = transport.buffered_bytes();
        if self.is_paused {
            PressureState::Pressured
        } else if buffered >= self.config.low_watermark {
            PressureState::Elevated
        } else {
            PressureState::Clear
        }
    }

    /// Whether the controller is currently in paused state.
    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    /// Reset the controller to initial (unpaused) state.
    pub fn reset(&mut self) {
        self.is_paused = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockTransport {
        buffered: usize,
        open: bool,
    }

    impl TransportQuery for MockTransport {
        fn is_open(&self) -> bool {
            self.open
        }
        fn buffered_bytes(&self) -> usize {
            self.buffered
        }
        fn max_message_size(&self) -> usize {
            65_536
        }
    }

    fn transport(buffered: usize) -> MockTransport {
        MockTransport {
            buffered,
            open: true,
        }
    }

    #[test]
    fn below_high_watermark_no_change() {
        let mut ctrl = BackpressureController::new(BackpressureConfig::default());
        let sig = ctrl.evaluate(&transport(1_000));
        assert_eq!(sig, Backpressure::NoChange);
        assert!(!ctrl.is_paused());
    }

    #[test]
    fn at_high_watermark_pauses() {
        let mut ctrl = BackpressureController::new(BackpressureConfig::default());
        let sig = ctrl.evaluate(&transport(65_536));
        assert_eq!(sig, Backpressure::Pause);
        assert!(ctrl.is_paused());
    }

    #[test]
    fn above_high_watermark_pauses() {
        let mut ctrl = BackpressureController::new(BackpressureConfig::default());
        let sig = ctrl.evaluate(&transport(100_000));
        assert_eq!(sig, Backpressure::Pause);
        assert!(ctrl.is_paused());
    }

    #[test]
    fn paused_stays_paused_above_low() {
        let mut ctrl = BackpressureController::new(BackpressureConfig::default());
        ctrl.evaluate(&transport(100_000)); // pause
        let sig = ctrl.evaluate(&transport(30_000)); // between high and low
        assert_eq!(sig, Backpressure::NoChange);
        assert!(ctrl.is_paused());
    }

    #[test]
    fn paused_resumes_at_low_watermark() {
        let mut ctrl = BackpressureController::new(BackpressureConfig::default());
        ctrl.evaluate(&transport(100_000)); // pause
        let sig = ctrl.evaluate(&transport(16_384)); // at low
        assert_eq!(sig, Backpressure::Resume);
        assert!(!ctrl.is_paused());
    }

    #[test]
    fn paused_resumes_below_low_watermark() {
        let mut ctrl = BackpressureController::new(BackpressureConfig::default());
        ctrl.evaluate(&transport(100_000)); // pause
        let sig = ctrl.evaluate(&transport(0));
        assert_eq!(sig, Backpressure::Resume);
        assert!(!ctrl.is_paused());
    }

    #[test]
    fn resume_then_no_change_below_high() {
        let mut ctrl = BackpressureController::new(BackpressureConfig::default());
        ctrl.evaluate(&transport(100_000)); // pause
        ctrl.evaluate(&transport(0)); // resume
        let sig = ctrl.evaluate(&transport(10_000)); // below high
        assert_eq!(sig, Backpressure::NoChange);
        assert!(!ctrl.is_paused());
    }

    #[test]
    fn hysteresis_prevents_flapping() {
        let mut ctrl = BackpressureController::new(BackpressureConfig::new(100, 20));
        assert_eq!(ctrl.evaluate(&transport(100)), Backpressure::Pause);
        assert_eq!(ctrl.evaluate(&transport(50)), Backpressure::NoChange);
        assert_eq!(ctrl.evaluate(&transport(20)), Backpressure::Resume);
        assert_eq!(ctrl.evaluate(&transport(50)), Backpressure::NoChange);
    }

    #[test]
    fn reset_clears_paused() {
        let mut ctrl = BackpressureController::new(BackpressureConfig::default());
        ctrl.evaluate(&transport(100_000)); // pause
        assert!(ctrl.is_paused());
        ctrl.reset();
        assert!(!ctrl.is_paused());
    }

    #[test]
    fn custom_watermarks() {
        let config = BackpressureConfig::new(1_000, 200);
        let mut ctrl = BackpressureController::new(config);
        assert_eq!(ctrl.evaluate(&transport(999)), Backpressure::NoChange);
        assert_eq!(ctrl.evaluate(&transport(1_000)), Backpressure::Pause);
        assert_eq!(ctrl.evaluate(&transport(201)), Backpressure::NoChange);
        assert_eq!(ctrl.evaluate(&transport(200)), Backpressure::Resume);
    }

    #[test]
    #[should_panic(expected = "low_watermark must be less than high_watermark")]
    fn invalid_config_panics() {
        BackpressureConfig::new(100, 100);
    }

    // ── PressureState tests ──

    #[test]
    fn pressure_state_clear() {
        let ctrl = BackpressureController::new(BackpressureConfig::default());
        assert_eq!(ctrl.pressure_state(&transport(1_000)), PressureState::Clear);
    }

    #[test]
    fn pressure_state_elevated() {
        let ctrl = BackpressureController::new(BackpressureConfig::default());
        // At low_watermark (16384) → Elevated
        assert_eq!(
            ctrl.pressure_state(&transport(16_384)),
            PressureState::Elevated
        );
    }

    #[test]
    fn pressure_state_pressured_after_pause() {
        let mut ctrl = BackpressureController::new(BackpressureConfig::default());
        ctrl.evaluate(&transport(100_000)); // triggers pause
        assert_eq!(
            ctrl.pressure_state(&transport(50_000)),
            PressureState::Pressured
        );
    }
}
