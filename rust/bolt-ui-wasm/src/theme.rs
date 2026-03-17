// EW2 REUSE: Copied verbatim from bolt-ui/src/theme.rs.
// This is the direct-reuse test — zero modifications needed for WASM.

use eframe::egui;

// -- Background / Surface tokens --
pub const WINDOW_BG: egui::Color32 = egui::Color32::from_rgb(0x12, 0x12, 0x12);
pub const PANEL_BG: egui::Color32 = egui::Color32::from_rgb(0x1A, 0x1A, 0x1A);
pub const SURFACE_SECONDARY: egui::Color32 = egui::Color32::from_rgb(0x26, 0x26, 0x26);

// -- Primary / Accent tokens --
pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(0xA4, 0xE2, 0x00);
pub const ACCENT_HOVER: egui::Color32 = egui::Color32::from_rgb(0xB8, 0xF0, 0x33);
pub const ACCENT_FG: egui::Color32 = egui::Color32::from_rgb(0x00, 0x00, 0x00);

// -- Text tokens --
pub const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(0xFA, 0xFA, 0xFA);
pub const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(0xA3, 0xA3, 0xA3);
pub const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(0x73, 0x73, 0x73);

// -- Semantic tokens --
pub const SUCCESS: egui::Color32 = egui::Color32::from_rgb(0x22, 0xC5, 0x5E);
pub const ERROR: egui::Color32 = egui::Color32::from_rgb(0xEF, 0x44, 0x44);
#[allow(dead_code)]
pub const WARNING: egui::Color32 = egui::Color32::from_rgb(0xEA, 0xB3, 0x08);

// -- Border / Separator --
pub const BORDER: egui::Color32 = egui::Color32::from_rgb(0x26, 0x26, 0x26);

// -- Spacing --
pub const SPACING_SM: f32 = 4.0;
pub const SPACING_MD: f32 = 8.0;
pub const SPACING_LG: f32 = 16.0;
pub const SPACING_XL: f32 = 24.0;

// -- Typography --
pub const FONT_SIZE_BODY: f32 = 14.0;
pub const FONT_SIZE_HEADING: f32 = 20.0;
pub const FONT_SIZE_TITLE: f32 = 28.0;
pub const FONT_SIZE_SMALL: f32 = 12.0;

// -- Corner radius --
pub const ROUNDING: u8 = 12;

pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    visuals.window_fill = WINDOW_BG;
    visuals.extreme_bg_color = WINDOW_BG;

    visuals.widgets.noninteractive.bg_fill = PANEL_BG;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.inactive.bg_fill = SURFACE_SECONDARY;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT_SECONDARY);
    visuals.widgets.hovered.bg_fill = ACCENT_HOVER;
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, ACCENT_FG);
    visuals.widgets.active.bg_fill = ACCENT;
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, ACCENT_FG);

    visuals.window_corner_radius = egui::CornerRadius::same(ROUNDING);
    visuals.selection.bg_fill = ACCENT.gamma_multiply(0.3);
    visuals.selection.stroke = egui::Stroke::new(1.0, ACCENT);

    let mut style = (*ctx.style()).clone();
    style.visuals = visuals;
    style.spacing.item_spacing = egui::vec2(SPACING_MD, SPACING_MD);
    style.spacing.button_padding = egui::vec2(SPACING_LG, SPACING_MD);

    ctx.set_style(style);
}
