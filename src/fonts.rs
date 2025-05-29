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
                fonts.font_data.insert("chinese".to_owned(), egui::FontData::from_owned(font_data));
                fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "chinese".to_owned());
                fonts.families.entry(egui::FontFamily::Monospace).or_default().insert(0, "chinese".to_owned());
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
                fonts.font_data.insert("chinese".to_owned(), egui::FontData::from_owned(font_data));
                fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "chinese".to_owned());
                fonts.families.entry(egui::FontFamily::Monospace).or_default().insert(0, "chinese".to_owned());
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
                fonts.font_data.insert("chinese".to_owned(), egui::FontData::from_owned(font_data));
                fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "chinese".to_owned());
                fonts.families.entry(egui::FontFamily::Monospace).or_default().insert(0, "chinese".to_owned());
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
        fonts.families.entry(egui::FontFamily::Proportional).or_default();
        fonts.families.entry(egui::FontFamily::Monospace).or_default();
    }

    ctx.set_fonts(fonts);
}

// 检查中文字体是否可用
pub fn test_chinese_display() -> bool {
    // 简单测试，在实际应用中可以通过渲染测试字符来检查
    #[cfg(target_os = "windows")]
    {
        std::path::Path::new("C:/Windows/Fonts/msyh.ttc").exists() ||
        std::path::Path::new("C:/Windows/Fonts/simhei.ttf").exists() ||
        std::path::Path::new("C:/Windows/Fonts/simsun.ttc").exists()
    }
    
    #[cfg(target_os = "macos")]
    {
        std::path::Path::new("/System/Library/Fonts/PingFang.ttc").exists() ||
        std::path::Path::new("/System/Library/Fonts/Hiragino Sans GB.ttc").exists() ||
        std::path::Path::new("/System/Library/Fonts/STHeiti Medium.ttc").exists()
    }
    
    #[cfg(target_os = "linux")]
    {
        std::path::Path::new("/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf").exists() ||
        std::path::Path::new("/usr/share/fonts/truetype/wqy/wqy-microhei.ttc").exists() ||
        std::path::Path::new("/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc").exists() ||
        std::path::Path::new("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc").exists()
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        false
    }
}