use crate::config::{ConfigManager, AppConfig};
use crate::file_entry::FileEntry;
use eframe::egui;
use std::path::{Path, PathBuf};

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
    config: AppConfig,
    font_loaded: bool,
    sidebar_expanded: bool,
}

impl Default for FileManagerApp {
    fn default() -> Self {
        Self::new()
    }
}

impl FileManagerApp {
    pub fn new() -> Self {
        let config_manager = ConfigManager::new();
        let config = config_manager.load_config();
        let entries = config.entries.clone();

        Self {
            entries,
            config_manager,
            current_path_input: String::new(),
            current_name_input: String::new(),
            current_description_input: String::new(),
            current_tag_input: String::new(),
            search_query: String::new(),
            selected_entry_index: None,
            error_message: None,
            config,
            font_loaded: false,
            sidebar_expanded: true,
        }
    }

    fn save_config(&self) {
        let mut config = self.config.clone();
        config.entries = self.entries.clone();
        if let Err(e) = self.config_manager.save_config(&config) {
            eprintln!("保存配置失败: {}", e);
        }
    }

    fn add_entry(&mut self, path: &str, name: &str, description: Option<String>, tags: Vec<String>) {
        let path_buf = PathBuf::from(path.trim());

        if !path_buf.exists() {
            self.error_message = Some(format!("路径不存在: {}", path));
            return;
        }

        // 检查是否已存在
        if self.entries.iter().any(|e| e.path == path_buf) {
            self.error_message = Some("该路径已存在".to_string());
            return;
        }

        let entry_name = if name.trim().is_empty() {
            path_buf.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("未命名")
                .to_string()
        } else {
            name.trim().to_string()
        };

        let entry = FileEntry::new(path_buf, entry_name, description, tags);
        self.entries.push(entry);
        self.save_config();
        self.error_message = None;
    }

    fn remove_entry(&mut self, index: usize) {
        if index < self.entries.len() {
            self.entries.remove(index);
            self.save_config();
        }
    }

    fn update_entry(&mut self, index: usize, path: &str, name: &str, description: Option<String>, tags: Vec<String>) {
        let path_buf = PathBuf::from(path.trim());

        if !path_buf.exists() {
            self.error_message = Some(format!("路径不存在: {}", path));
            return;
        }

        if index < self.entries.len() {
            let entry_name = if name.trim().is_empty() {
                path_buf.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("未命名")
                    .to_string()
            } else {
                name.trim().to_string()
            };

            self.entries[index] = FileEntry::new(path_buf, entry_name, description, tags);
            self.save_config();
            self.selected_entry_index = None;
            self.current_path_input.clear();
            self.current_name_input.clear();
            self.current_description_input.clear();
            self.current_tag_input.clear();
            self.error_message = None;
        }
    }

