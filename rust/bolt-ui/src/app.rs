use eframe::egui;

use crate::screens::{self, Screen};
use crate::theme;

pub struct BoltApp {
    current_screen: Screen,
    peer_code_input: String,
}

impl BoltApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply_theme(&cc.egui_ctx);
        Self {
            current_screen: Screen::Connect,
            peer_code_input: String::new(),
        }
    }
}

impl eframe::App for BoltApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("LocalBolt")
                        .size(theme::FONT_SIZE_HEADING)
                        .color(theme::ACCENT),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
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
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_screen {
                Screen::Connect => {
                    screens::connect::show(ui, &mut self.peer_code_input);
                }
                Screen::Transfer => {
                    screens::transfer::show(ui);
                }
                Screen::Verify => {
                    screens::verify::show(ui);
                }
            }
        });
    }
}
