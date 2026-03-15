use eframe::egui;

// AC-EN-08: Theme baseline constants (minimal parity with current Tauri WebView app).
// PM-EN-02: Minimal parity first — match current look, no custom design language.

pub const WINDOW_BG: egui::Color32 = egui::Color32::from_rgb(17, 17, 17);
pub const PANEL_BG: egui::Color32 = egui::Color32::from_rgb(28, 28, 30);
pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(99, 102, 241);
pub const ACCENT_HOVER: egui::Color32 = egui::Color32::from_rgb(129, 132, 255);
pub const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(245, 245, 245);
pub const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(163, 163, 163);
pub const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(115, 115, 115);
pub const SUCCESS: egui::Color32 = egui::Color32::from_rgb(34, 197, 94);
pub const ERROR: egui::Color32 = egui::Color32::from_rgb(239, 68, 68);
pub const WARNING: egui::Color32 = egui::Color32::from_rgb(234, 179, 8);
pub const BORDER: egui::Color32 = egui::Color32::from_rgb(55, 55, 60);

pub const SPACING_SM: f32 = 4.0;
pub const SPACING_MD: f32 = 8.0;
pub const SPACING_LG: f32 = 16.0;
pub const SPACING_XL: f32 = 24.0;

pub const FONT_SIZE_BODY: f32 = 14.0;
pub const FONT_SIZE_HEADING: f32 = 20.0;
pub const FONT_SIZE_TITLE: f32 = 28.0;
pub const FONT_SIZE_MONO: f32 = 18.0;

pub const ROUNDING: u8 = 8;
pub const ROUNDING_F32: f32 = 8.0;

pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    visuals.window_fill = WINDOW_BG;
    visuals.extreme_bg_color = WINDOW_BG;

    visuals.widgets.noninteractive.bg_fill = PANEL_BG;
    visuals.widgets.noninteractive.fg_stroke =
        egui::Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.inactive.bg_fill = PANEL_BG;
    visuals.widgets.inactive.fg_stroke =
        egui::Stroke::new(1.0, TEXT_SECONDARY);
    visuals.widgets.hovered.bg_fill = ACCENT_HOVER;
    visuals.widgets.active.bg_fill = ACCENT;

    visuals.window_corner_radius = egui::CornerRadius::same(ROUNDING);

    let mut style = (*ctx.style()).clone();
    style.visuals = visuals;
    style.spacing.item_spacing = egui::vec2(SPACING_MD, SPACING_MD);
    style.spacing.button_padding = egui::vec2(SPACING_LG, SPACING_MD);

    ctx.set_style(style);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_constants_valid() {
        assert!(FONT_SIZE_BODY > 0.0);
        assert!(FONT_SIZE_HEADING > FONT_SIZE_BODY);
        assert!(FONT_SIZE_TITLE > FONT_SIZE_HEADING);
        assert!(SPACING_SM < SPACING_MD);
        assert!(SPACING_MD < SPACING_LG);
        assert!(SPACING_LG < SPACING_XL);
        assert!(ROUNDING > 0);
    }

    #[test]
    fn colors_are_dark_theme() {
        assert!(WINDOW_BG.r() < 30);
        assert!(TEXT_PRIMARY.r() > 200);
    }
}
