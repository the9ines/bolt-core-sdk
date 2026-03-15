//! IPC client for bolt-ui → daemon communication.
//! Connects to daemon Unix socket, performs NDJSON handshake, receives events.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};

const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);
const APP_VERSION: &str = "0.0.1";

/// IPC event received from daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcEvent {
    pub id: String,
    pub kind: String,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub ts_ms: u64,
    pub payload: serde_json::Value,
}

/// IPC client handle.
pub struct IpcClient {
    event_rx: mpsc::Receiver<IpcEvent>,
    _reader_thread: thread::JoinHandle<()>,
}

impl IpcClient {
    /// Connect to daemon IPC socket and perform version handshake.
    pub fn connect(socket_path: &str) -> Result<Self, String> {
        let stream = UnixStream::connect(socket_path)
            .map_err(|e| format!("IPC connect failed: {e}"))?;

        stream
            .set_read_timeout(Some(HANDSHAKE_TIMEOUT))
            .map_err(|e| format!("set timeout: {e}"))?;

        let mut writer = stream.try_clone().map_err(|e| format!("clone: {e}"))?;

        // Send version.handshake
        let handshake = serde_json::json!({
            "id": "app-0",
            "kind": "decision",
            "type": "version.handshake",
            "ts_ms": now_ms(),
            "payload": {"app_version": APP_VERSION}
        });
        let mut line = serde_json::to_string(&handshake).map_err(|e| format!("serialize: {e}"))?;
        line.push('\n');
        writer
            .write_all(line.as_bytes())
            .map_err(|e| format!("write handshake: {e}"))?;
        writer.flush().map_err(|e| format!("flush: {e}"))?;

        // Read version.status
        let mut reader = BufReader::new(stream);
        let mut buf = String::new();
        reader
            .read_line(&mut buf)
            .map_err(|e| format!("read version.status: {e}"))?;

        let status: IpcEvent =
            serde_json::from_str(buf.trim()).map_err(|e| format!("parse version.status: {e}"))?;

        if status.msg_type != "version.status" {
            return Err(format!("expected version.status, got {}", status.msg_type));
        }

        let compatible = status.payload.get("compatible").and_then(|v| v.as_bool()).unwrap_or(false);
        if !compatible {
            return Err("daemon version incompatible".into());
        }

        // Clear read timeout for event loop
        reader
            .get_ref()
            .set_read_timeout(None)
            .map_err(|e| format!("clear timeout: {e}"))?;

        // Start event reader thread
        let (tx, rx) = mpsc::channel();
        let reader_thread = thread::spawn(move || {
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        if let Ok(event) = serde_json::from_str::<IpcEvent>(line.trim()) {
                            if tx.send(event).is_err() {
                                break;
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            event_rx: rx,
            _reader_thread: reader_thread,
        })
    }

    /// Try to receive the next event (non-blocking).
    pub fn try_recv(&self) -> Option<IpcEvent> {
        self.event_rx.try_recv().ok()
    }

    /// Receive all pending events.
    pub fn drain_events(&self) -> Vec<IpcEvent> {
        let mut events = Vec::new();
        while let Some(e) = self.try_recv() {
            events.push(e);
        }
        events
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipc_event_deserialize() {
        let json = r#"{"id":"evt-0","kind":"event","type":"session.sas","ts_ms":123,"payload":{"sas":"123456","remote_identity_pk_b64":"abc"}}"#;
        let event: IpcEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.msg_type, "session.sas");
        assert_eq!(event.payload["sas"], "123456");
    }

    #[test]
    fn connect_to_nonexistent_socket_fails() {
        let result = IpcClient::connect("/tmp/nonexistent-bolt-ui-test.sock");
        assert!(result.is_err());
    }
}
