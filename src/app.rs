use crate::config::ConfigManager;
use crate::file_entry::FileEntry;
use chrono::Local;
use eframe::egui;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

pub struct FileManagerApp {
    entries: Vec<FileEntry>,
    current_path_input: String,
    current_name_input: String,
    current_description_input: String,
    current_tag_input: String,
    search_query: String,
    config_manager: ConfigManager,
    selected_entry_index: Option<usize>,
    error_message: Option<String>,
    config: crate::config::Config,
    font_loaded: bool,
    sidebar_expanded: bool,
    show_settings: bool,
    config_path_input: String,
    all_tags: HashSet<String>,
    tag_suggestions: Vec<String>,
    show_tag_suggestions: bool,
    editing_entry: Option<usize>,
    show_compact_view: bool,
    theme_mode: ThemeMode,
    inline_editing: Option<usize>,
    inline_edit_name: String,
    inline_edit_description: String,
    inline_edit_tags: String,
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

        Self {
            entries,
            current_path_input: String::new(),
            current_name_input: String::new(),
            current_description_input: String::new(),
            current_tag_input: String::new(),
            search_query: String::new(),
            config_manager,
            selected_entry_index: None,
            error_message: None,
            config,
            font_loaded: false,
            sidebar_expanded: true,
            show_settings: false,
            config_path_input: String::new(),
            all_tags,
            tag_suggestions: Vec::new(),
            show_tag_suggestions: false,
            editing_entry: None,
            show_compact_view: true,
            theme_mode: ThemeMode::System,
            inline_editing: None,
            inline_edit_name: String::new(),
            inline_edit_description: String::new(),
            inline_edit_tags: String::new(),
        }
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        let dark_mode = match self.theme_mode {
            ThemeMode::Dark => true,
            ThemeMode::Light => false,
            ThemeMode::System => {
                // 检测系统主题
                #[cfg(target_os = "macos")]
                {
                    use std::process::Command;
                    let output = Command::new("defaults")
                        .args(&["read", "-g", "AppleInterfaceStyle"])
                        .output();
                    output
                        .map(|o| String::from_utf8_lossy(&o.stdout).contains("Dark"))
                        .unwrap_or(false)
                }
                #[cfg(not(target_os = "macos"))]
                false
            }
        };

