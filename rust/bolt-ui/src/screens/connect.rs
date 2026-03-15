use eframe::egui;

use crate::state::ConnectionState;
use crate::theme;

/// AC-EN-10: Connection screen — real peer code, dynamic status.
/// Peer code from bolt_core::peer_code (runtime-generated, not placeholder).
pub fn show(
    ui: &mut egui::Ui,
    local_peer_code: &str,
    peer_code_input: &mut String,
    connection: &ConnectionState,
    connect_requested: &mut bool,
) {
    ui.add_space(theme::SPACING_XL);

    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new("Your Peer Code")
                .size(theme::FONT_SIZE_SMALL)
                .color(theme::TEXT_SECONDARY),
        );
        ui.add_space(theme::SPACING_SM);

        // Real peer code from bolt_core — not hardcoded
        ui.label(
            egui::RichText::new(local_peer_code)
                .size(theme::FONT_SIZE_TITLE)
                .color(theme::ACCENT)
                .monospace(),
        );

        ui.add_space(theme::SPACING_SM);
        ui.label(
            egui::RichText::new("Share this code with your peer")
                .size(theme::FONT_SIZE_SMALL)
                .color(theme::TEXT_MUTED),
        );

        ui.add_space(theme::SPACING_XL);
        ui.separator();
        ui.add_space(theme::SPACING_XL);

        ui.label(
            egui::RichText::new("Enter Remote Peer Code")
                .size(theme::FONT_SIZE_BODY)
                .color(theme::TEXT_SECONDARY),
        );
        ui.add_space(theme::SPACING_SM);

        let input = egui::TextEdit::singleline(peer_code_input)
            .hint_text("Enter 6-character code")
            .font(egui::TextStyle::Monospace)
            .desired_width(200.0)
            .char_limit(bolt_core::constants::PEER_CODE_LENGTH);
        ui.add(input);

        ui.add_space(theme::SPACING_LG);

        let is_connecting = matches!(connection, ConnectionState::Connecting);
        let btn_label = if is_connecting { "Connecting\u{2026}" } else { "Connect" };

        let connect_btn = egui::Button::new(
            egui::RichText::new(btn_label)
                .size(theme::FONT_SIZE_BODY)
                .color(theme::ACCENT_FG),
        )
        .fill(theme::ACCENT)
        .corner_radius(theme::ROUNDING);

        let btn_enabled = !peer_code_input.is_empty() && !is_connecting;
        if ui.add_enabled(btn_enabled, connect_btn).clicked() {
            *connect_requested = true;
        }

        ui.add_space(theme::SPACING_XL);

        // Dynamic connection status — no placeholder text
        let status_color = match connection {
            ConnectionState::Disconnected => theme::TEXT_MUTED,
            ConnectionState::Connecting => theme::WARNING,
            ConnectionState::Connected { .. } => theme::SUCCESS,
            ConnectionState::Error(_) => theme::ERROR,
        };

        let status_string = match connection {
            ConnectionState::Connected { remote_peer_code } => {
                format!("Connected to {}", remote_peer_code)
            }
            ConnectionState::Error(msg) => format!("Error: {}", msg),
            other => other.status_text().to_string(),
        };

        ui.label(
            egui::RichText::new(&status_string)
                .size(theme::FONT_SIZE_BODY)
                .color(status_color),
        );
    });
}
