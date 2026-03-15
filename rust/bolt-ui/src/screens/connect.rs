use eframe::egui;

use crate::theme;

/// AC-EN-07: Connection screen skeleton.
/// Displays local peer code and accepts remote peer code input.
/// No actual connection logic — skeleton only (EN2 scope).
pub fn show(ui: &mut egui::Ui, peer_code_input: &mut String) {
    ui.add_space(theme::SPACING_XL);

    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new("Your Peer Code")
                .size(theme::FONT_SIZE_BODY)
                .color(theme::TEXT_SECONDARY),
        );
        ui.add_space(theme::SPACING_SM);
        ui.label(
            egui::RichText::new("ABC123")
                .size(theme::FONT_SIZE_TITLE)
                .color(theme::TEXT_PRIMARY)
                .monospace(),
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
            .hint_text("e.g. XYZ789")
            .font(egui::TextStyle::Monospace)
            .desired_width(200.0);
        ui.add(input);

        ui.add_space(theme::SPACING_LG);

        let connect_btn = egui::Button::new(
            egui::RichText::new("Connect")
                .size(theme::FONT_SIZE_BODY)
                .color(theme::TEXT_PRIMARY),
        )
        .fill(theme::ACCENT)
        .corner_radius(theme::ROUNDING);

        if ui.add(connect_btn).clicked() {
            // EN3 scope: wire to daemon IPC
        }

        ui.add_space(theme::SPACING_XL);
        ui.label(
            egui::RichText::new("Status: Not connected")
                .size(theme::FONT_SIZE_BODY)
                .color(theme::TEXT_MUTED),
        );
    });
}
