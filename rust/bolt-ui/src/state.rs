use std::time::{Duration, Instant};

/// Runtime state model for bolt-ui screens.
/// AC-EN-14: consumes bolt_core (shared Rust core), no transport/protocol logic.

/// Connect timeout — prevents infinite "Connecting..." hang.
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Connection lifecycle state.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Variants constructed by daemon IPC path
pub enum ConnectionState {
    Disconnected,
    Connecting { started_at: Instant },
    Connected { remote_peer_code: String },
    TimedOut,
    Error(String),
}

impl ConnectionState {
    pub fn status_text(&self) -> &str {
        match self {
            Self::Disconnected => "Not connected",
            Self::Connecting { .. } => "Connecting\u{2026}",
            Self::Connected { .. } => "Connected",
            Self::TimedOut => "Connection timed out",
            Self::Error(_) => "Connection failed",
        }
    }

    #[allow(dead_code)]
    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected { .. })
    }

    /// Check if a Connecting state has exceeded the timeout.
    /// Returns true if timed out (caller should transition to TimedOut).
    pub fn is_timed_out(&self) -> bool {
        match self {
            Self::Connecting { started_at } => started_at.elapsed() >= CONNECT_TIMEOUT,
            _ => false,
        }
    }

    /// Returns true if user can retry (from error/timeout/disconnected).
    pub fn can_retry(&self) -> bool {
        matches!(
            self,
            Self::Disconnected | Self::TimedOut | Self::Error(_)
        )
    }

    /// Returns true if user can cancel (from connecting).
    pub fn can_cancel(&self) -> bool {
        matches!(self, Self::Connecting { .. })
    }
}

/// Transfer lifecycle state.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // Variants constructed by daemon IPC path
pub enum TransferState {
    Idle,
    Sending { file_name: String, progress: f32 },
    Receiving { file_name: String, progress: f32 },
    Complete { file_name: String },
    Failed { file_name: String, reason: String },
}

impl TransferState {
    pub fn status_text(&self) -> String {
        match self {
            Self::Idle => "No active transfer".to_string(),
            Self::Sending { file_name, progress } => {
                format!("Sending {} ({:.0}%)", file_name, progress * 100.0)
            }
            Self::Receiving { file_name, progress } => {
                format!("Receiving {} ({:.0}%)", file_name, progress * 100.0)
            }
            Self::Complete { file_name } => format!("{} — complete", file_name),
            Self::Failed { file_name, reason } => {
                format!("{} — failed: {}", file_name, reason)
            }
        }
    }

    pub fn progress(&self) -> f32 {
        match self {
            Self::Sending { progress, .. } | Self::Receiving { progress, .. } => *progress,
            Self::Complete { .. } => 1.0,
            _ => 0.0,
        }
    }
}

/// Verification lifecycle state.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Variants constructed by daemon IPC path
pub enum VerifyState {
    NotStarted,
    Pending { sas_code: String },
    Confirmed,
    Rejected,
}

impl VerifyState {
    pub fn status_text(&self) -> &str {
        match self {
            Self::NotStarted => "No active session to verify",
            Self::Pending { .. } => "Compare this code with your peer",
            Self::Confirmed => "Verified — session authenticated",
            Self::Rejected => "Rejected — session terminated",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn connection_state_transitions() {
        let state = ConnectionState::Disconnected;
        assert_eq!(state.status_text(), "Not connected");
        assert!(state.can_retry());
        assert!(!state.can_cancel());

        let state = ConnectionState::Connecting {
            started_at: Instant::now(),
        };
        assert_eq!(state.status_text(), "Connecting\u{2026}");
        assert!(!state.can_retry());
        assert!(state.can_cancel());

        let state = ConnectionState::Connected {
            remote_peer_code: "XYZ789".to_string(),
        };
        assert!(state.is_connected());
        assert_eq!(state.status_text(), "Connected");

        let state = ConnectionState::TimedOut;
        assert_eq!(state.status_text(), "Connection timed out");
        assert!(state.can_retry());

        let state = ConnectionState::Error("test".to_string());
        assert_eq!(state.status_text(), "Connection failed");
        assert!(state.can_retry());
    }

    #[test]
    fn connect_timeout_fires() {
        // Verify that a Connecting state created in the past is detected as timed out.
        let past = Instant::now() - CONNECT_TIMEOUT - Duration::from_millis(100);
        let state = ConnectionState::Connecting { started_at: past };
        assert!(state.is_timed_out());
    }

    #[test]
    fn connect_timeout_does_not_fire_early() {
        let state = ConnectionState::Connecting {
            started_at: Instant::now(),
        };
        assert!(!state.is_timed_out());
    }

    #[test]
    fn connect_timeout_real_elapsed() {
        // Use a very short "timeout" simulation to test real elapsed time.
        let started = Instant::now();
        thread::sleep(Duration::from_millis(10));
        let state = ConnectionState::Connecting { started_at: started };
        // 10ms < 5s, so should not be timed out
        assert!(!state.is_timed_out());
    }

    #[test]
    fn disconnect_cancels_connecting() {
        // Simulate cancel: Connecting -> Disconnected
        let mut state = ConnectionState::Connecting {
            started_at: Instant::now(),
        };
        assert!(state.can_cancel());
        state = ConnectionState::Disconnected;
        assert!(state.can_retry());
        assert_eq!(state.status_text(), "Not connected");
    }

    #[test]
    fn retry_from_error() {
        let state = ConnectionState::Error("daemon unavailable".to_string());
        assert!(state.can_retry());
        // After retry, would go back to Connecting
    }

    #[test]
    fn retry_from_timeout() {
        let state = ConnectionState::TimedOut;
        assert!(state.can_retry());
    }

    #[test]
    fn transfer_state_progress() {
        assert_eq!(TransferState::Idle.progress(), 0.0);

        let state = TransferState::Sending {
            file_name: "test.bin".to_string(),
            progress: 0.5,
        };
        assert_eq!(state.progress(), 0.5);
        assert!(state.status_text().contains("50%"));

        let state = TransferState::Complete {
            file_name: "test.bin".to_string(),
        };
        assert_eq!(state.progress(), 1.0);
    }

    #[test]
    fn verify_state_lifecycle() {
        assert_eq!(
            VerifyState::NotStarted.status_text(),
            "No active session to verify"
        );
        assert_eq!(
            VerifyState::Confirmed.status_text(),
            "Verified — session authenticated"
        );
        assert_eq!(
            VerifyState::Rejected.status_text(),
            "Rejected — session terminated"
        );
    }

    #[test]
    fn no_placeholder_strings() {
        let conn = ConnectionState::Disconnected;
        assert!(!conn.status_text().contains("ABC123"));

        let verify = VerifyState::NotStarted;
        assert!(!verify.status_text().contains("A3 F7 2B"));
    }
}