    fn open_path(&self, path: &Path) {
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("explorer")
                .arg(path.to_string_lossy().to_string())
                .spawn();
        }

        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open")
                .arg(path.to_string_lossy().to_string())
                .spawn();
        }

        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open")
                .arg(path.to_string_lossy().to_string())
                .spawn();
        }
    }

    fn render_add_section(&mut self, ui: &mut egui::Ui) {
        // 美化的添加面板
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(250, 251, 252))
            .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(220, 220, 220)))
            .rounding(egui::Rounding::same(12.0))
            .inner_margin(egui::Margin::same(16.0))
            .shadow(egui::epaint::Shadow {
                extrusion: 3.0,
                color: egui::Color32::from_black_alpha(15),
            })
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    // 标题区域
                    ui.horizontal(|ui| {
                        let title_text = if self.selected_entry_index.is_some() {
                            "📝 编辑条目"
                        } else {
                            "➕ 添加新条目"
                        };
                        
                        ui.label(
                            egui::RichText::new(title_text)
                                .size(18.0)
                                .color(egui::Color32::from_rgb(70, 130, 180))
                                .strong()
                        );
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if self.selected_entry_index.is_some() {
                                let cancel_btn = egui::Button::new("❌ 取消编辑")
                                    .fill(egui::Color32::from_rgb(255, 140, 0))
                                    .rounding(egui::Rounding::same(6.0));
                                if ui.add_sized([80.0, 24.0], cancel_btn).clicked() {
                                    self.selected_entry_index = None;
                                    self.current_path_input.clear();
                                    self.current_name_input.clear();
                                    self.current_description_input.clear();
                                    self.current_tag_input.clear();
                                }
                            }
                        });
                    });

                    ui.add_space(12.0);

                    // 名称输入区域
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("📝 显示名称")
                                .size(14.0)
                                .color(egui::Color32::from_rgb(80, 80, 80))
                                .strong()
                        );
                        ui.add_space(4.0);
                        ui.add_sized(
                            [ui.available_width(), 32.0],
                            egui::TextEdit::singleline(&mut self.current_name_input)
                                .hint_text("为这个条目起个好记的名字...")
                                .font(egui::TextStyle::Body)
                        );
                    });

                    ui.add_space(12.0);

                    // 路径输入区域
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("📍 文件/文件夹路径")
                                .size(14.0)
                                .color(egui::Color32::from_rgb(80, 80, 80))
                                .strong()
                        );
                        ui.add_space(4.0);
                        
                        ui.horizontal(|ui| {
                            let _path_input = ui.add_sized(
                                [ui.available_width() - 170.0, 32.0],
                                egui::TextEdit::singleline(&mut self.current_path_input)
                                    .hint_text("输入路径或使用浏览按钮选择...")
                                    .font(egui::TextStyle::Body)
                            );
                            
                            let folder_btn = egui::Button::new("📁 文件夹")
                                .fill(egui::Color32::from_rgb(100, 149, 237))
                                .rounding(egui::Rounding::same(6.0));
                            if ui.add_sized([80.0, 32.0], folder_btn).clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                    self.current_path_input = path.to_string_lossy().to_string();
                                }
                            }
                            
                            let file_btn = egui::Button::new("📄 文件")
                                .fill(egui::Color32::from_rgb(34, 139, 34))
                                .rounding(egui::Rounding::same(6.0));
                            if ui.add_sized([80.0, 32.0], file_btn).clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_file() {
                                    self.current_path_input = path.to_string_lossy().to_string();
                                }
                            }
                        });
                    });

                    ui.add_space(12.0);

                    // 描述输入区域
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("📄 描述 (可选)")
                                .size(14.0)
                                .color(egui::Color32::from_rgb(80, 80, 80))
                                .strong()
                        );
                        ui.add_space(4.0);
                        ui.add_sized(
                            [ui.available_width(), 60.0],
                            egui::TextEdit::multiline(&mut self.current_description_input)
                                .hint_text("为这个条目添加详细描述...")
                                .font(egui::TextStyle::Body)
                        );
                    });

                    ui.add_space(12.0);

                    // 标签输入区域
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("🏷️ 标签 (用逗号分隔)")
                                .size(14.0)
                                .color(egui::Color32::from_rgb(80, 80, 80))
                                .strong()
                        );
                        ui.add_space(4.0);
                        ui.add_sized(
                            [ui.available_width(), 32.0],
                            egui::TextEdit::singleline(&mut self.current_tag_input)
                                .hint_text("例如: 工作, 重要, 项目...")
                                .font(egui::TextStyle::Body)
                        );
                    });

                    ui.add_space(16.0);

                    // 操作按钮区域
                    ui.vertical(|ui| {
                        let button_text = if self.selected_entry_index.is_some() {
                            "🔄 更新条目"
                        } else {
                            "➕ 添加到列表"
                        };

                        let button_color = if self.selected_entry_index.is_some() {
                            egui::Color32::from_rgb(70, 130, 180)  // 蓝色用于更新
                        } else {
                            egui::Color32::from_rgb(34, 139, 34)   // 绿色用于添加
                        };

                        let main_btn = egui::Button::new(
                            egui::RichText::new(button_text)
                                .size(16.0)
                                .color(egui::Color32::WHITE)
                                .strong()
                        )
                        .fill(button_color)
                        .rounding(egui::Rounding::same(8.0));
                        
                        if ui.add_sized([ui.available_width(), 40.0], main_btn).clicked() {
                            let tags: Vec<String> = self
                                .current_tag_input
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect();

                            let path_input = self.current_path_input.clone();
                            let name_input = self.current_name_input.clone();
                            let description_input = if self.current_description_input.trim().is_empty() {
                                None
                            } else {
                                Some(self.current_description_input.clone())
                            };
                            
                            if let Some(index) = self.selected_entry_index {
                                self.update_entry(index, &path_input, &name_input, description_input, tags);
                            } else {
                                self.add_entry(&path_input, &name_input, description_input, tags);

                                if self.error_message.is_none() {
                                    self.current_path_input.clear();
                                    self.current_name_input.clear();
                                    self.current_description_input.clear();
                                    self.current_tag_input.clear();
                                }
                            }
                        }

                        ui.add_space(8.0);

                        // 快捷键提示
                        ui.horizontal(|ui| {
                            ui.colored_label(
                                egui::Color32::from_rgb(120, 120, 120),
                                egui::RichText::new("💡 提示: 可直接拖拽文件到窗口快速添加")
                                    .size(12.0)
                            );
                        });
                    });

                    // 错误信息显示
                    if let Some(error) = &self.error_message.clone() {
                        ui.add_space(12.0);
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(255, 245, 245))
                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 99, 99)))
                            .rounding(egui::Rounding::same(6.0))
                            .inner_margin(egui::Margin::same(8.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.colored_label(
                                        egui::Color32::from_rgb(220, 53, 69),
                                        egui::RichText::new("❌ 错误").strong()
                                    );
                                    ui.colored_label(
                                        egui::Color32::from_rgb(185, 28, 28),
                                        error
                                    );
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let close_btn = egui::Button::new("✖")
                                            .fill(egui::Color32::from_rgb(220, 53, 69))
                                            .rounding(egui::Rounding::same(4.0));
                                        if ui.add_sized([24.0, 20.0], close_btn).clicked() {
                                            self.error_message = None;
                                        }
                                    });
                                });
                            });
                    }
                });
            });
    }

    fn setup_chinese_fonts(&mut self, ctx: &egui::Context) {
        if self.font_loaded {
            return;
        }

        let mut fonts = egui::FontDefinitions::default();
        
        // 尝试加载多个中文字体以获得更好的字符覆盖率
        let font_sources = self.get_all_chinese_fonts();
        
        for (name, data) in font_sources {
            fonts.font_data.insert(name.clone(), egui::FontData::from_owned(data));
            
            // 将字体添加到字体族
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, name.clone());

            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push(name);
        }

        // 确保有 emoji 支持
        if let Some(emoji_font) = self.try_load_emoji_font() {
            fonts.font_data.insert("emoji".to_owned(), egui::FontData::from_owned(emoji_font));
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "emoji".to_owned());
        }

        ctx.set_fonts(fonts);
        
        // 设置UI风格以优化中文显示
        let mut style = ctx.style().as_ref().clone();
        style.text_styles = [
            (egui::TextStyle::Small, egui::FontId::proportional(10.0)),
            (egui::TextStyle::Body, egui::FontId::proportional(14.0)),
            (egui::TextStyle::Monospace, egui::FontId::monospace(13.0)),
            (egui::TextStyle::Button, egui::FontId::proportional(14.0)),
            (egui::TextStyle::Heading, egui::FontId::proportional(18.0)),
        ].into();
        ctx.set_style(style);
        
        self.font_loaded = true;
    }

    fn get_all_chinese_fonts(&self) -> Vec<(String, Vec<u8>)> {
        let mut fonts = Vec::new();
        
        // macOS 系统字体
        #[cfg(target_os = "macos")]
        {
            let font_paths = vec![
                ("pingfang", "/System/Library/Fonts/PingFang.ttc"),
                ("heiti", "/System/Library/Fonts/STHeiti Light.ttc"),
                ("hiragino", "/System/Library/Fonts/Hiragino Sans GB.ttc"),
                ("arial_unicode", "/System/Library/Fonts/Arial Unicode.ttf"),
            ];
            
            for (name, path) in font_paths {
                if let Ok(data) = std::fs::read(path) {
                    fonts.push((name.to_string(), data));
                }
            }
        }

        // Windows 系统字体
        #[cfg(target_os = "windows")]
        {
            let font_paths = vec![
                ("msyh", "C:\\Windows\\Fonts\\msyh.ttc"),      // 微软雅黑
                ("simhei", "C:\\Windows\\Fonts\\simhei.ttf"),  // 黑体
                ("simsun", "C:\\Windows\\Fonts\\simsun.ttc"),  // 宋体
                ("simkai", "C:\\Windows\\Fonts\\simkai.ttf"),  // 楷体
                ("arial_unicode", "C:\\Windows\\Fonts\\ARIALUNI.TTF"), // Arial Unicode MS
            ];
            
            for (name, path) in font_paths {
                if let Ok(data) = std::fs::read(path) {
                    fonts.push((name.to_string(), data));
                }
            }
        }

        // Linux 系统字体
        #[cfg(target_os = "linux")]
        {
            let font_paths = vec![
                ("noto_cjk", "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"),
                ("wqy_microhei", "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc"),
                ("wqy_zenhei", "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc"),
                ("source_han", "/usr/share/fonts/opentype/source-han-sans/SourceHanSansCN-Regular.otf"),
            ];
            
            for (name, path) in font_paths {
                if let Ok(data) = std::fs::read(path) {
                    fonts.push((name.to_string(), data));
                }
            }
        }

        fonts
    }

    fn try_load_emoji_font(&self) -> Option<Vec<u8>> {
        #[cfg(target_os = "macos")]
        {
            if let Ok(data) = std::fs::read("/System/Library/Fonts/Apple Color Emoji.ttc") {
                return Some(data);
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(data) = std::fs::read("C:\\Windows\\Fonts\\seguiemj.ttf") {
                return Some(data);
            }
        }

        #[cfg(target_os = "linux")]
        {
            let paths = vec![
                "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf",
                "/usr/share/fonts/TTF/NotoColorEmoji.ttf",
            ];
            for path in paths {
                if let Ok(data) = std::fs::read(path) {
                    return Some(data);
                }
            }
        }

        None
    }

    fn render_file_list(&mut self, ui: &mut egui::Ui) {
        // 美化的头部区域
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.strong("📋 已保存的路径");
                ui.separator();
                ui.colored_label(
                    egui::Color32::from_rgb(100, 149, 237),
                    format!("共 {} 项", self.entries.len())
                );
                if !self.search_query.is_empty() {
                    ui.separator();
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 140, 0),
                        format!("筛选: {}", self.search_query)
                    );
                }
                ui.add_space(8.0);
            });
        });

        ui.add_space(4.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                // 收集匹配的条目索引
                let filtered_indices: Vec<usize> = self
                    .entries
                    .iter()
                    .enumerate()
                    .filter(|(_, entry)| entry.matches_query(&self.search_query))
                    .map(|(i, _)| i)
                    .collect();

                let mut to_remove = None;
                let mut to_edit = None;

                for &index in &filtered_indices {
                    let entry = &self.entries[index];
                    let is_selected = self.selected_entry_index == Some(index);

                    // 克隆需要的值以避免借用冲突
                    let entry_path = entry.path.clone();
                    let entry_name = entry.name.clone();
                    let entry_description = entry.description.clone();
                    let entry_tags = entry.tags.clone();
                    let entry_is_directory = entry.is_directory;
                    let entry_created_at = entry.created_at;

                    // 为选中项添加背景色
                    let bg_color = if is_selected {
                        Some(egui::Color32::from_rgb(230, 240, 255))
                    } else {
                        None
                    };

                    // 使用更美观的卡片式设计
                    egui::Frame::none()
                        .fill(bg_color.unwrap_or(egui::Color32::from_rgb(248, 249, 250)))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 220, 220)))
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::same(12.0))
                        .shadow(egui::epaint::Shadow {
                            extrusion: 2.0,
                            color: egui::Color32::from_black_alpha(20),
                        })
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                // 主要信息行
                                ui.horizontal(|ui| {
                                    // 图标和名称
                                    let icon = if entry_is_directory { "📁" } else { "📄" };
                                    ui.label(egui::RichText::new(icon).size(20.0));
                                    
                                    ui.add_space(8.0);
                                    
                                    // 显示名称（可点击，确保中文居中对齐）
                                    let name_response = ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(&entry_name)
                                                .size(16.0)
                                                .color(egui::Color32::from_rgb(51, 51, 51))
                                                .strong()
                                                .family(egui::FontFamily::Proportional)
                                        )
                                        .sense(egui::Sense::click())
                                        .wrap(false)
                                    );
                                    
                                    if name_response.clicked() {
                                        self.open_path(&entry_path);
                                    }
                                    
                                    if name_response.hovered() {
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                    }

                                    // 操作按钮
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        // 美化的删除按钮
                                        let delete_btn = egui::Button::new("🗑️")
                                            .fill(egui::Color32::from_rgb(255, 99, 99))
                                            .rounding(egui::Rounding::same(4.0));
                                        if ui.add_sized([32.0, 24.0], delete_btn).clicked() {
                                            to_remove = Some(index);
                                        }

                                        ui.add_space(4.0);

                                        // 美化的编辑按钮
                                        let edit_btn = egui::Button::new("📝")
                                            .fill(egui::Color32::from_rgb(100, 149, 237))
                                            .rounding(egui::Rounding::same(4.0));
                                        if ui.add_sized([32.0, 24.0], edit_btn).clicked() {
                                            to_edit = Some(index);
                                        }
                                    });
                                });

                                ui.add_space(6.0);

                                // 路径行（小字显示）
                                ui.horizontal(|ui| {
                                    ui.colored_label(
                                        egui::Color32::from_rgb(120, 120, 120),
                                        egui::RichText::new(format!("📍 {}", entry_path.to_string_lossy()))
                                            .size(12.0)
                                            .family(egui::FontFamily::Proportional)
                                    );
                                });

                                // 描述行
                                if let Some(description) = &entry_description {
                                    ui.add_space(2.0);
                                    ui.horizontal(|ui| {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(85, 85, 85),
                                            egui::RichText::new(format!("📄 {}", description))
                                                .size(13.0)
                                                .family(egui::FontFamily::Proportional)
                                        );
                                    });
                                }

                                // 标签和时间行
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    // 标签
                                    if !entry_tags.is_empty() {
                                        for tag in &entry_tags {
                                            let tag_btn = egui::Button::new(
                                                egui::RichText::new(format!("🏷️ {}", tag))
                                                    .size(11.0)
                                                    .family(egui::FontFamily::Proportional)
                                            )
                                            .fill(egui::Color32::from_rgb(230, 240, 255))
                                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 149, 237)))
                                            .rounding(egui::Rounding::same(12.0));
                                            
                                            if ui.add_sized([tag.len() as f32 * 8.0 + 30.0, 20.0], tag_btn).clicked() {
                                                self.search_query = tag.clone();
                                            }
                                            ui.add_space(4.0);
                                        }
                                    }

                                    // 时间在右侧
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(140, 140, 140),
                                            egui::RichText::new(format!("📅 {}", entry_created_at.format("%Y-%m-%d %H:%M")))
                                                .size(11.0)
                                                .family(egui::FontFamily::Proportional)
                                        );
                                    });
                                });
                            });
                        });
                    
                    ui.add_space(4.0);
                }

                // 处理删除和编辑操作
                if let Some(index) = to_remove {
                    self.remove_entry(index);
                }

                if let Some(index) = to_edit {
                    let entry = &self.entries[index];
                    self.selected_entry_index = Some(index);
                    self.current_path_input = entry.path.to_string_lossy().to_string();
                    self.current_name_input = entry.name.clone();
                    self.current_description_input = entry.description.clone().unwrap_or_default();
                    self.current_tag_input = entry.tags.join(", ");
                }

                if filtered_indices.is_empty() && !self.entries.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(40.0);
                        ui.label(egui::RichText::new("🔍").size(48.0));
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new("没有找到匹配的结果")
                                .size(16.0)
                                .color(egui::Color32::from_rgb(120, 120, 120))
                                .family(egui::FontFamily::Proportional)
                        );
                        ui.add_space(40.0);
                    });
                } else if self.entries.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(60.0);
                        ui.label(egui::RichText::new("📁").size(64.0));
                        ui.add_space(16.0);
                        ui.label(
                            egui::RichText::new("还没有添加任何文件或文件夹")
                                .size(18.0)
                                .color(egui::Color32::from_rgb(100, 100, 100))
                                .family(egui::FontFamily::Proportional)
                        );
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new("💡 提示: 可以拖拽文件/文件夹到此窗口快速添加")
                                .size(14.0)
                                .color(egui::Color32::from_rgb(150, 150, 150))
                                .family(egui::FontFamily::Proportional)
                        );
                        ui.add_space(60.0);
                    });
                }
            });
    }
}