        if dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
    }

    fn get_theme_colors(&self) -> (egui::Color32, egui::Color32, egui::Color32, egui::Color32) {
        match self.theme_mode {
            ThemeMode::Dark => (
                egui::Color32::from_rgb(40, 44, 52),    // 背景色
                egui::Color32::from_rgb(60, 64, 72),    // 卡片背景
                egui::Color32::from_rgb(220, 220, 220), // 主文字
                egui::Color32::from_rgb(150, 150, 150), // 次要文字
            ),
            _ => (
                egui::Color32::from_rgb(248, 249, 250), // 背景色
                egui::Color32::WHITE,                   // 卡片背景
                egui::Color32::from_rgb(51, 51, 51),    // 主文字
                egui::Color32::from_rgb(120, 120, 120), // 次要文字
            ),
        }
    }

    // 按钮样式辅助函数 - 类似 Zed 的简洁风格
    fn create_primary_button(&self, text: &str) -> egui::Button {
        let is_dark = match self.theme_mode {
            ThemeMode::Dark => true,
            ThemeMode::Light => false,
            ThemeMode::System => {
                #[cfg(target_os = "macos")]
                {
                    use std::process::Command;
                    let output = Command::new("defaults")
                        .args(&["read", "-g", "AppleInterfaceStyle"])
                        .output();
                    output
                        .map(|o| String::from_utf8_lossy(&o.stdout).contains("Dark"))
                        .unwrap_or(false)
                }
                #[cfg(not(target_os = "macos"))]
                false
            }
        };

        let (bg_color, text_color) = if is_dark {
            (egui::Color32::from_rgb(0, 122, 255), egui::Color32::WHITE)
        } else {
            (egui::Color32::from_rgb(0, 122, 255), egui::Color32::WHITE)
        };

        egui::Button::new(egui::RichText::new(text).color(text_color))
            .fill(bg_color)
            .rounding(egui::Rounding::same(6.0))
    }

    fn create_secondary_button(&self, text: &str) -> egui::Button {
        let is_dark = match self.theme_mode {
            ThemeMode::Dark => true,
            ThemeMode::Light => false,
            ThemeMode::System => {
                #[cfg(target_os = "macos")]
                {
                    use std::process::Command;
                    let output = Command::new("defaults")
                        .args(&["read", "-g", "AppleInterfaceStyle"])
                        .output();
                    output
                        .map(|o| String::from_utf8_lossy(&o.stdout).contains("Dark"))
                        .unwrap_or(false)
                }
                #[cfg(not(target_os = "macos"))]
                false
            }
        };

        let (bg_color, text_color, border_color) = if is_dark {
            (
                egui::Color32::from_rgb(60, 64, 72),
                egui::Color32::from_rgb(220, 220, 220),
                egui::Color32::from_rgb(100, 100, 100),
            )
        } else {
            (
                egui::Color32::from_rgb(248, 249, 250),
                egui::Color32::from_rgb(51, 51, 51),
                egui::Color32::from_rgb(200, 200, 200),
            )
        };

        egui::Button::new(egui::RichText::new(text).color(text_color))
            .fill(bg_color)
            .stroke(egui::Stroke::new(1.0, border_color))
            .rounding(egui::Rounding::same(6.0))
    }

    fn create_danger_button(&self, text: &str) -> egui::Button {
        egui::Button::new(egui::RichText::new(text).color(egui::Color32::WHITE))
            .fill(egui::Color32::from_rgb(255, 59, 48))
            .rounding(egui::Rounding::same(6.0))
    }

    fn create_small_button(&self, text: &str, button_type: &str) -> egui::Button {
        let is_dark = match self.theme_mode {
            ThemeMode::Dark => true,
            ThemeMode::Light => false,
            ThemeMode::System => {
                #[cfg(target_os = "macos")]
                {
                    use std::process::Command;
                    let output = Command::new("defaults")
                        .args(&["read", "-g", "AppleInterfaceStyle"])
                        .output();
                    output
                        .map(|o| String::from_utf8_lossy(&o.stdout).contains("Dark"))
                        .unwrap_or(false)
                }
                #[cfg(not(target_os = "macos"))]
                false
            }
        };

        let (bg_color, text_color) = match button_type {
            "primary" => (egui::Color32::from_rgb(0, 122, 255), egui::Color32::WHITE),
            "danger" => (egui::Color32::from_rgb(255, 59, 48), egui::Color32::WHITE),
            "success" => (egui::Color32::from_rgb(52, 199, 89), egui::Color32::WHITE),
            _ => {
                if is_dark {
                    (
                        egui::Color32::from_rgb(60, 64, 72),
                        egui::Color32::from_rgb(220, 220, 220),
                    )
                } else {
                    (
                        egui::Color32::from_rgb(248, 249, 250),
                        egui::Color32::from_rgb(51, 51, 51),
                    )
                }
            }
        };

        egui::Button::new(egui::RichText::new(text).size(12.0).color(text_color))
            .fill(bg_color)
            .rounding(egui::Rounding::same(4.0))
    }

    fn create_tag_button(&self, text: &str) -> egui::Button {
        let is_dark = match self.theme_mode {
            ThemeMode::Dark => true,
            ThemeMode::Light => false,
            ThemeMode::System => {
                #[cfg(target_os = "macos")]
                {
                    use std::process::Command;
                    let output = Command::new("defaults")
                        .args(&["read", "-g", "AppleInterfaceStyle"])
                        .output();
                    output
                        .map(|o| String::from_utf8_lossy(&o.stdout).contains("Dark"))
                        .unwrap_or(false)
                }
                #[cfg(not(target_os = "macos"))]
                false
            }
        };

        let (bg_color, text_color) = if is_dark {
            (egui::Color32::from_rgb(88, 86, 214), egui::Color32::WHITE)
        } else {
            (egui::Color32::from_rgb(88, 86, 214), egui::Color32::WHITE)
        };

        egui::Button::new(egui::RichText::new(text).size(12.0).color(text_color))
            .fill(bg_color)
            .rounding(egui::Rounding::same(12.0))
    }

    fn save_config(&mut self) {
        self.config.entries = self.entries.clone();
        if let Err(e) = self.config_manager.save_config(&self.config) {
            self.error_message = Some(format!("保存配置失败: {}", e));
        }
    }

    fn add_entry(
        &mut self,
        path: &str,
        name: &str,
        description: Option<String>,
        tags: Vec<String>,
    ) {
        let path_buf = PathBuf::from(path);
        let is_directory = path_buf.is_dir();

        let final_name = if name.is_empty() {
            path_buf
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(path)
                .to_string()
        } else {
            name.to_string()
        };

        for tag in &tags {
            self.all_tags.insert(tag.clone());
        }

        let entry = FileEntry {
            path: path_buf,
            name: final_name,
            description,
            tags,
            is_directory,
            created_at: Local::now(),
        };

        self.entries.push(entry);
        self.save_config();
        self.error_message = None;
    }

    fn remove_entry(&mut self, index: usize) {
        if index < self.entries.len() {
            self.entries.remove(index);
            self.save_config();
            if self.selected_entry_index == Some(index) {
                self.selected_entry_index = None;
            }
            if self.inline_editing == Some(index) {
                self.inline_editing = None;
            }
        }
    }

    fn update_entry(&mut self, index: usize, name: String, description: String, tags: Vec<String>) {
        if index < self.entries.len() {
            let entry = &mut self.entries[index];
            entry.name = name;
            entry.description = if description.is_empty() {
                None
            } else {
                Some(description)
            };

            for tag in &tags {
                self.all_tags.insert(tag.clone());
            }
            entry.tags = tags;

            self.save_config();
            self.selected_entry_index = None;
            self.editing_entry = None;
            self.inline_editing = None;
        }
    }

    fn start_inline_edit(&mut self, index: usize) {
        if index < self.entries.len() {
            let entry = &self.entries[index];
            self.inline_editing = Some(index);
            self.inline_edit_name = entry.name.clone();
            self.inline_edit_description = entry.description.clone().unwrap_or_default();
            self.inline_edit_tags = entry
                .tags
                .iter()
                .map(|t| format!("#{}", t))
                .collect::<Vec<_>>()
                .join(" ");
        }
    }

    fn save_inline_edit(&mut self) {
        if let Some(index) = self.inline_editing {
            let tags = self.parse_tags(&self.inline_edit_tags);
            self.update_entry(
                index,
                self.inline_edit_name.clone(),
                self.inline_edit_description.clone(),
                tags,
            );
            self.inline_editing = None;
        }
    }

    fn cancel_inline_edit(&mut self) {
        self.inline_editing = None;
        self.inline_edit_name.clear();
        self.inline_edit_description.clear();
        self.inline_edit_tags.clear();
    }

    fn open_path(&mut self, path: &PathBuf) {
        let path_str = path.to_string_lossy();

        #[cfg(target_os = "windows")]
        {
            if path.is_dir() {
                let _ = std::process::Command::new("explorer")
                    .arg(&path_str.to_string())
                    .spawn();
            } else {
                let _ = std::process::Command::new("explorer")
                    .arg("/select,")
                    .arg(&path_str.to_string())
                    .spawn();
            }
        }

        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open")
                .arg(&path_str.to_string())
                .spawn();
        }

        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open")
                .arg(&path_str.to_string())
                .spawn();
        }
    }

    fn render_add_section(&mut self, ui: &mut egui::Ui) {
        let (bg_color, card_color, text_color, secondary_color) = self.get_theme_colors();

        egui::Frame::none()
            .fill(card_color)
            .stroke(egui::Stroke::new(1.0, secondary_color))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(16.0))
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    // 标题
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(" 添加新项目")
                                .size(18.0)
                                .strong()
                                .color(text_color),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("❌").clicked() {
                                self.sidebar_expanded = false;
                            }
                        });
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(16.0);

                    // 路径输入
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("📂 文件/文件夹路径")
                                .color(text_color)
                                .strong(),
                        );
                        ui.add_space(6.0);
                        ui.add_sized(
                            [ui.available_width(), 32.0],
                            egui::TextEdit::singleline(&mut self.current_path_input)
                                .hint_text("输入完整路径或拖拽文件到此处"),
                        );

                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            let folder_btn = self.create_secondary_button("选择文件夹");
                            if ui.add_sized([130.0, 32.0], folder_btn).clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                    self.current_path_input = path.to_string_lossy().to_string();
                                }
                            }

                            let file_btn = self.create_secondary_button("选择文件");
                            if ui.add_sized([130.0, 32.0], file_btn).clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_file() {
                                    self.current_path_input = path.to_string_lossy().to_string();
                                }
                            }
                        });
                    });

                    ui.add_space(20.0);

                    // 名称输入
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("🏷️ 显示名称")
                                .color(text_color)
                                .strong(),
                        );
                        ui.add_space(6.0);
                        ui.add_sized(
                            [ui.available_width(), 32.0],
                            egui::TextEdit::singleline(&mut self.current_name_input)
                                .hint_text("留空将使用文件名"),
                        );
                    });

                    ui.add_space(20.0);

                    // 描述输入
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("📝 描述 (可选)")
                                .color(text_color)
                                .strong(),
                        );
                        ui.add_space(6.0);
                        ui.add_sized(
                            [ui.available_width(), 80.0],
                            egui::TextEdit::multiline(&mut self.current_description_input)
                                .hint_text("添加描述信息..."),
                        );
                    });

                    ui.add_space(20.0);

                    // 标签输入
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("🏷️ 标签 (用空格分隔)")
                                .color(text_color)
                                .strong(),
                        );
                        ui.add_space(6.0);
                        let tag_response = ui.add_sized(
                            [ui.available_width(), 32.0],
                            egui::TextEdit::singleline(&mut self.current_tag_input)
                                .hint_text("例如: #工作 #重要 #项目"),
                        );

                        if tag_response.changed() {
                            self.update_tag_suggestions();
                        }

                        // 标签建议
                        if self.show_tag_suggestions && !self.tag_suggestions.is_empty() {
                            ui.add_space(8.0);
                            egui::Frame::none()
                                .fill(bg_color)
                                .stroke(egui::Stroke::new(1.0, secondary_color))
                                .rounding(egui::Rounding::same(6.0))
                                .inner_margin(egui::Margin::same(8.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new("建议标签:")
                                            .size(12.0)
                                            .color(secondary_color),
                                    );
                                    ui.horizontal_wrapped(|ui| {
                                        for suggestion in &self.tag_suggestions.clone() {
                                            let tag_btn =
                                                self.create_tag_button(&format!("#{}", suggestion));
                                            if ui
                                                .add_sized(
                                                    [suggestion.len() as f32 * 8.0 + 24.0, 24.0],
                                                    tag_btn,
                                                )
                                                .clicked()
                                            {
                                                self.apply_tag_suggestion(suggestion);
                                            }
                                        }
                                    });
                                });
                        }
                    });

                    ui.add_space(24.0);

                    // 操作按钮
                    ui.horizontal(|ui| {
                        let add_btn = self.create_primary_button("添加");
                        if ui.add_sized([100.0, 40.0], add_btn).clicked() {
                            if !self.current_path_input.is_empty() {
                                let tags = self.parse_tags(&self.current_tag_input);
                                let description = if self.current_description_input.is_empty() {
                                    None
                                } else {
                                    Some(self.current_description_input.clone())
                                };
                                let path_input = self.current_path_input.clone();
                                let name_input = self.current_name_input.clone();

                                self.add_entry(&path_input, &name_input, description, tags);

                                // 清空输入框
                                self.current_path_input.clear();
                                self.current_name_input.clear();
                                self.current_description_input.clear();
                                self.current_tag_input.clear();
                                self.show_tag_suggestions = false;
                            }
                        }

                        ui.add_space(8.0);

                        let clear_btn = self.create_secondary_button("清空");
                        if ui.add_sized([100.0, 40.0], clear_btn).clicked() {
                            self.current_path_input.clear();
                            self.current_name_input.clear();
                            self.current_description_input.clear();
                            self.current_tag_input.clear();
                            self.show_tag_suggestions = false;
                        }
                    });

                    // 编辑模式
                    if let Some(editing_index) = self.editing_entry {
                        ui.add_space(24.0);
                        ui.separator();
                        ui.add_space(16.0);
                        ui.label(
                            egui::RichText::new("📝 编辑模式")
                                .size(16.0)
                                .strong()
                                .color(text_color),
                        );

                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            let update_btn = self.create_primary_button("更新");
                            if ui.add_sized([100.0, 40.0], update_btn).clicked() {
                                let tags = self.parse_tags(&self.current_tag_input);
                                self.update_entry(
                                    editing_index,
                                    self.current_name_input.clone(),
                                    self.current_description_input.clone(),
                                    tags,
                                );

                                // 清空输入框
                                self.current_path_input.clear();
                                self.current_name_input.clear();
                                self.current_description_input.clear();
                                self.current_tag_input.clear();
                            }

                            ui.add_space(8.0);

                            let cancel_btn = self.create_danger_button("取消");
                            if ui.add_sized([100.0, 40.0], cancel_btn).clicked() {
                                self.editing_entry = None;
                                self.current_path_input.clear();
                                self.current_name_input.clear();
                                self.current_description_input.clear();
                                self.current_tag_input.clear();
                            }
                        });
                    }

                    // 错误消息
                    if let Some(error) = &self.error_message {
                        ui.add_space(16.0);
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(248, 215, 218))
                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 53, 69)))
                            .rounding(egui::Rounding::same(6.0))
                            .inner_margin(egui::Margin::same(12.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.colored_label(egui::Color32::from_rgb(220, 53, 69), "❌");
                                    ui.colored_label(egui::Color32::from_rgb(114, 28, 36), error);
                                });
                            });
                    }
                });
            });
    }

    fn setup_chinese_fonts(&mut self, ctx: &egui::Context) {
        use egui::FontFamily;

        let mut fonts = egui::FontDefinitions::default();

        if let Some(font_data) = self.get_all_chinese_fonts().into_iter().next() {
            fonts.font_data.insert("chinese_font".to_owned(), font_data);
            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .insert(0, "chinese_font".to_owned());
            fonts
                .families
                .entry(FontFamily::Monospace)
                .or_default()
                .push("chinese_font".to_owned());
        }

        if let Some(emoji_font_data) = self.try_load_emoji_font() {
            fonts
                .font_data
                .insert("emoji_font".to_owned(), emoji_font_data);
            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .insert(0, "emoji_font".to_owned());
        }

        ctx.set_fonts(fonts);
        self.font_loaded = true;
    }

    fn get_all_chinese_fonts(&self) -> Vec<egui::FontData> {
        let mut font_data_vec = Vec::new();

        #[cfg(target_os = "windows")]
        {
            let font_paths = [
                "C:/Windows/Fonts/msyh.ttc",   // 微软雅黑
                "C:/Windows/Fonts/simhei.ttf", // 黑体
                "C:/Windows/Fonts/simsun.ttc", // 宋体
                "C:/Windows/Fonts/simkai.ttf", // 楷体
                "C:/Windows/Fonts/SIMLI.TTF",  // 隶书
            ];

            for font_path in &font_paths {
                if let Ok(font_data) = std::fs::read(font_path) {
                    font_data_vec.push(egui::FontData::from_owned(font_data));
                    break;
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let font_paths = [
                "/System/Library/Fonts/PingFang.ttc",
                "/System/Library/Fonts/STHeiti Medium.ttc",
                "/System/Library/Fonts/Hiragino Sans GB.ttc",
                "/Library/Fonts/Arial Unicode MS.ttf",
            ];

            for font_path in &font_paths {
                if let Ok(font_data) = std::fs::read(font_path) {
                    font_data_vec.push(egui::FontData::from_owned(font_data));
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
                    font_data_vec.push(egui::FontData::from_owned(font_data));
                    break;
                }
            }
        }

        font_data_vec
    }

    fn try_load_emoji_font(&self) -> Option<egui::FontData> {
        #[cfg(target_os = "windows")]
        {
            if let Ok(font_data) = std::fs::read("C:/Windows/Fonts/seguiemj.ttf") {
                return Some(egui::FontData::from_owned(font_data));
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(font_data) = std::fs::read("/System/Library/Fonts/Apple Color Emoji.ttc") {
                return Some(egui::FontData::from_owned(font_data));
            }
        }

        None
    }

    fn render_file_list(&mut self, ui: &mut egui::Ui) {
        let (bg_color, card_color, text_color, secondary_color) = self.get_theme_colors();

        // 工具栏
        egui::Frame::none()
            .fill(card_color)
            .stroke(egui::Stroke::new(1.0, secondary_color))
            .rounding(egui::Rounding::same(6.0))
            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("📋 文件列表")
                            .size(16.0)
                            .strong()
                            .color(text_color),
                    );
                    ui.separator();

                    // 视图切换
                    let view_text = if self.show_compact_view {
                        "紧凑视图"
                    } else {
                        "详细视图"
                    };
                    let view_btn = self.create_secondary_button(view_text);
                    if ui.add_sized([100.0, 24.0], view_btn).clicked() {
                        self.show_compact_view = !self.show_compact_view;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.colored_label(secondary_color, format!("共 {} 项", self.entries.len()));
                    });
                });
            });

        ui.add_space(8.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                let filtered_indices: Vec<usize> = self
                    .entries
                    .iter()
                    .enumerate()
                    .filter(|(_, entry)| entry.matches_query(&self.search_query))
                    .map(|(i, _)| i)
                    .collect();

                let mut to_remove = None;
                let mut to_edit = None;

                if self.show_compact_view {
                    // 紧凑视图 - 类似表格
                    for &index in &filtered_indices {
                        let entry_path = self.entries[index].path.clone();
                        let entry_name = self.entries[index].name.clone();
                        let entry_tags = self.entries[index].tags.clone();
                        let entry_is_directory = self.entries[index].is_directory;
                        let is_selected = self.selected_entry_index == Some(index);

                        let bg_color = if is_selected {
                            egui::Color32::from_rgb(230, 240, 255)
                        } else if index % 2 == 0 {
                            card_color
                        } else {
                            bg_color
                        };

                        // 紧凑的一行式布局
                        egui::Frame::none()
                            .fill(bg_color)
                            .stroke(egui::Stroke::new(0.5, secondary_color))
                            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // 图标 + 名称 (固定宽度)
                                    ui.allocate_ui_with_layout(
                                        [240.0, 24.0].into(),
                                        egui::Layout::left_to_right(egui::Align::Center),
                                        |ui| {
                                            let icon =
                                                if entry_is_directory { "📁" } else { "📄" };
                                            ui.label(egui::RichText::new(icon).size(16.0));

                                            let name_response = ui.add(
                                                egui::Label::new(
                                                    egui::RichText::new(&entry_name)
                                                        .size(14.0)
                                                        .strong()
                                                        .color(text_color),
                                                )
                                                .sense(egui::Sense::click())
                                                .truncate(true),
                                            );

                                            if name_response.clicked() {
                                                self.open_path(&entry_path);
                                            }

                                            if name_response.hovered() {
                                                ui.ctx().set_cursor_icon(
                                                    egui::CursorIcon::PointingHand,
                                                );
                                            }
                                        },
                                    );

                                    // 标签 (动态宽度)
                                    if !entry_tags.is_empty() {
                                        ui.separator();
                                        for tag in &entry_tags {
                                            let tag_btn =
                                                self.create_tag_button(&format!("#{}", tag));
                                            let _ = ui.add_sized(
                                                [tag.len() as f32 * 8.0 + 16.0, 20.0],
                                                tag_btn,
                                            );
                                        }
                                    }

                                    // 操作按钮 (右对齐)
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            let delete_btn =
                                                self.create_small_button("删除", "danger");
                                            if ui.add_sized([60.0, 24.0], delete_btn).clicked() {
                                                to_remove = Some(index);
                                            }

                                            let edit_btn =
                                                self.create_small_button("编辑", "primary");
                                            if ui.add_sized([60.0, 24.0], edit_btn).clicked() {
                                                to_edit = Some(index);
                                            }

                                            let open_btn =
                                                self.create_small_button("打开", "success");
                                            if ui.add_sized([60.0, 24.0], open_btn).clicked() {
                                                self.open_path(&entry_path);
                                            }
                                        },
                                    );
                                });
                            });
                    }
                } else {
                    // 详细视图 - 卡片式，支持内联编辑
                    for &index in &filtered_indices {
                        let entry_path = self.entries[index].path.clone();
                        let entry_name = self.entries[index].name.clone();
                        let entry_description = self.entries[index].description.clone();
                        let entry_tags = self.entries[index].tags.clone();
                        let entry_is_directory = self.entries[index].is_directory;
                        let entry_created_at = self.entries[index].created_at;
                        let is_selected = self.selected_entry_index == Some(index);
                        let is_inline_editing = self.inline_editing == Some(index);

                        let card_bg = if is_selected {
                            egui::Color32::from_rgb(230, 240, 255)
                        } else {
                            card_color
                        };

                        egui::Frame::none()
                            .fill(card_bg)
                            .stroke(egui::Stroke::new(
                                1.0,
                                if is_inline_editing {
                                    egui::Color32::from_rgb(52, 152, 219)
                                } else {
                                    secondary_color
                                },
                            ))
                            .rounding(egui::Rounding::same(8.0))
                            .inner_margin(egui::Margin::same(16.0))
                            .shadow(if is_inline_editing {
                                egui::epaint::Shadow {
                                    extrusion: 4.0,
                                    color: egui::Color32::from_rgba_unmultiplied(52, 152, 219, 40),
                                }
                            } else {
                                egui::epaint::Shadow::NONE
                            })
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    // 主信息行
                                    ui.horizontal(|ui| {
                                        let icon = if entry_is_directory { "📁" } else { "📄" };
                                        ui.label(egui::RichText::new(icon).size(20.0));

                                        ui.add_space(8.0);

                                        if is_inline_editing {
                                            // 内联编辑名称
                                            ui.add_sized(
                                                [200.0, 24.0],
                                                egui::TextEdit::singleline(
                                                    &mut self.inline_edit_name,
                                                ),
                                            );
                                        } else {
                                            let name_response = ui.add(
                                                egui::Label::new(
                                                    egui::RichText::new(&entry_name)
                                                        .size(16.0)
                                                        .strong()
                                                        .color(text_color),
                                                )
                                                .sense(egui::Sense::click()),
                                            );

                                            if name_response.clicked() {
                                                self.open_path(&entry_path);
                                            }

                                            if name_response.hovered() {
                                                ui.ctx().set_cursor_icon(
                                                    egui::CursorIcon::PointingHand,
                                                );
                                            }
                                        }

                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if is_inline_editing {
                                                    // 内联编辑操作按钮
                                                    let save_btn =
                                                        self.create_small_button("保存", "success");
                                                    if ui
                                                        .add_sized([60.0, 24.0], save_btn)
                                                        .clicked()
                                                    {
                                                        self.save_inline_edit();
                                                    }

                                                    let cancel_btn = self
                                                        .create_small_button("取消", "secondary");
                                                    if ui
                                                        .add_sized([60.0, 24.0], cancel_btn)
                                                        .clicked()
                                                    {
                                                        self.cancel_inline_edit();
                                                    }
                                                } else {
                                                    // 普通操作按钮
                                                    let delete_btn =
                                                        self.create_small_button("删除", "danger");
                                                    if ui
                                                        .add_sized([60.0, 24.0], delete_btn)
                                                        .clicked()
                                                    {
                                                        to_remove = Some(index);
                                                    }

                                                    let inline_edit_btn = self.create_small_button(
                                                        "快速编辑",
                                                        "secondary",
                                                    );
                                                    if ui
                                                        .add_sized([80.0, 24.0], inline_edit_btn)
                                                        .clicked()
                                                    {
                                                        self.start_inline_edit(index);
                                                    }

                                                    let edit_btn = self
                                                        .create_small_button("详细编辑", "primary");
                                                    if ui
                                                        .add_sized([80.0, 24.0], edit_btn)
                                                        .clicked()
                                                    {
                                                        to_edit = Some(index);
                                                    }
                                                }
                                            },
                                        );
                                    });

                                    ui.add_space(8.0);

                                    // 路径
                                    ui.horizontal(|ui| {
                                        ui.colored_label(
                                            secondary_color,
                                            egui::RichText::new(format!(
                                                "📍 {}",
                                                entry_path.display()
                                            ))
                                            .size(12.0),
                                        );
                                    });

                                    ui.add_space(6.0);

                                    // 描述
                                    if is_inline_editing {
                                        // 内联编辑描述
                                        ui.label(
                                            egui::RichText::new("📝 描述:")
                                                .size(12.0)
                                                .color(text_color),
                                        );
                                        ui.add_sized(
                                            [ui.available_width(), 60.0],
                                            egui::TextEdit::multiline(
                                                &mut self.inline_edit_description,
                                            )
                                            .hint_text("添加描述..."),
                                        );
                                    } else if let Some(description) = &entry_description {
                                        ui.horizontal(|ui| {
                                            ui.colored_label(
                                                text_color,
                                                egui::RichText::new(format!("📄 {}", description))
                                                    .size(13.0),
                                            );
                                        });
                                    }

                                    ui.add_space(6.0);

                                    // 标签
                                    if is_inline_editing {
                                        // 内联编辑标签
                                        ui.label(
                                            egui::RichText::new("🏷️ 标签:")
                                                .size(12.0)
                                                .color(text_color),
                                        );
                                        ui.add_sized(
                                            [ui.available_width(), 24.0],
                                            egui::TextEdit::singleline(&mut self.inline_edit_tags)
                                                .hint_text("例如: #工作 #重要"),
                                        );
                                    } else {
                                        ui.horizontal(|ui| {
                                            if !entry_tags.is_empty() {
                                                for tag in &entry_tags {
                                                    let tag_btn =
                                                        self.create_tag_button(&format!("{}", tag));
                                                    if ui
                                                        .add_sized(
                                                            [tag.len() as f32 * 8.0 + 32.0, 24.0],
                                                            tag_btn,
                                                        )
                                                        .clicked()
                                                    {
                                                        self.search_query = tag.clone();
                                                    }
                                                }
                                            }

                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    ui.colored_label(
                                                        secondary_color,
                                                        egui::RichText::new(format!(
                                                            "📅 {}",
                                                            entry_created_at.format("%m-%d %H:%M")
                                                        ))
                                                        .size(11.0),
                                                    );
                                                },
                                            );
                                        });
                                    }
                                });
                            });

                        ui.add_space(6.0);
                    }
                }

                // 处理操作
                if let Some(index) = to_remove {
                    self.remove_entry(index);
                }

                if let Some(index) = to_edit {
                    let entry = &self.entries[index];
                    self.editing_entry = Some(index);
                    self.selected_entry_index = Some(index);
                    self.current_path_input = entry.path.to_string_lossy().to_string();
                    self.current_name_input = entry.name.clone();
                    self.current_description_input = entry.description.clone().unwrap_or_default();
                    self.current_tag_input = entry
                        .tags
                        .iter()
                        .map(|t| format!("#{}", t))
                        .collect::<Vec<_>>()
                        .join(" ");
                    self.sidebar_expanded = true;
                    self.show_settings = false;
                }

                // 空状态
                if filtered_indices.is_empty() && !self.entries.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(60.0);
                        ui.label(egui::RichText::new("🔍").size(64.0));
                        ui.add_space(16.0);
                        ui.label(
                            egui::RichText::new("没有找到匹配的结果")
                                .size(18.0)
                                .color(secondary_color),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new("尝试修改搜索条件")
                                .size(14.0)
                                .color(secondary_color),
                        );
                    });
                } else if self.entries.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(80.0);
                        ui.label(egui::RichText::new("📁").size(80.0));
                        ui.add_space(20.0);
                        ui.label(
                            egui::RichText::new("还没有添加任何文件或文件夹")
                                .size(20.0)
                                .color(text_color),
                        );
                        ui.add_space(12.0);
                        ui.label(
                            egui::RichText::new("点击右上角 '添加' 按钮开始，或拖拽文件到窗口")
                                .size(16.0)
                                .color(secondary_color),
                        );
                    });
                }
            });
    }

    fn update_tag_suggestions(&mut self) {
        if self.current_tag_input.ends_with('#') || self.current_tag_input.contains(" #") {
            let current_prefix = self
                .current_tag_input
                .split_whitespace()
                .last()
                .unwrap_or("")
                .trim_start_matches('#');

            self.tag_suggestions = self
                .all_tags
                .iter()
                .filter(|tag| tag.starts_with(current_prefix) && !current_prefix.is_empty())
                .take(5)
                .cloned()
                .collect();

            self.show_tag_suggestions = !self.tag_suggestions.is_empty();
        } else {
            self.show_tag_suggestions = false;
        }
    }

    fn apply_tag_suggestion(&mut self, suggestion: &str) {
        let mut parts: Vec<String> = self
            .current_tag_input
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        if let Some(last) = parts.last_mut() {
            if last.starts_with('#') {
                *last = format!("#{}", suggestion);
            }
        }
        self.current_tag_input = parts.join(" ") + " ";
        self.show_tag_suggestions = false;
    }

    fn parse_tags(&self, input: &str) -> Vec<String> {
        input
            .split_whitespace()
            .filter_map(|tag| {
                let tag = tag.trim_start_matches('#').trim();
                if tag.is_empty() {
                    None
                } else {
                    Some(tag.to_string())
                }
            })
            .collect()
    }

    fn render_settings_section(&mut self, ui: &mut egui::Ui) {
        let (bg_color, card_color, text_color, secondary_color) = self.get_theme_colors();

        egui::Frame::none()
            .fill(card_color)
            .stroke(egui::Stroke::new(1.0, secondary_color))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(16.0))
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("⚙️ 设置")
                                .size(18.0)
                                .strong()
                                .color(text_color),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("❌").clicked() {
                                self.show_settings = false;
                            }
                        });
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(16.0);

                    // 主题设置
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("🎨 主题模式")
                                .strong()
                                .color(text_color),
                        );
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            ui.radio_value(&mut self.theme_mode, ThemeMode::Light, "☀️ 浅色");
                            ui.radio_value(&mut self.theme_mode, ThemeMode::Dark, "🌙 深色");
                            ui.radio_value(&mut self.theme_mode, ThemeMode::System, "💻 跟随系统");
                        });
                    });

                    ui.add_space(20.0);

                    // 配置文件路径设置
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("📂 配置文件路径")
                                .strong()
                                .color(text_color),
                        );
                        ui.add_space(8.0);
                        ui.add_sized(
                            [ui.available_width(), 32.0],
                            egui::TextEdit::singleline(&mut self.config_path_input)
                                .hint_text("配置文件保存位置"),
                        );

                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            let save_btn = self.create_primary_button("保存路径");
                            if ui.add_sized([100.0, 32.0], save_btn).clicked() {
                                // 这里可以添加更改配置文件路径的逻辑
                            }

                            let browse_btn = self.create_secondary_button("浏览");
                            if ui.add_sized([100.0, 32.0], browse_btn).clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("JSON", &["json"])
                                    .save_file()
                                {
                                    self.config_path_input = path.to_string_lossy().to_string();
                                }
                            }
                        });
                    });

                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(16.0);

                    // 其他设置
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("🔧 显示设置")
                                .strong()
                                .color(text_color),
                        );
                        ui.add_space(12.0);

                        ui.checkbox(&mut self.show_compact_view, "默认使用紧凑视图");

                        ui.add_space(16.0);

                        let clear_btn = self.create_danger_button("清空所有数据");
                        if ui.add_sized([150.0, 32.0], clear_btn).clicked() {
                            self.entries.clear();
                            self.save_config();
                        }
                    });

                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(16.0);

                    // 统计信息
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("📊 统计信息")
                                .strong()
                                .color(text_color),
                        );
                        ui.add_space(12.0);

                        egui::Frame::none()
                            .fill(bg_color)
                            .stroke(egui::Stroke::new(1.0, secondary_color))
                            .rounding(egui::Rounding::same(6.0))
                            .inner_margin(egui::Margin::same(12.0))
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.label(format!("📄 总文件数: {}", self.entries.len()));
                                    ui.label(format!("🏷️ 总标签数: {}", self.all_tags.len()));
                                    ui.label(format!(
                                        "📁 文件夹数: {}",
                                        self.entries.iter().filter(|e| e.is_directory).count()
                                    ));
                                    ui.label(format!(
                                        "📄 文件数: {}",
                                        self.entries.iter().filter(|e| !e.is_directory).count()
                                    ));
                                });
                            });
                    });
                });
            });
    }
}

