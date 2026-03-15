use std::time::Instant;

use eframe::egui;

use crate::daemon::{self, DaemonProcess};
use crate::ipc::IpcClient;
use crate::screens::{self, Screen};
use crate::state::*;
use crate::theme;

pub struct BoltApp {
    pub current_screen: Screen,
    pub mode: ConnectMode,
    pub host_info: Option<HostInfo>,
    pub join_room: String,
    pub join_session: String,
    pub join_peer_code: String,
    pub local_peer_code: String,
    pub connection: ConnectionState,
    pub transfer: TransferState,
    pub verify: VerifyState,
    pub daemon_proc: Option<DaemonProcess>,
    pub ipc_client: Option<IpcClient>,
    pub prereq_error: Option<String>,
    daemon_bin: Option<std::path::PathBuf>,
    data_dir: String,
    socket_path: String,
}

impl BoltApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply_theme(&cc.egui_ctx);

        let local_peer_code = bolt_core::peer_code::generate_secure_peer_code();

        // Unique data dir to avoid identity collision in multi-instance
        let data_dir = format!("/tmp/bolt-ui-{}", std::process::id());
        let socket_path = format!("/tmp/bolt-ui-{}.sock", std::process::id());

        // Pre-check daemon binary
        let (daemon_bin, prereq_error) = match daemon::find_daemon_binary() {
            Ok(path) => (Some(path), None),
            Err(e) => (None, Some(e)),
        };

        Self {
            current_screen: Screen::Connect,
            mode: ConnectMode::Host,
            host_info: None,
            join_room: String::new(),
            join_session: String::new(),
            join_peer_code: String::new(),
            local_peer_code,
            connection: ConnectionState::Idle,
            transfer: TransferState::Idle,
            verify: VerifyState::NotStarted,
            daemon_proc: None,
            ipc_client: None,
            prereq_error,
            daemon_bin,
            data_dir,
            socket_path,
        }
    }

    pub fn start_host(&mut self) {
        let _daemon_bin = match &self.daemon_bin {
            Some(b) => b.clone(),
            None => {
                self.connection = ConnectionState::Error(
                    "Daemon binary not found".into(),
                );
                return;
            }
        };

        if !daemon::probe_rendezvous("127.0.0.1:3001") {
            self.connection = ConnectionState::Error(
                "Rendezvous server not reachable at ws://127.0.0.1:3001".into(),
            );
            return;
        }

        let room = daemon::generate_room_id();
        let session = daemon::generate_session_id();
        let peer_code = self.local_peer_code.clone();

        // Create data dir
        let _ = std::fs::create_dir_all(&self.data_dir);

        // Host doesn't know join peer code yet — use wildcard placeholder
        // The daemon --expect-peer will be the joiner's code, entered later
        // For now, set a placeholder that will be replaced when join info is entered
        self.host_info = Some(HostInfo {
            peer_code: peer_code.clone(),
            room: room.clone(),
            session: session.clone(),
        });

        // We can't spawn yet — we need to know the join peer's code for --expect-peer
        // Show host info and wait for the join side to connect
        self.connection = ConnectionState::Idle;
    }

    pub fn start_host_with_joiner(&mut self, joiner_code: &str) {
        let daemon_bin = match &self.daemon_bin {
            Some(b) => b.clone(),
            None => {
                self.connection = ConnectionState::Error("Daemon binary not found".into());
                return;
            }
        };

        let info = match &self.host_info {
            Some(i) => i.clone(),
            None => {
                self.connection = ConnectionState::Error("No host info".into());
                return;
            }
        };

        let _ = std::fs::create_dir_all(&self.data_dir);

        match DaemonProcess::spawn_host(
            &daemon_bin,
            &info.peer_code,
            joiner_code,
            &info.room,
            &info.session,
            &self.socket_path,
            &self.data_dir,
        ) {
            Ok(proc) => {
                self.daemon_proc = Some(proc);
                self.connection = ConnectionState::Connecting {
                    started_at: Instant::now(),
                };
            }
            Err(e) => {
                self.connection = ConnectionState::Error(format!("Spawn failed: {e}"));
            }
        }
    }

    pub fn start_join(&mut self) {
        let daemon_bin = match &self.daemon_bin {
            Some(b) => b.clone(),
            None => {
                self.connection = ConnectionState::Error("Daemon binary not found".into());
                return;
            }
        };

        if !daemon::probe_rendezvous("127.0.0.1:3001") {
            self.connection = ConnectionState::Error(
                "Rendezvous server not reachable at ws://127.0.0.1:3001".into(),
            );
            return;
        }

        let _ = std::fs::create_dir_all(&self.data_dir);

        match DaemonProcess::spawn_join(
            &daemon_bin,
            &self.local_peer_code,
            &self.join_peer_code,
            &self.join_room,
            &self.join_session,
            &self.socket_path,
            &self.data_dir,
        ) {
            Ok(proc) => {
                self.daemon_proc = Some(proc);
                self.connection = ConnectionState::Connecting {
                    started_at: Instant::now(),
                };
            }
            Err(e) => {
                self.connection = ConnectionState::Error(format!("Spawn failed: {e}"));
            }
        }
    }

    pub fn cancel_connect(&mut self) {
        if let Some(mut proc) = self.daemon_proc.take() {
            proc.kill();
        }
        self.ipc_client = None;
        self.connection = ConnectionState::Idle;
        self.transfer = TransferState::Idle;
        self.verify = VerifyState::NotStarted;
    }

    pub fn poll_daemon(&mut self) {
        // Check timeout
        if self.connection.is_timed_out() {
            let error_detail = self.daemon_proc
                .as_ref()
                .and_then(|p| p.last_error())
                .unwrap_or_else(|| "Connection timed out".into());
            if let Some(mut proc) = self.daemon_proc.take() {
                proc.kill();
            }
            self.ipc_client = None;
            self.connection = ConnectionState::Error(error_detail);
            return;
        }

        // Poll daemon process
        if let Some(proc) = &mut self.daemon_proc {
            if !proc.is_running() {
                let error = proc.last_error()
                    .unwrap_or_else(|| "Daemon exited unexpectedly".into());
                self.connection = ConnectionState::Error(error);
                self.daemon_proc = None;
                self.ipc_client = None;
                return;
            }

            // Try to establish IPC connection if not yet connected
            if self.ipc_client.is_none()
                && matches!(self.connection, ConnectionState::Connecting { .. })
            {
                // Wait a bit for daemon to start IPC server
                if proc.recent_stderr(20).iter().any(|l| l.contains("[IPC] listening")) {
                    match IpcClient::connect(&self.socket_path) {
                        Ok(client) => {
                            self.ipc_client = Some(client);
                        }
                        Err(_) => {
                            // Will retry on next frame
                        }
                    }
                }
            }
        }

        // Process IPC events (primary data path — not stderr)
        if let Some(client) = &self.ipc_client {
            for event in client.drain_events() {
                match event.msg_type.as_str() {
                    "daemon.status" => {
                        // Handshake complete — daemon is responsive
                    }
                    "session.connected" => {
                        let caps = event.payload.get("negotiated_capabilities")
                            .and_then(|v| v.as_array())
                            .map(|a| a.len())
                            .unwrap_or(0);
                        // Only transition to Connected when capabilities are negotiated (HELLO done)
                        if caps > 0 {
                            self.connection = ConnectionState::Connected;
                            self.transfer = TransferState::Ready;
                        }
                    }
                    "session.sas" => {
                        if let Some(sas) = event.payload.get("sas").and_then(|v| v.as_str()) {
                            self.verify = VerifyState::Pending {
                                sas_code: sas.to_string(),
                            };
                        }
                    }
                    "session.error" => {
                        let reason = event.payload.get("reason")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown error")
                            .to_string();
                        self.connection = ConnectionState::Error(reason);
                    }
                    "session.ended" => {
                        self.connection = ConnectionState::Idle;
                        self.transfer = TransferState::Idle;
                        self.verify = VerifyState::NotStarted;
                    }
                    "transfer.started" => {
                        let file_name = event.payload.get("file_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string();
                        let direction = event.payload.get("direction")
                            .and_then(|v| v.as_str())
                            .unwrap_or("receive");
                        if direction == "send" {
                            self.transfer = TransferState::Sending { file_name, progress: 0.0 };
                        } else {
                            self.transfer = TransferState::Receiving { file_name, progress: 0.0 };
                        }
                    }
                    "transfer.progress" => {
                        let progress = event.payload.get("progress")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0) as f32;
                        match &mut self.transfer {
                            TransferState::Sending { progress: p, .. }
                            | TransferState::Receiving { progress: p, .. } => {
                                *p = progress;
                            }
                            _ => {}
                        }
                    }
                    "transfer.complete" => {
                        let file_name = event.payload.get("file_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string();
                        self.transfer = TransferState::Complete { file_name };
                    }
                    _ => {} // Unknown event — ignore
                }
            }
        }
    }
}

