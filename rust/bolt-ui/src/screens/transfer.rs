use eframe::egui;

use crate::theme;

/// AC-EN-07: Transfer screen skeleton.
/// Shows file transfer progress, file list, and send/receive controls.
/// No actual transfer logic — skeleton only (EN2 scope).
pub fn show(ui: &mut egui::Ui) {
    ui.add_space(theme::SPACING_XL);

    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new("File Transfer")
                .size(theme::FONT_SIZE_HEADING)
                .color(theme::TEXT_PRIMARY),
        );

        ui.add_space(theme::SPACING_XL);

        // Progress bar skeleton
        ui.label(
            egui::RichText::new("No active transfer")
                .size(theme::FONT_SIZE_BODY)
                .color(theme::TEXT_MUTED),
        );
        ui.add_space(theme::SPACING_SM);

        let progress_bar = egui::ProgressBar::new(0.0)
            .text("0%")
            .desired_width(300.0);
        ui.add(progress_bar);

        ui.add_space(theme::SPACING_XL);

        // File list skeleton
        egui::Frame::NONE
            .fill(theme::WINDOW_BG)
            .corner_radius(theme::ROUNDING)
            .inner_margin(theme::SPACING_LG)
            .show(ui, |ui| {
                ui.set_min_width(300.0);
                ui.label(
                    egui::RichText::new("Files")
                        .size(theme::FONT_SIZE_BODY)
                        .color(theme::TEXT_SECONDARY),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new("No files queued")
                        .size(theme::FONT_SIZE_BODY)
                        .color(theme::TEXT_MUTED),
                );
            });

        ui.add_space(theme::SPACING_XL);

        // Send button skeleton
        let send_btn = egui::Button::new(
            egui::RichText::new("Send File")
                .size(theme::FONT_SIZE_BODY)
                .color(theme::TEXT_PRIMARY),
        )
        .fill(theme::ACCENT)
        .corner_radius(theme::ROUNDING);

        if ui.add(send_btn).clicked() {
            // EN3 scope: file picker + transfer initiation
        }
    });
}