impl eframe::App for FileManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 应用主题
        self.apply_theme(ctx);

        // 设置中文字体
        if !self.font_loaded {
            self.setup_chinese_fonts(ctx);
        }

        // 处理拖拽文件
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                for file in &i.raw.dropped_files {
                    if let Some(path) = &file.path {
                        self.add_entry(
                            &path.to_string_lossy(),
                            "",
                            None,
                            vec!["拖拽添加".to_string()],
                        );
                    }
                }
            }
        });

        let (bg_color, card_color, text_color, secondary_color) = self.get_theme_colors();

        // 顶部工具栏
        egui::TopBottomPanel::top("top_panel")
            .exact_height(48.0)
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(52, 73, 94))
                    .inner_margin(egui::Margin::symmetric(16.0, 12.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    // 标题
                    ui.label(
                        egui::RichText::new("🗂️ 文件管理器")
                            .size(18.0)
                            .color(egui::Color32::WHITE)
                            .strong(),
                    );

                    ui.separator();

                    // 搜索框
                    ui.label(egui::RichText::new("🔍").color(egui::Color32::WHITE));
                    let _search_response = ui.add_sized(
                        [220.0, 28.0],
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("搜索文件、标签..."),
                    );

                    ui.separator();

                    // 主题切换按钮
                    let theme_text = match self.theme_mode {
                        ThemeMode::Light => "☀️",
                        ThemeMode::Dark => "🌙",
                        ThemeMode::System => "💻",
                    };
                    let theme_btn = self.create_secondary_button(theme_text);
                    if ui.add_sized([32.0, 28.0], theme_btn).clicked() {
                        self.theme_mode = match self.theme_mode {
                            ThemeMode::Light => ThemeMode::Dark,
                            ThemeMode::Dark => ThemeMode::System,
                            ThemeMode::System => ThemeMode::Light,
                        };
                    }

                    // 操作按钮
                    let add_btn = self.create_primary_button("添加");
                    if ui.add_sized([70.0, 28.0], add_btn).clicked() {
                        self.sidebar_expanded = true;
                        self.show_settings = false;
                    }

                    let settings_btn = self.create_secondary_button("设置");
                    if ui.add_sized([70.0, 28.0], settings_btn).clicked() {
                        self.show_settings = !self.show_settings;
                        self.sidebar_expanded = false;
                    }
                });
            });

        // 侧边栏
        if self.sidebar_expanded || self.show_settings {
            egui::SidePanel::right("side_panel")
                .resizable(true)
                .default_width(360.0)
                .min_width(320.0)
                .max_width(480.0)
                .frame(
                    egui::Frame::none()
                        .fill(bg_color)
                        .inner_margin(egui::Margin::same(8.0)),
                )
                .show(ctx, |ui| {
                    if self.show_settings {
                        self.render_settings_section(ui);
                    } else {
                        self.render_add_section(ui);
                    }
                });
        }

        // 主内容区域
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(bg_color)
                    .inner_margin(egui::Margin::same(12.0)),
            )
            .show(ctx, |ui| {
                self.render_file_list(ui);
            });
    }
}
