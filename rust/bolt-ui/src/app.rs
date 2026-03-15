use std::time::Instant;

use eframe::egui;

use crate::screens::{self, Screen};
use crate::state::{ConnectionState, TransferState, VerifyState};
use crate::theme;

pub struct BoltApp {
    current_screen: Screen,
    local_peer_code: String,
    peer_code_input: String,
    connection: ConnectionState,
    transfer: TransferState,
    verify: VerifyState,
}

impl BoltApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply_theme(&cc.egui_ctx);

        let local_peer_code = bolt_core::peer_code::generate_secure_peer_code();

        Self {
            current_screen: Screen::Connect,
            local_peer_code,
            peer_code_input: String::new(),
            connection: ConnectionState::Disconnected,
            transfer: TransferState::Idle,
            verify: VerifyState::NotStarted,
        }
    }

    /// Attempt connection. Validates peer code, then transitions to Connecting
    /// with a timestamp for timeout detection.
    fn attempt_connect(&mut self) {
        if self.peer_code_input.len() != bolt_core::constants::PEER_CODE_LENGTH {
            self.connection = ConnectionState::Error(format!(
                "Peer code must be {} characters",
                bolt_core::constants::PEER_CODE_LENGTH
            ));
            return;
        }

        if !bolt_core::peer_code::is_valid_peer_code(&self.peer_code_input) {
            self.connection =
                ConnectionState::Error("Invalid peer code format".to_string());
            return;
        }

        // Transition to Connecting with timestamp for timeout detection.
        // NOTE: No daemon IPC exists yet. This will time out after CONNECT_TIMEOUT
        // because no daemon is listening. This is the honest, deterministic path
        // instead of hanging forever.
        self.connection = ConnectionState::Connecting {
            started_at: Instant::now(),
        };
    }

    /// Cancel an in-progress connection attempt.
    fn cancel_connect(&mut self) {
        if self.connection.can_cancel() {
            self.connection = ConnectionState::Disconnected;
        }
    }

    /// Check for connect timeout and transition state if expired.
    fn check_connect_timeout(&mut self) {
        if self.connection.is_timed_out() {
            self.connection = ConnectionState::TimedOut;
        }
    }
}

impl eframe::App for BoltApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check connect timeout every frame
        self.check_connect_timeout();

        // Request repaint while connecting (to detect timeout)
        if matches!(self.connection, ConnectionState::Connecting { .. }) {
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
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            let nav = [
                                (Screen::Connect, "Connect"),
                                (Screen::Transfer, "Transfer"),
                                (Screen::Verify, "Verify"),
                            ];
                            for (screen, label) in nav.iter().rev() {
                                let is_active = self.current_screen == *screen;
                                let text = if is_active {
                                    egui::RichText::new(*label).color(theme::ACCENT)
                                } else {
                                    egui::RichText::new(*label).color(theme::TEXT_SECONDARY)
                                };
                                if ui.selectable_label(is_active, text).clicked() {
                                    self.current_screen = *screen;
                                }
                            }
                        },
                    );
                });
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(theme::WINDOW_BG)
                    .inner_margin(theme::SPACING_LG),
            )
            .show(ctx, |ui| match self.current_screen {
                Screen::Connect => {
                    let mut connect_requested = false;
                    let mut cancel_requested = false;
                    screens::connect::show(
                        ui,
                        &self.local_peer_code,
                        &mut self.peer_code_input,
                        &self.connection,
                        &mut connect_requested,
                        &mut cancel_requested,
                    );
                    if connect_requested {
                        self.attempt_connect();
                    }
                    if cancel_requested {
                        self.cancel_connect();
                    }
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
                            self.connection = ConnectionState::Disconnected;
                        }
                        screens::verify::VerifyAction::None => {}
                    }
                }
            });
    }
}
