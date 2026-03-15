mod app;
mod screens;
mod state;
mod theme;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([480.0, 640.0])
            .with_min_inner_size([360.0, 480.0])
            .with_title("LocalBolt"),
        ..Default::default()
    };

    eframe::run_native(
        "LocalBolt",
        options,
        Box::new(|cc| Ok(Box::new(app::BoltApp::new(cc)))),
    )
}
