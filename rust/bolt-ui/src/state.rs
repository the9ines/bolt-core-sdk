/// Runtime state model for bolt-ui screens.
/// Replaces EN2 placeholder strings with typed state driven by core APIs.
/// AC-EN-14: consumes bolt_core (shared Rust core), no transport/protocol logic.

/// Connection lifecycle state.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Variants constructed by daemon IPC path
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected { remote_peer_code: String },
    Error(String),
}

impl ConnectionState {
    pub fn status_text(&self) -> &str {
        match self {
            Self::Disconnected => "Not connected",
            Self::Connecting => "Connecting\u{2026}",
            Self::Connected { .. } => "Connected",
            Self::Error(_) => "Connection failed",
        }
    }

    #[allow(dead_code)] // Used by daemon IPC connection guard
    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected { .. })
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

    #[test]
    fn connection_state_transitions() {
        let state = ConnectionState::Disconnected;
        assert_eq!(state.status_text(), "Not connected");
        assert!(!state.is_connected());

        let state = ConnectionState::Connecting;
        assert_eq!(state.status_text(), "Connecting\u{2026}");

        let state = ConnectionState::Connected {
            remote_peer_code: "XYZ789".to_string(),
        };
        assert!(state.is_connected());
        assert_eq!(state.status_text(), "Connected");

        let state = ConnectionState::Error("timeout".to_string());
        assert_eq!(state.status_text(), "Connection failed");
    }

    #[test]
    fn transfer_state_progress() {
        let state = TransferState::Idle;
        assert_eq!(state.progress(), 0.0);

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
        let state = VerifyState::NotStarted;
        assert_eq!(state.status_text(), "No active session to verify");

        let state = VerifyState::Pending {
            sas_code: "A3F72B".to_string(),
        };
        assert_eq!(state.status_text(), "Compare this code with your peer");

        assert_eq!(VerifyState::Confirmed.status_text(), "Verified — session authenticated");
        assert_eq!(VerifyState::Rejected.status_text(), "Rejected — session terminated");
    }

    #[test]
    fn no_placeholder_strings() {
        // Verify no hardcoded placeholder values exist in state defaults
        let conn = ConnectionState::Disconnected;
        assert!(!conn.status_text().contains("ABC123"));

        let verify = VerifyState::NotStarted;
        assert!(!verify.status_text().contains("A3 F7 2B"));
    }
}
