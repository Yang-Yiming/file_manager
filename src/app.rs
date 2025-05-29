use crate::config::{Config, ConfigManager};
use crate::file_entry::FileEntry;
use eframe::egui;
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

pub struct FileManagerApp {
    entries: Vec<FileEntry>,
    search_query: String,
    config_manager: ConfigManager,
    config: Config,
    font_loaded: bool,
    show_settings: bool,
    all_tags: HashSet<String>,
    theme_mode: ThemeMode,
    
    // 性能优化相关
    filtered_indices: Vec<usize>,
    last_search_query: String,
    last_filter_time: Instant,
    
    // 简化的添加功能
    add_path_input: String,
    add_name_input: String,
    show_add_dialog: bool,
}

impl Default for FileManagerApp {
    fn default() -> Self {
        Self::new()
    }
}

impl FileManagerApp {
    pub fn new() -> Self {
        let config_manager = ConfigManager::new();
        let config = config_manager.load_config().unwrap_or_default();
        let entries = config.entries.clone();

        let mut all_tags = HashSet::new();
        for entry in &entries {
            for tag in &entry.tags {
                all_tags.insert(tag.clone());
            }
        }

        let filtered_indices: Vec<usize> = (0..entries.len()).collect();

        Self {
            entries,
            search_query: String::new(),
            config_manager,
            config,
            font_loaded: false,
            show_settings: false,
            all_tags,
            theme_mode: ThemeMode::System,
            filtered_indices,
            last_search_query: String::new(),
            last_filter_time: Instant::now(),
            add_path_input: String::new(),
            add_name_input: String::new(),
            show_add_dialog: false,
        }
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        match self.theme_mode {
            ThemeMode::Light => ctx.set_visuals(egui::Visuals::light()),
            ThemeMode::Dark => ctx.set_visuals(egui::Visuals::dark()),
            ThemeMode::System => {
                // 使用系统默认
            }
        }
    }

    fn setup_fonts_once(&mut self, ctx: &egui::Context) {
        if self.font_loaded {
            return;
        }

        let mut fonts = egui::FontDefinitions::default();
        
        // 使用 egui 内置的中文字体支持
        // 尝试加载系统中文字体，失败则使用默认字体
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

        // 如果没有找到系统字体，使用 egui 的默认字体（支持基本中文）
        if !font_loaded {
            // egui 的默认字体已经包含一些中文字符支持
            println!("警告: 未找到系统中文字体，使用默认字体");
        }

        ctx.set_fonts(fonts);
        self.font_loaded = true;
    }

    fn update_filter(&mut self) {
        // 只有搜索查询改变时才重新过滤
        if self.search_query != self.last_search_query {
            self.filtered_indices = self
                .entries
                .iter()
                .enumerate()
                .filter(|(_, entry)| entry.matches_query(&self.search_query))
                .map(|(i, _)| i)
                .collect();
            
            self.last_search_query = self.search_query.clone();
            self.last_filter_time = Instant::now();
        }
    }

    fn save_config(&mut self) {
        self.config.entries = self.entries.clone();
        let _ = self.config_manager.save_config(&self.config);
    }

    fn add_entry(&mut self) {
        if self.add_path_input.is_empty() {
            return;
        }

        let path = PathBuf::from(&self.add_path_input);
        let name = if self.add_name_input.is_empty() {
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("未命名")
                .to_string()
        } else {
            self.add_name_input.clone()
        };

        let is_directory = path.is_dir();
        let entry = FileEntry::new(path, name, None, Vec::new(), is_directory);
        
        self.entries.push(entry);
        self.save_config();
        
        // 清空输入
        self.add_path_input.clear();
        self.add_name_input.clear();
        self.show_add_dialog = false;
        
        // 强制重新过滤
        self.last_search_query.clear();
    }

    fn remove_entry(&mut self, index: usize) {
        if index < self.entries.len() {
            self.entries.remove(index);
            self.save_config();
            self.last_search_query.clear(); // 强制重新过滤
        }
    }

