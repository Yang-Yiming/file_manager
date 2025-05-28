use crate::config::ConfigManager;
use crate::file_entry::FileEntry;
use eframe::egui;
use std::path::{Path, PathBuf};

pub struct FileManagerApp {
    entries: Vec<FileEntry>,
    current_path_input: String,
    current_tag_input: String,
    search_query: String,
    config_manager: ConfigManager,
    selected_entry_index: Option<usize>,
    error_message: Option<String>,
}

impl Default for FileManagerApp {
    fn default() -> Self {
        Self::new()
    }
}

impl FileManagerApp {
    pub fn new() -> Self {
        let config_manager = ConfigManager::new();
        let entries = config_manager.load_entries();

        Self {
            entries,
            config_manager,
            current_path_input: String::new(),
            current_tag_input: String::new(),
            search_query: String::new(),
            selected_entry_index: None,
            error_message: None,
        }
    }

    fn save_config(&self) {
        if let Err(e) = self.config_manager.save_entries(&self.entries) {
            eprintln!("保存配置失败: {}", e);
        }
    }

    fn add_entry(&mut self, path: &str, tags: Vec<String>) {
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

        let entry = FileEntry::new(path_buf, tags);
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

    fn update_entry(&mut self, index: usize, path: &str, tags: Vec<String>) {
        let path_buf = PathBuf::from(path.trim());

        if !path_buf.exists() {
            self.error_message = Some(format!("路径不存在: {}", path));
            return;
        }

        if index < self.entries.len() {
            self.entries[index] = FileEntry::new(path_buf, tags);
            self.save_config();
            self.selected_entry_index = None;
            self.current_path_input.clear();
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
        ui.collapsing("➕ 添加新文件/文件夹", |ui| {
            ui.horizontal(|ui| {
                ui.label("路径:");
                ui.text_edit_singleline(&mut self.current_path_input);
                if ui.button("📁 浏览").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.current_path_input = path.to_string_lossy().to_string();
                    }
                }
                if ui.button("📄 选择文件").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.current_path_input = path.to_string_lossy().to_string();
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("标签 (逗号分隔):");
                ui.text_edit_singleline(&mut self.current_tag_input);
            });

            ui.horizontal(|ui| {
                let button_text = if self.selected_entry_index.is_some() {
                    "💾 更新"
                } else {
                    "➕ 添加"
                };

                if ui.button(button_text).clicked() {
                    let tags: Vec<String> = self
                        .current_tag_input
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();

                    let path_input = self.current_path_input.clone();
                    
                    if let Some(index) = self.selected_entry_index {
                        self.update_entry(index, &path_input, tags);
                    } else {
                        self.add_entry(&path_input, tags);

                        if self.error_message.is_none() {
                            self.current_path_input.clear();
                            self.current_tag_input.clear();
                        }
                    }
                }

                if self.selected_entry_index.is_some() && ui.button("❌ 取消编辑").clicked() {
                    self.selected_entry_index = None;
                    self.current_path_input.clear();
                    self.current_tag_input.clear();
                }
            });
        });
    }

    fn render_file_list(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("📋 已保存的路径 ({})", self.entries.len()));

        egui::ScrollArea::vertical()
            .max_height(400.0)
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

                    ui.horizontal(|ui| {
                        // 图标
                        let icon = if entry.is_directory { "📁" } else { "📄" };
                        ui.label(icon);

                        // 路径（可点击）
                        if ui.link(entry.path.to_string_lossy()).clicked() {
                            self.open_path(&entry.path);
                        }

                        ui.separator();

                        // 标签
                        for tag in &entry.tags {
                            ui.small(format!("🏷️ {}", tag));
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🗑️").clicked() {
                                to_remove = Some(index);
                            }

                            if ui.button("📝").clicked() {
                                to_edit = Some(index);
                            }
                        });
                    });
                    ui.separator();
                }

                // 处理删除和编辑操作
                if let Some(index) = to_remove {
                    self.remove_entry(index);
                }

                if let Some(index) = to_edit {
                    let entry = &self.entries[index];
                    self.selected_entry_index = Some(index);
                    self.current_path_input = entry.path.to_string_lossy().to_string();
                    self.current_tag_input = entry.tags.join(", ");
                }

                if filtered_indices.is_empty() && !self.entries.is_empty() {
                    ui.label("没有找到匹配的结果");
                } else if self.entries.is_empty() {
                    ui.label("还没有添加任何文件或文件夹");
                    ui.label("💡 提示: 可以拖拽文件/文件夹到此窗口快速添加");
                }
            });
    }
}

impl eframe::App for FileManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 处理拖拽文件
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                for file in &i.raw.dropped_files {
                    if let Some(path) = &file.path {
                        self.add_entry(&path.to_string_lossy(), vec!["拖拽添加".to_string()]);
                    }
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🗂️ 文件快速访问器");

            ui.separator();

            // 错误消息显示
            if let Some(ref error) = self.error_message {
                ui.colored_label(egui::Color32::RED, error);
            }

            // 添加新条目区域
            self.render_add_section(ui);

            ui.separator();

            // 搜索区域
            ui.horizontal(|ui| {
                ui.label("🔍 搜索:");
                ui.text_edit_singleline(&mut self.search_query);
                if ui.button("清空").clicked() {
                    self.search_query.clear();
                }
            });

            ui.separator();

            // 文件列表
            self.render_file_list(ui);

            ui.separator();

            // 底部信息
            ui.horizontal(|ui| {
                ui.small(format!(
                    "配置文件: {}",
                    self.config_manager.get_config_path().display()
                ));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.small("💡 支持拖拽文件到窗口");
                });
            });
        });
    }
}
