#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod file_entry;
mod fonts;
mod theme;

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    // 在debug模式下启用日志
    #[cfg(debug_assertions)]
    {
        // 简单的日志，只在debug模式下使用
        println!("启动文件管理器...");
    }

    // 检查配置目录是否可访问
    if let Err(_e) = check_config_access() {
        #[cfg(debug_assertions)]
        eprintln!("配置访问警告: {}", _e);
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([700.0, 500.0])
            .with_min_inner_size([600.0, 400.0])
            .with_resizable(true)
            .with_title_shown(true)
            .with_decorations(true)
            .with_transparent(false),
        centered: true,
        follow_system_theme: true,
        run_and_return: false,
        
        // 性能优化设置
        multisampling: 0,  // 关闭多重采样以提升性能
        depth_buffer: 0,   // 不需要深度缓冲
        stencil_buffer: 0, // 不需要模板缓冲
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        renderer: eframe::Renderer::Glow,
        
        // 持久化设置
        persist_window: true,
        
        ..Default::default()
    };

    #[cfg(debug_assertions)]
    println!("正在初始化应用程序...");

    eframe::run_native(
        "文件快速访问器 v0.2.2",
        options,
        Box::new(|cc| {
            // 设置渲染选项
            setup_rendering(&cc.egui_ctx);
            
            // 创建应用实例
            let app = app::FileManagerApp::new();
            
            #[cfg(debug_assertions)]
            println!("应用程序初始化完成");
            
            Box::new(app)
        }),
    )
}

/// 设置渲染相关选项
fn setup_rendering(ctx: &egui::Context) {
    // 设置像素比例
    ctx.set_pixels_per_point(1.0);
    
    // 优化文本渲染以支持中文显示
    ctx.tessellation_options_mut(|tess_options| {
        tess_options.round_text_to_pixels = true;
        tess_options.feathering_size_in_pixels = 1.0;
        // 提高曲线分辨率以获得更好的渲染质量
        tess_options.bezier_tolerance = 0.02;
        tess_options.epsilon = 0.0001;
    });
    
    // 应用现代化主题
    theme::ModernTheme::apply_theme(ctx, theme::ThemeMode::System);
}

/// 检查配置文件访问权限
fn check_config_access() -> Result<(), String> {
    let config_manager = config::ConfigManager::new();
    
    // 尝试创建一个临时配置以测试写入权限
    let test_config = config::AppConfig::default();
    config_manager.save_config(&test_config)
        .map_err(|e| format!("无法访问配置目录: {}", e))?;
    
    // 尝试读取配置
    config_manager.load_config()
        .map_err(|e| format!("无法读取配置文件: {}", e))?;
    
    Ok(())
}