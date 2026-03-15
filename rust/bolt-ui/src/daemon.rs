//! Daemon process lifecycle manager for bolt-ui.
//! Spawns bolt-daemon as a child process with rendezvous CLI args.
//! No daemon IPC API redesign — uses existing CLI contract.

use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

/// Daemon process handle with stderr capture.
pub struct DaemonProcess {
    child: Child,
    pid: u32,
    pub stderr_lines: Arc<Mutex<Vec<String>>>,
    _stderr_thread: thread::JoinHandle<()>,
}

/// Result of attempting to find the daemon binary.
pub fn find_daemon_binary() -> Result<PathBuf, String> {
    // 1. Check compile-time workspace path (works when running via cargo run)
    let workspace_release = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../bolt-daemon/target/release/bolt-daemon");
    if workspace_release.exists() {
        return Ok(workspace_release);
    }

    let workspace_debug = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../bolt-daemon/target/debug/bolt-daemon");
    if workspace_debug.exists() {
        return Ok(workspace_debug);
    }

    // 2. Check current working directory (where user launched from)
    let cwd_candidate = PathBuf::from("bolt-daemon");
    if cwd_candidate.exists() {
        return Ok(std::fs::canonicalize(&cwd_candidate).unwrap_or(cwd_candidate));
    }

    // 3. Check relative to current executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let sibling = exe_dir.join("bolt-daemon");
            if sibling.exists() {
                return Ok(sibling);
            }
            for ancestor in exe_dir.ancestors().skip(1).take(5) {
                let candidate = ancestor.join("bolt-daemon/target/release/bolt-daemon");
                if candidate.exists() {
                    return Ok(candidate);
                }
            }
        }
    }

    // 4. Check Desktop (common deployment location)
    let home = std::env::var("HOME").unwrap_or_default();
    let desktop_candidate = PathBuf::from(format!("{home}/Desktop/bolt-daemon"));
    if desktop_candidate.exists() {
        return Ok(desktop_candidate);
    }

    // 5. Check well-known ecosystem paths
    let ecosystem_paths = [
        format!("{home}/Desktop/the9ines.com/bolt-ecosystem/bolt-daemon/target/release/bolt-daemon"),
        format!("{home}/Projects/bolt-ecosystem/bolt-daemon/target/release/bolt-daemon"),
    ];
    for p in &ecosystem_paths {
        let path = PathBuf::from(p);
        if path.exists() {
            return Ok(path);
        }
    }

    // 4. Check PATH
    if let Ok(output) = Command::new("which").arg("bolt-daemon").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Ok(PathBuf::from(path));
            }
        }
    }

    Err("bolt-daemon binary not found. Build with: cd bolt-daemon && cargo build --release".into())
}

/// Check if rendezvous server is reachable (quick TCP probe).
pub fn probe_rendezvous(url: &str) -> bool {
    // Extract host:port from ws://host:port
    let addr = url
        .trim_start_matches("ws://")
        .trim_start_matches("wss://");
    std::net::TcpStream::connect_timeout(
        &addr.parse().unwrap_or_else(|_| "127.0.0.1:3001".parse().unwrap()),
        std::time::Duration::from_secs(2),
    )
    .is_ok()
}

/// Find the rendezvous server binary.
pub fn find_rendezvous_binary() -> Option<PathBuf> {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../bolt-rendezvous/target/release/bolt-rendezvous");
    if workspace.exists() {
        return Some(workspace);
    }
    None
}

impl DaemonProcess {
    /// Spawn daemon as Host (answerer).
    pub fn spawn_host(
        daemon_bin: &PathBuf,
        peer_id: &str,
        expect_peer: &str,
        room: &str,
        session: &str,
        socket_path: &str,
        data_dir: &str,
        rendezvous_url: &str,
    ) -> Result<Self, String> {
        let ws_url = format!("ws://{rendezvous_url}");
        Self::spawn(
            daemon_bin,
            &[
                "--role", "answerer",
                "--signal", "rendezvous",
                "--rendezvous-url", &ws_url,
                "--room", room,
                "--session", session,
                "--peer-id", peer_id,
                "--expect-peer", expect_peer,
                "--socket-path", socket_path,
                "--data-dir", data_dir,
                "--pairing-policy", "allow",
            ],
        )
    }

