#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod file_entry;

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([800.0, 500.0])
            .with_resizable(true)
            .with_maximize_button(true)
            .with_minimize_button(true)
            .with_close_button(true)
            .with_title_shown(true)
            .with_decorations(true),
        centered: true,
        follow_system_theme: false,
        default_theme: eframe::Theme::Light,
        run_and_return: false,
        multisampling: 4,
        depth_buffer: 0,
        stencil_buffer: 0,
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        "文件快速访问器",
        options,
        Box::new(|cc| {
            // 设置更好的渲染选项
            cc.egui_ctx.set_pixels_per_point(1.0);
            
            // 启用更好的文本渲染
            cc.egui_ctx.tessellation_options_mut(|tess_options| {
                tess_options.round_text_to_pixels = true;
                tess_options.feathering_size_in_pixels = 1.0;
            });
            
            Box::new(app::FileManagerApp::new())
        }),
    )
}