impl eframe::App for FileManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 设置中文字体（始终启用）
        if !self.font_loaded {
            self.setup_chinese_fonts(ctx);
        }
        
        // 设置全局UI样式以改善中文显示
        let mut style = ctx.style().as_ref().clone();
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(8.0, 6.0);
        style.spacing.menu_margin = egui::style::Margin::same(8.0);
        style.spacing.indent = 18.0;
        
        style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(4.0);
        style.visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);
        style.visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);
        style.visuals.widgets.active.rounding = egui::Rounding::same(6.0);
        style.visuals.widgets.open.rounding = egui::Rounding::same(6.0);
        
        ctx.set_style(style);
        // 处理拖拽文件
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                for file in &i.raw.dropped_files {
                    if let Some(path) = &file.path {
                        self.add_entry(&path.to_string_lossy(), "", None, vec!["拖拽添加".to_string()]);
                    }
                }
            }
        });

        // 美化的顶部工具栏
        egui::TopBottomPanel::top("top_panel")
            .frame(egui::Frame::none()
                .fill(egui::Color32::from_rgb(45, 55, 75))
                .inner_margin(egui::Margin::symmetric(16.0, 12.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("🗂️ 文件快速访问器")
                            .size(20.0)
                            .color(egui::Color32::WHITE)
                            .strong()
                    );
                    
                    ui.add_space(16.0);
                    
                    // 美化的侧边栏切换按钮
                    let toggle_text = if self.sidebar_expanded { "◀ 收起面板" } else { "▶ 展开面板" };
                    let toggle_btn = egui::Button::new(
                        egui::RichText::new(toggle_text)
                            .color(egui::Color32::WHITE)
                    )
                    .fill(egui::Color32::from_rgb(70, 130, 180))
                    .rounding(egui::Rounding::same(6.0));
                    
                    if ui.add_sized([100.0, 28.0], toggle_btn).clicked() {
                        self.sidebar_expanded = !self.sidebar_expanded;
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(200, 200, 200),
                            egui::RichText::new("v1.0 - 高性能版本").size(12.0)
                        );
                    });
                });
            });

        // 美化的底部状态栏
        egui::TopBottomPanel::bottom("bottom_panel")
            .frame(egui::Frame::none()
                .fill(egui::Color32::from_rgb(248, 249, 250))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 220, 220)))
                .inner_margin(egui::Margin::symmetric(16.0, 8.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(
                        egui::Color32::from_rgb(100, 100, 100),
                        egui::RichText::new(format!(
                            "📄 配置文件: {}",
                            self.config_manager.get_config_path().display()
                        )).size(11.0)
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(100, 149, 237),
                            egui::RichText::new("💡 支持拖拽文件到窗口").size(11.0)
                        );
                    });
                });
            });

        // 右侧边栏（添加面板）
        if self.sidebar_expanded {
            egui::SidePanel::right("add_panel")
                .resizable(true)
                .default_width(300.0)
                .min_width(250.0)
                .max_width(500.0)
                .show(ctx, |ui| {
                    self.render_add_section(ui);
                });
        }

        // 主内容区域
        egui::CentralPanel::default().show(ctx, |ui| {
            // 美化的搜索区域
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(250, 251, 252))
                .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 149, 237)))
                .rounding(egui::Rounding::same(10.0))
                .inner_margin(egui::Margin::same(12.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("🔍 快速搜索")
                                .size(16.0)
                                .color(egui::Color32::from_rgb(70, 130, 180))
                                .strong()
                        );
                        ui.separator();
                        
                        let search_response = ui.add_sized(
                            [ui.available_width() - 90.0, 28.0],
                            egui::TextEdit::singleline(&mut self.search_query)
                                .hint_text("搜索名称、路径、描述、标签...")
                                .font(egui::TextStyle::Body)
                        );
                        
                        if search_response.changed() {
                            // 搜索时自动去除错误消息
                            self.error_message = None;
                        }
                        
                        let clear_btn = egui::Button::new("🗑️ 清空")
                            .fill(egui::Color32::from_rgb(255, 140, 0))
                            .rounding(egui::Rounding::same(6.0));
                        if ui.add_sized([70.0, 28.0], clear_btn).clicked() {
                            self.search_query.clear();
                        }
                    });
                });

            ui.add_space(8.0);

            // 文件列表
            self.render_file_list(ui);
        });
    }
}