    /// Spawn daemon as Join (offerer).
    pub fn spawn_join(
        daemon_bin: &PathBuf,
        peer_id: &str,
        to_peer: &str,
        room: &str,
        session: &str,
        socket_path: &str,
        data_dir: &str,
        rendezvous_url: &str,
    ) -> Result<Self, String> {
        let ws_url = format!("ws://{rendezvous_url}");
        Self::spawn(
            daemon_bin,
            &[
                "--role", "offerer",
                "--signal", "rendezvous",
                "--rendezvous-url", &ws_url,
                "--room", room,
                "--session", session,
                "--peer-id", peer_id,
                "--to", to_peer,
                "--socket-path", socket_path,
                "--data-dir", data_dir,
                "--pairing-policy", "allow",
            ],
        )
    }

    fn spawn(daemon_bin: &PathBuf, args: &[&str]) -> Result<Self, String> {
        // Ensure data-dir has 0700 permissions (daemon enforces this)
        if let Some(idx) = args.iter().position(|a| *a == "--data-dir") {
            if let Some(dir) = args.get(idx + 1) {
                let _ = std::fs::create_dir_all(dir);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700));
                }
            }
        }

        let mut child = Command::new(daemon_bin)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn daemon: {e}"))?;

        let pid = child.id();
        let stderr_lines: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let lines_clone = Arc::clone(&stderr_lines);

        let stderr = child.stderr.take().ok_or("No stderr handle")?;
        let stderr_thread = thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                if let Ok(mut buf) = lines_clone.lock() {
                    buf.push(line);
                }
            }
        });

        Ok(Self {
            child,
            pid,
            stderr_lines,
            _stderr_thread: stderr_thread,
        })
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// Check if daemon is still running.
    pub fn is_running(&mut self) -> bool {
        self.child.try_wait().ok().flatten().is_none()
    }

    /// Get latest stderr lines (non-destructive peek).
    pub fn recent_stderr(&self, last_n: usize) -> Vec<String> {
        self.stderr_lines
            .lock()
            .map(|buf| {
                let start = buf.len().saturating_sub(last_n);
                buf[start..].to_vec()
            })
            .unwrap_or_default()
    }

    /// Check stderr for connection-established signals.
    pub fn has_connected(&self) -> bool {
        self.stderr_lines
            .lock()
            .map(|buf| {
                buf.iter().any(|l| {
                    l.contains("hello/ack complete")
                        || l.contains("connection state: Connected")
                        || l.contains("[DC_OPEN]")
                })
            })
            .unwrap_or(false)
    }

    /// Check stderr for SAS/pairing info.
    pub fn sas_code(&self) -> Option<String> {
        self.stderr_lines
            .lock()
            .ok()
            .and_then(|buf| {
                buf.iter().find_map(|l| {
                    if l.contains("[SAS]") {
                        // Extract SAS from log line like "[SAS] AB12CD"
                        l.split("[SAS]").nth(1).map(|s| s.trim().to_string())
                    } else {
                        None
                    }
                })
            })
    }

    /// Check stderr for data channel open (transfer-ready).
    pub fn is_transfer_ready(&self) -> bool {
        self.stderr_lines
            .lock()
            .map(|buf| buf.iter().any(|l| l.contains("[DC_OPEN]")))
            .unwrap_or(false)
    }

    /// Check stderr for errors.
    pub fn last_error(&self) -> Option<String> {
        self.stderr_lines
            .lock()
            .ok()
            .and_then(|buf| {
                buf.iter().rev().find_map(|l| {
                    if l.contains("FATAL") || l.contains("ERROR") || l.contains("error") {
                        Some(l.clone())
                    } else {
                        None
                    }
                })
            })
    }

    /// Kill the daemon process.
    pub fn kill(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for DaemonProcess {
    fn drop(&mut self) {
        self.kill();
    }
}

/// Generate a short random ID for room/session.
pub fn generate_room_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("r{:x}", ts % 0xFFFFFF)
}

pub fn generate_session_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("s{:x}", ts % 0xFFFFFFFF)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn room_session_ids_not_empty() {
        let room = generate_room_id();
        let session = generate_session_id();
        assert!(!room.is_empty());
        assert!(!session.is_empty());
        assert!(room.starts_with('r'));
        assert!(session.starts_with('s'));
    }

    #[test]
    fn find_daemon_binary_does_not_panic() {
        // May succeed or fail depending on build state — must not panic
        let _ = find_daemon_binary();
    }

    #[test]
    fn probe_rendezvous_does_not_hang() {
        // Quick probe — should return within 2s even if server is down
        let start = std::time::Instant::now();
        let _ = probe_rendezvous("127.0.0.1:39999"); // unlikely port
        assert!(start.elapsed().as_secs() < 5);
    }
}
