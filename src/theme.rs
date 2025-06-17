use eframe::egui;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

impl Default for ThemeMode {
    fn default() -> Self {
        ThemeMode::System
    }
}

pub struct ModernTheme;

impl ModernTheme {
    pub fn apply_theme(ctx: &egui::Context, theme_mode: ThemeMode) {
        let mut visuals = match theme_mode {
            ThemeMode::Light => Self::zed_light_theme(),
            ThemeMode::Dark => Self::zed_dark_theme(),
            ThemeMode::System => {
                if ctx.style().visuals.dark_mode {
                    Self::zed_dark_theme()
                } else {
                    Self::zed_light_theme()
                }
            }
        };

        // 设置圆角
        visuals.widgets.noninteractive.rounding = egui::Rounding::same(4.0);
        visuals.widgets.inactive.rounding = egui::Rounding::same(4.0);
        visuals.widgets.hovered.rounding = egui::Rounding::same(4.0);
        visuals.widgets.active.rounding = egui::Rounding::same(4.0);
        visuals.widgets.open.rounding = egui::Rounding::same(4.0);

        ctx.set_visuals(visuals);
    }

    fn zed_light_theme() -> egui::Visuals {
        let mut visuals = egui::Visuals::light();

        // Zed风格的浅色灰调
        visuals.panel_fill = egui::Color32::from_rgb(248, 248, 248);
        visuals.window_fill = egui::Color32::from_rgb(253, 253, 253);
        visuals.extreme_bg_color = egui::Color32::from_rgb(242, 242, 242);

        // 按钮和交互元素的灰调
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(245, 245, 245);
        visuals.widgets.noninteractive.bg_stroke =
            egui::Stroke::new(1.0, egui::Color32::from_rgb(225, 225, 225));

        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(240, 240, 240);
        visuals.widgets.inactive.bg_stroke =
            egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 220, 220));

        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(235, 235, 235);
        visuals.widgets.hovered.bg_stroke =
            egui::Stroke::new(1.0, egui::Color32::from_rgb(210, 210, 210));

        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(230, 230, 230);
        visuals.widgets.active.bg_stroke =
            egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 200, 200));

        // 选择和高亮使用灰蓝色
        visuals.selection.bg_fill = egui::Color32::from_rgb(100, 120, 140).gamma_multiply(0.3);
        visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 120, 140));

        // 超链接使用深灰色
        visuals.hyperlink_color = egui::Color32::from_rgb(80, 80, 80);

        visuals
    }

    fn zed_dark_theme() -> egui::Visuals {
        let mut visuals = egui::Visuals::dark();

        // Zed风格的深色灰调
        visuals.panel_fill = egui::Color32::from_rgb(24, 24, 24);
        visuals.window_fill = egui::Color32::from_rgb(28, 28, 28);
        visuals.extreme_bg_color = egui::Color32::from_rgb(18, 18, 18);

        // 按钮和交互元素的深灰调
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(32, 32, 32);
        visuals.widgets.noninteractive.bg_stroke =
            egui::Stroke::new(1.0, egui::Color32::from_rgb(45, 45, 45));

        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(36, 36, 36);
        visuals.widgets.inactive.bg_stroke =
            egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 50, 50));

        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(42, 42, 42);
        visuals.widgets.hovered.bg_stroke =
            egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60));

        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(48, 48, 48);
        visuals.widgets.active.bg_stroke =
            egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 70, 70));

        // 选择和高亮使用浅灰色
        visuals.selection.bg_fill = egui::Color32::from_rgb(120, 120, 120).gamma_multiply(0.4);
        visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(160, 160, 160));

        // 超链接使用浅灰色
        visuals.hyperlink_color = egui::Color32::from_rgb(180, 180, 180);

        visuals
    }
}
