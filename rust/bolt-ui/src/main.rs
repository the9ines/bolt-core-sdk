pub mod app;
pub mod daemon;
pub mod ipc;
mod screens;
pub mod state;
mod theme;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([520.0, 700.0])
            .with_min_inner_size([400.0, 550.0])
            .with_title("LocalBolt"),
        ..Default::default()
    };

    eframe::run_native(
        "LocalBolt",
        options,
        Box::new(|cc| Ok(Box::new(app::BoltApp::new(cc)))),
    )
}
