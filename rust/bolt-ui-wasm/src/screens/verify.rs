// EW2 REUSE: Copied from bolt-ui/src/screens/verify.rs.
// Change: crate path only. Zero logic changes.

use eframe::egui;

use crate::state::VerifyState;
use crate::theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyAction {
    None,
    Confirm,
    Reject,
}

pub fn show(ui: &mut egui::Ui, verify: &VerifyState, action: &mut VerifyAction) {
    ui.add_space(theme::SPACING_XL);

    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new("Verify Connection")
                .size(theme::FONT_SIZE_HEADING)
                .color(theme::TEXT_PRIMARY),
        );

        ui.add_space(theme::SPACING_XL);

        let status_color = match verify {
            VerifyState::NotStarted => theme::TEXT_MUTED,
            VerifyState::Pending { .. } => theme::ACCENT,
            VerifyState::Confirmed => theme::SUCCESS,
            VerifyState::Rejected => theme::ERROR,
        };

        ui.label(
            egui::RichText::new(verify.status_text())
                .size(theme::FONT_SIZE_BODY)
                .color(status_color),
        );

        ui.add_space(theme::SPACING_LG);

        match verify {
            VerifyState::Pending { sas_code } => {
                let formatted = sas_code
                    .chars()
                    .collect::<Vec<_>>()
                    .chunks(2)
                    .map(|c| c.iter().collect::<String>())
                    .collect::<Vec<_>>()
                    .join(" ");

                egui::Frame::NONE
                    .fill(theme::PANEL_BG)
                    .corner_radius(theme::ROUNDING)
                    .inner_margin(theme::SPACING_XL)
                    .stroke(egui::Stroke::new(2.0, theme::ACCENT))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(&formatted)
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
                        *action = VerifyAction::Confirm;
                    }
                    if ui.add(reject_btn).clicked() {
                        *action = VerifyAction::Reject;
                    }
                });
            }
            VerifyState::NotStarted => {
                egui::Frame::NONE
                    .fill(theme::PANEL_BG)
                    .corner_radius(theme::ROUNDING)
                    .inner_margin(theme::SPACING_XL)
                    .stroke(egui::Stroke::new(1.0, theme::BORDER))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("- - - - - -")
                                .size(theme::FONT_SIZE_TITLE)
                                .color(theme::TEXT_MUTED)
                                .monospace(),
                        );
                    });
            }
            VerifyState::Confirmed => {
                egui::Frame::NONE
                    .fill(theme::PANEL_BG)
                    .corner_radius(theme::ROUNDING)
                    .inner_margin(theme::SPACING_XL)
                    .stroke(egui::Stroke::new(2.0, theme::SUCCESS))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("\u{2713} Verified")
                                .size(theme::FONT_SIZE_TITLE)
                                .color(theme::SUCCESS),
                        );
                    });
            }
            VerifyState::Rejected => {
                egui::Frame::NONE
                    .fill(theme::PANEL_BG)
                    .corner_radius(theme::ROUNDING)
                    .inner_margin(theme::SPACING_XL)
                    .stroke(egui::Stroke::new(2.0, theme::ERROR))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("\u{2717} Rejected")
                                .size(theme::FONT_SIZE_TITLE)
                                .color(theme::ERROR),
                        );
                    });
            }
        }
    });
}
