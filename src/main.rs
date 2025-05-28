mod app;
mod config;
mod file_entry;

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "文件快速访问器",
        options,
        Box::new(|_cc| Box::new(app::FileManagerApp::new())),
    )
}
