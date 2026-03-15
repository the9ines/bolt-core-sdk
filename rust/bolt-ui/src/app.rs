use eframe::egui;

use crate::screens::{self, Screen};
use crate::state::{ConnectionState, TransferState, VerifyState};
use crate::theme;

pub struct BoltApp {
    current_screen: Screen,
    // Runtime state — replaces EN2 placeholders
    local_peer_code: String,
    peer_code_input: String,
    connection: ConnectionState,
    transfer: TransferState,
    verify: VerifyState,
}

impl BoltApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply_theme(&cc.egui_ctx);

        // AC-EN-10: Real peer code from bolt_core, not placeholder
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

    /// Simulate connection attempt (EN3 state wiring).
    /// Real daemon IPC is EN3+ scope — this drives the state model.
    fn attempt_connect(&mut self) {
        if self.peer_code_input.len() == bolt_core::constants::PEER_CODE_LENGTH {
            let valid = bolt_core::peer_code::is_valid_peer_code(&self.peer_code_input);
            if valid {
                self.connection = ConnectionState::Connecting;
            } else {
                self.connection =
                    ConnectionState::Error("Invalid peer code format".to_string());
            }
        } else {
            self.connection = ConnectionState::Error(format!(
                "Peer code must be {} characters",
                bolt_core::constants::PEER_CODE_LENGTH
            ));
        }
    }
}

impl eframe::App for BoltApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("header")
            .frame(
                egui::Frame::NONE
                    .fill(theme::PANEL_BG)
                    .inner_margin(egui::Margin::symmetric(theme::SPACING_LG as i8, theme::SPACING_MD as i8)),
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
                    screens::connect::show(
                        ui,
                        &self.local_peer_code,
                        &mut self.peer_code_input,
                        &self.connection,
                        &mut connect_requested,
                    );
                    if connect_requested {
                        self.attempt_connect();
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
