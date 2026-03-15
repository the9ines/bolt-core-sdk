use eframe::egui;

// AC-EN-08/EN3: Theme tokens aligned with web app (localbolt-v3/localbolt/localbolt-app).
// PM-EN-02: Minimal parity first — match current web app color language.
// Source: tailwind.config.ts + index.css across all three web consumers.

// -- Background / Surface tokens --
// Web: --background = hsl(0 0% 7%) = #121212
pub const WINDOW_BG: egui::Color32 = egui::Color32::from_rgb(0x12, 0x12, 0x12);
// Web: --card / dark-lighter = #1A1A1A
pub const PANEL_BG: egui::Color32 = egui::Color32::from_rgb(0x1A, 0x1A, 0x1A);
// Web: --secondary / dark-accent = #262626
pub const SURFACE_SECONDARY: egui::Color32 = egui::Color32::from_rgb(0x26, 0x26, 0x26);

// -- Primary / Accent tokens --
// Web: --primary / neon = #A4E200
pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(0xA4, 0xE2, 0x00);
// Hover variant (10% lighter)
pub const ACCENT_HOVER: egui::Color32 = egui::Color32::from_rgb(0xB8, 0xF0, 0x33);
// Web: --primary-foreground = #000000
pub const ACCENT_FG: egui::Color32 = egui::Color32::from_rgb(0x00, 0x00, 0x00);

// -- Text tokens --
// Web: --foreground = hsl(0 0% 98%) = #FAFAFA
pub const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(0xFA, 0xFA, 0xFA);
// Web: --muted-foreground = hsl(0 0% 64%) = #A3A3A3
pub const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(0xA3, 0xA3, 0xA3);
// Muted / disabled
pub const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(0x73, 0x73, 0x73);

// -- Semantic tokens --
pub const SUCCESS: egui::Color32 = egui::Color32::from_rgb(0x22, 0xC5, 0x5E);
// Web: --destructive = hsl(0 84% 60%) = #EF4444
pub const ERROR: egui::Color32 = egui::Color32::from_rgb(0xEF, 0x44, 0x44);
pub const WARNING: egui::Color32 = egui::Color32::from_rgb(0xEA, 0xB3, 0x08);

// -- Border / Separator --
// Web: --border = hsl(0 0% 15%) = #262626
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
#[allow(dead_code)] // Used when monospace sizing differs from body
pub const FONT_SIZE_MONO: f32 = 18.0;
pub const FONT_SIZE_SMALL: f32 = 12.0;

// -- Corner radius --
// Web: --radius = 0.75rem = 12px
pub const ROUNDING: u8 = 12;

pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    visuals.window_fill = WINDOW_BG;
    visuals.extreme_bg_color = WINDOW_BG;

    visuals.widgets.noninteractive.bg_fill = PANEL_BG;
    visuals.widgets.noninteractive.fg_stroke =
        egui::Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.inactive.bg_fill = SURFACE_SECONDARY;
    visuals.widgets.inactive.fg_stroke =
        egui::Stroke::new(1.0, TEXT_SECONDARY);
    visuals.widgets.hovered.bg_fill = ACCENT_HOVER;
    visuals.widgets.hovered.fg_stroke =
        egui::Stroke::new(1.0, ACCENT_FG);
    visuals.widgets.active.bg_fill = ACCENT;
    visuals.widgets.active.fg_stroke =
        egui::Stroke::new(1.0, ACCENT_FG);

    visuals.window_corner_radius = egui::CornerRadius::same(ROUNDING);
    visuals.selection.bg_fill = ACCENT.gamma_multiply(0.3);
    visuals.selection.stroke = egui::Stroke::new(1.0, ACCENT);

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
    fn web_parity_accent_is_neon() {
        // Web app primary/accent = #A4E200 (neon green)
        assert_eq!(ACCENT, egui::Color32::from_rgb(0xA4, 0xE2, 0x00));
    }

    #[test]
    fn web_parity_background() {
        // Web app --background = #121212
        assert_eq!(WINDOW_BG, egui::Color32::from_rgb(0x12, 0x12, 0x12));
    }

    #[test]
    fn web_parity_foreground() {
        // Web app --foreground = #FAFAFA
        assert_eq!(TEXT_PRIMARY, egui::Color32::from_rgb(0xFA, 0xFA, 0xFA));
    }

    #[test]
    fn web_parity_radius() {
        // Web app --radius = 0.75rem = 12px
        assert_eq!(ROUNDING, 12);
    }

    #[test]
    fn theme_constants_ordered() {
        assert!(SPACING_SM < SPACING_MD);
        assert!(SPACING_MD < SPACING_LG);
        assert!(SPACING_LG < SPACING_XL);
        assert!(FONT_SIZE_SMALL < FONT_SIZE_BODY);
        assert!(FONT_SIZE_BODY < FONT_SIZE_HEADING);
        assert!(FONT_SIZE_HEADING < FONT_SIZE_TITLE);
    }
}
