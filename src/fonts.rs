// 字体管理模块
use eframe::egui;

pub fn setup_chinese_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 尝试加载系统中文字体
    let mut font_loaded = false;

    #[cfg(target_os = "windows")]
    {
        let font_paths = [
            "C:/Windows/Fonts/msyh.ttc",   // 微软雅黑
            "C:/Windows/Fonts/simhei.ttf", // 黑体
            "C:/Windows/Fonts/simsun.ttc", // 宋体
        ];

        for font_path in &font_paths {
            if let Ok(font_data) = std::fs::read(font_path) {
                fonts
                    .font_data
                    .insert("chinese".to_owned(), egui::FontData::from_owned(font_data));
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "chinese".to_owned());
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .insert(0, "chinese".to_owned());
                font_loaded = true;
                break;
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let font_paths = [
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/System/Library/Fonts/STHeiti Medium.ttc",
        ];

        for font_path in &font_paths {
            if let Ok(font_data) = std::fs::read(font_path) {
                fonts
                    .font_data
                    .insert("chinese".to_owned(), egui::FontData::from_owned(font_data));
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "chinese".to_owned());
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .insert(0, "chinese".to_owned());
                font_loaded = true;
                break;
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let font_paths = [
            "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        ];

        for font_path in &font_paths {
            if let Ok(font_data) = std::fs::read(font_path) {
                fonts
                    .font_data
                    .insert("chinese".to_owned(), egui::FontData::from_owned(font_data));
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "chinese".to_owned());
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .insert(0, "chinese".to_owned());
                font_loaded = true;
                break;
            }
        }
    }

    if !font_loaded {
        // 如果没有找到系统字体，添加基本的 Unicode 支持
        // egui 的默认字体已经支持一些中文字符
        println!("警告: 未找到系统中文字体，将使用默认字体（可能显示不完整）");

        // 确保字体定义正确设置
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default();
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default();
    }

    ctx.set_fonts(fonts);
}
