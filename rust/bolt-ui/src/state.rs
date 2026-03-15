use std::time::{Duration, Instant};

/// Runtime state model for bolt-ui screens.

pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// Host/Join connection mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectMode {
    Host,
    Join,
}

/// Connection info displayed to host for sharing.
#[derive(Debug, Clone)]
pub struct HostInfo {
    pub peer_code: String,
    pub room: String,
    pub session: String,
}

impl HostInfo {
    pub fn share_string(&self) -> String {
        format!("{} / {} / {}", self.room, self.session, self.peer_code)
    }
}

/// Connection lifecycle state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Idle,
    Connecting { started_at: Instant },
    Connected,
    TimedOut,
    Error(String),
}

impl ConnectionState {
    pub fn status_text(&self) -> &str {
        match self {
            Self::Idle => "Ready",
            Self::Connecting { .. } => "Connecting\u{2026}",
            Self::Connected => "Connected",
            Self::TimedOut => "Connection timed out",
            Self::Error(_) => "Connection failed",
        }
    }

    pub fn is_timed_out(&self) -> bool {
        match self {
            Self::Connecting { started_at } => started_at.elapsed() >= CONNECT_TIMEOUT,
            _ => false,
        }
    }

    pub fn can_retry(&self) -> bool {
        matches!(self, Self::Idle | Self::TimedOut | Self::Error(_))
    }

    pub fn can_cancel(&self) -> bool {
        matches!(self, Self::Connecting { .. })
    }
}

/// Transfer lifecycle state.
#[derive(Debug, Clone, PartialEq)]
pub enum TransferState {
    Idle,
    Ready,
    #[allow(dead_code)]
    Sending { file_name: String, progress: f32 },
    #[allow(dead_code)]
    Receiving { file_name: String, progress: f32 },
    #[allow(dead_code)]
    Complete { file_name: String },
    #[allow(dead_code)]
    Failed { file_name: String, reason: String },
}

impl TransferState {
    pub fn status_text(&self) -> String {
        match self {
            Self::Idle => "No active transfer".to_string(),
            Self::Ready => "Connected — ready to transfer".to_string(),
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
pub enum VerifyState {
    NotStarted,
    #[allow(dead_code)]
    Pending { sas_code: String },
    #[allow(dead_code)]
    Confirmed,
    #[allow(dead_code)]
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

    #[test]
    fn connection_timeout() {
        let past = Instant::now() - CONNECT_TIMEOUT - Duration::from_millis(100);
        let state = ConnectionState::Connecting { started_at: past };
        assert!(state.is_timed_out());

        let state = ConnectionState::Connecting { started_at: Instant::now() };
        assert!(!state.is_timed_out());
    }

    #[test]
    fn host_info_share_string() {
        let info = HostInfo {
            peer_code: "ABC123".into(),
            room: "r1a2b3".into(),
            session: "s4d5e6".into(),
        };
        let s = info.share_string();
        assert!(s.contains("r1a2b3"));
        assert!(s.contains("s4d5e6"));
        assert!(s.contains("ABC123"));
    }

    #[test]
    fn cancel_retry_states() {
        assert!(ConnectionState::Idle.can_retry());
        assert!(ConnectionState::TimedOut.can_retry());
        assert!(ConnectionState::Error("x".into()).can_retry());
        assert!(!ConnectionState::Connected.can_retry());

        let connecting = ConnectionState::Connecting { started_at: Instant::now() };
        assert!(connecting.can_cancel());
        assert!(!ConnectionState::Idle.can_cancel());
    }

    #[test]
    fn transfer_ready_state() {
        let state = TransferState::Ready;
        assert!(state.status_text().contains("ready"));
    }
}
