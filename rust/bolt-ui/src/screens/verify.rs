use eframe::egui;

use crate::theme;

/// AC-EN-07: Verification screen skeleton.
/// Shows SAS code display with confirm/reject controls.
/// No actual SAS logic — skeleton only (EN2 scope).
pub fn show(ui: &mut egui::Ui) {
    ui.add_space(theme::SPACING_XL);

    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new("Verify Connection")
                .size(theme::FONT_SIZE_HEADING)
                .color(theme::TEXT_PRIMARY),
        );

        ui.add_space(theme::SPACING_XL);

        ui.label(
            egui::RichText::new("Compare this code with your peer:")
                .size(theme::FONT_SIZE_BODY)
                .color(theme::TEXT_SECONDARY),
        );

        ui.add_space(theme::SPACING_LG);

        // SAS code display skeleton
        egui::Frame::NONE
            .fill(theme::WINDOW_BG)
            .corner_radius(theme::ROUNDING)
            .inner_margin(theme::SPACING_XL)
            .stroke(egui::Stroke::new(1.0, theme::BORDER))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("A3 F7 2B")
                        .size(theme::FONT_SIZE_TITLE)
                        .color(theme::ACCENT)
                        .monospace(),
                );
            });

        ui.add_space(theme::SPACING_XL);

        ui.horizontal(|ui| {
            let confirm_btn = egui::Button::new(
                egui::RichText::new("Confirm")
                    .size(theme::FONT_SIZE_BODY)
                    .color(theme::TEXT_PRIMARY),
            )
            .fill(theme::SUCCESS)
            .corner_radius(theme::ROUNDING);

            let reject_btn = egui::Button::new(
                egui::RichText::new("Reject")
                    .size(theme::FONT_SIZE_BODY)
                    .color(theme::TEXT_PRIMARY),
            )
            .fill(theme::ERROR)
            .corner_radius(theme::ROUNDING);

            if ui.add(confirm_btn).clicked() {
                // EN3 scope: SAS confirmation logic
            }

            if ui.add(reject_btn).clicked() {
                // EN3 scope: SAS rejection + disconnect
            }
        });

        ui.add_space(theme::SPACING_XL);

        ui.label(
            egui::RichText::new("Status: Awaiting verification")
                .size(theme::FONT_SIZE_BODY)
                .color(theme::TEXT_MUTED),
        );
    });
}