impl eframe::App for BoltApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll daemon every frame while connecting/connected
        if self.daemon_proc.is_some() {
            self.poll_daemon();
            ctx.request_repaint();
        }

        egui::TopBottomPanel::top("header")
            .frame(
                egui::Frame::NONE
                    .fill(theme::PANEL_BG)
                    .inner_margin(egui::Margin::symmetric(
                        theme::SPACING_LG as i8,
                        theme::SPACING_MD as i8,
                    )),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("LocalBolt")
                            .size(theme::FONT_SIZE_HEADING)
                            .color(theme::ACCENT),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        for (screen, label) in [
                            (Screen::Verify, "Verify"),
                            (Screen::Transfer, "Transfer"),
                            (Screen::Connect, "Connect"),
                        ] {
                            let active = self.current_screen == screen;
                            let color = if active { theme::ACCENT } else { theme::TEXT_SECONDARY };
                            if ui.selectable_label(active, egui::RichText::new(label).color(color)).clicked() {
                                self.current_screen = screen;
                            }
                        }
                    });
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(theme::WINDOW_BG).inner_margin(theme::SPACING_LG))
            .show(ctx, |ui| match self.current_screen {
                Screen::Connect => {
                    screens::connect::show(ui, self);
                }
                Screen::Transfer => {
                    screens::transfer::show(ui, &self.transfer);
                }
                Screen::Verify => {
                    let mut action = screens::verify::VerifyAction::None;
                    screens::verify::show(ui, &self.verify, &mut action);
                    match action {
                        screens::verify::VerifyAction::Confirm => {
                            self.verify = VerifyState::Confirmed;
                        }
                        screens::verify::VerifyAction::Reject => {
                            self.verify = VerifyState::Rejected;
                            self.cancel_connect();
                        }
                        screens::verify::VerifyAction::None => {}
                    }
                }
            });
    }
}

impl Drop for BoltApp {
    fn drop(&mut self) {
        // Clean up daemon process and temp files on exit
        if let Some(mut proc) = self.daemon_proc.take() {
            proc.kill();
        }
        let _ = std::fs::remove_file(&self.socket_path);
        let _ = std::fs::remove_dir_all(&self.data_dir);
    }
}