    fn open_path(&mut self, path: &PathBuf) {
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("explorer")
                .arg(path)
                .spawn();
        }
        
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open")
                .arg(path)
                .spawn();
        }
        
        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open")
                .arg(path)
                .spawn();
        }
    }

    fn render_simple_add_dialog(&mut self, ui: &mut egui::Ui) {
        ui.heading("添加文件/文件夹");
        ui.separator();

        ui.label("路径:");
        ui.text_edit_singleline(&mut self.add_path_input);
        
        if ui.button("选择文件").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                self.add_path_input = path.to_string_lossy().to_string();
            }
        }
        
        if ui.button("选择文件夹").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                self.add_path_input = path.to_string_lossy().to_string();
            }
        }

        ui.add_space(8.0);
        ui.label("名称 (可选):");
        ui.text_edit_singleline(&mut self.add_name_input);

        ui.add_space(16.0);
        ui.horizontal(|ui| {
            if ui.button("添加").clicked() {
                self.add_entry();
            }
            if ui.button("取消").clicked() {
                self.show_add_dialog = false;
                self.add_path_input.clear();
                self.add_name_input.clear();
            }
        });
    }

    fn render_simple_list(&mut self, ui: &mut egui::Ui) {
        ui.heading("文件列表");
        ui.separator();

        let mut to_remove = None;
        let filtered_indices = self.filtered_indices.clone();

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for &index in &filtered_indices {
                    if index >= self.entries.len() {
                        continue;
                    }
                    let entry = &self.entries[index];
                    let entry_path = entry.path.clone();
                    let entry_name = entry.name.clone();
                    let entry_tags = entry.tags.clone();
                    let entry_is_directory = entry.is_directory;
                    
                    ui.horizontal(|ui| {
                        // 简单图标
                        let icon = if entry_is_directory { "📁" } else { "📄" };
                        ui.label(icon);
                        
                        // 文件名（可点击打开）
                        if ui.link(&entry_name).clicked() {
                            self.open_path(&entry_path);
                        }
                        
                        // 标签
                        for tag in &entry_tags {
                            ui.small(format!("#{}", tag));
                        }
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("删除").clicked() {
                                to_remove = Some(index);
                            }
                        });
                    });
                    
                    ui.separator();
                }
                
                if filtered_indices.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        if self.entries.is_empty() {
                            ui.label("还没有添加任何文件");
                            ui.small("点击'添加'按钮开始");
                        } else {
                            ui.label("没有找到匹配的结果");
                        }
                    });
                }
            });

        if let Some(index) = to_remove {
            self.remove_entry(index);
        }
    }
    fn render_simple_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("设置");
        ui.separator();

        ui.label("主题:");
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.theme_mode, ThemeMode::Light, "浅色");
            ui.selectable_value(&mut self.theme_mode, ThemeMode::Dark, "深色");
            ui.selectable_value(&mut self.theme_mode, ThemeMode::System, "系统");
        });

        ui.add_space(16.0);
        ui.label(format!("文件数量: {}", self.entries.len()));
        ui.label(format!("标签数量: {}", self.all_tags.len()));

        if ui.button("清空所有数据").clicked() {
            self.entries.clear();
            self.all_tags.clear();
            self.save_config();
            self.last_search_query.clear();
        }
    }
}

impl eframe::App for FileManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 只在第一次设置字体
        self.setup_fonts_once(ctx);
        
        // 应用主题
        self.apply_theme(ctx);

        // 处理拖拽文件（简化）
        ctx.input(|i| {
            for file in &i.raw.dropped_files {
                if let Some(path) = &file.path {
                    self.add_path_input = path.to_string_lossy().to_string();
                    self.show_add_dialog = true;
                }
            }
        });

        // 顶部面板（简化）
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🗂️ 文件管理器");
                ui.separator();
                
                ui.label("🔍");
                ui.text_edit_singleline(&mut self.search_query);
                
                ui.separator();
                
                if ui.button("添加").clicked() {
                    self.show_add_dialog = !self.show_add_dialog;
                    self.show_settings = false;
                }
                
                if ui.button("设置").clicked() {
                    self.show_settings = !self.show_settings;
                    self.show_add_dialog = false;
                }
            });
        });

        // 侧边面板
        if self.show_add_dialog || self.show_settings {
            egui::SidePanel::right("side").show(ctx, |ui| {
                if self.show_add_dialog {
                    self.render_simple_add_dialog(ui);
                } else if self.show_settings {
                    self.render_simple_settings(ui);
                }
            });
        }

        // 主面板
        egui::CentralPanel::default().show(ctx, |ui| {
            self.update_filter();
            self.render_simple_list(ui);
        });
    }
}