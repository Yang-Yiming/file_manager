use crate::config::{AppConfig, ConfigManager, DataManager, UserData};
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
    data_manager: DataManager,
    config: AppConfig,
    user_data: UserData,
    font_loaded: bool,
    show_settings: bool,
    all_tags: HashSet<String>,
    theme_mode: ThemeMode,
    
    // 性能优化相关
    filtered_indices: Vec<usize>,
    last_search_query: String,
    last_filter_time: Instant,
    
    // 添加功能
    add_path_input: String,
    add_name_input: String,
    add_tags_input: String,
    add_description_input: String,
    show_add_dialog: bool,
    
    // 标签管理
    show_tag_editor: bool,
    editing_entry_index: Option<usize>,
    tag_filter: String,
    
    // 路径管理
    custom_config_path: String,
    custom_data_path: String,
    
    // 导入导出功能
    show_import_export: bool,
    import_merge_mode: bool,
    export_status: String,
    import_status: String,
    
    // 增强的标签管理
    show_tag_manager: bool,
    tag_cloud_filter: String,
    selected_tags: HashSet<String>,
    batch_tag_input: String,
    show_tag_suggestions: bool,
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
        
        // 创建数据管理器
        let data_manager = if let Some(data_path) = &config.data_file_path {
            if !data_path.is_empty() {
                let data_path_buf = PathBuf::from(data_path);
                if data_path_buf.exists() || data_path_buf.parent().map_or(false, |p| p.exists()) {
                    DataManager::new_with_path(data_path_buf)
                } else {
                    DataManager::new()
                }
            } else {
                DataManager::new()
            }
        } else {
            DataManager::new()
        };
        
        let user_data = data_manager.load_data().unwrap_or_default();
        let entries = user_data.entries.clone();

        let mut all_tags = HashSet::new();
        for entry in &entries {
            for tag in &entry.tags {
                all_tags.insert(tag.clone());
            }
        }

        let filtered_indices: Vec<usize> = (0..entries.len()).collect();
        
        // 从配置中恢复主题模式
        let theme_mode = match config.theme_mode.as_str() {
            "Dark" => ThemeMode::Dark,
            "System" => ThemeMode::System,
            _ => ThemeMode::Light,
        };

        Self {
            entries,
            search_query: String::new(),
            config_manager,
            data_manager,
            config: config.clone(),
            user_data,
            font_loaded: false,
            show_settings: false,
            all_tags,
            theme_mode,
            filtered_indices,
            last_search_query: String::new(),
            last_filter_time: Instant::now(),
            add_path_input: String::new(),
            add_name_input: String::new(),
            add_tags_input: String::new(),
            add_description_input: String::new(),
            show_add_dialog: false,
            show_tag_editor: false,
            editing_entry_index: None,
            tag_filter: String::new(),
            custom_config_path: String::new(),
            custom_data_path: config.data_file_path.clone().unwrap_or_default(),
            
            // 导入导出功能
            show_import_export: false,
            import_merge_mode: true,
            export_status: String::new(),
            import_status: String::new(),
            
            // 增强的标签管理
            show_tag_manager: false,
            tag_cloud_filter: String::new(),
            selected_tags: HashSet::new(),
            batch_tag_input: String::new(),
            show_tag_suggestions: false,
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
                    fonts.font_data.insert(
                        "chinese".to_owned(),
                        egui::FontData::from_owned(font_data),
                    );
                    fonts.families.get_mut(&egui::FontFamily::Proportional)
                        .unwrap()
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
                "/System/Library/Fonts/STHeiti Light.ttc",
            ];
            
            for font_path in &font_paths {
                if let Ok(font_data) = std::fs::read(font_path) {
                    fonts.font_data.insert(
                        "chinese".to_owned(),
                        egui::FontData::from_owned(font_data),
                    );
                    fonts.families.get_mut(&egui::FontFamily::Proportional)
                        .unwrap()
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
                "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            ];
            
            for font_path in &font_paths {
                if let Ok(font_data) = std::fs::read(font_path) {
                    fonts.font_data.insert(
                        "chinese".to_owned(),
                        egui::FontData::from_owned(font_data),
                    );
                    fonts.families.get_mut(&egui::FontFamily::Proportional)
                        .unwrap()
                        .insert(0, "chinese".to_owned());
                    font_loaded = true;
                    break;
                }
            }
        }

        // 如果没有找到系统字体，使用 egui 的默认字体
        if !font_loaded {
            #[cfg(debug_assertions)]
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
                .filter(|(_, entry)| {
                    entry.matches_query(&self.search_query) && self.matches_tag_filter(entry)
                })
                .map(|(i, _)| i)
                .collect();
            
            self.last_search_query = self.search_query.clone();
            self.last_filter_time = Instant::now();
        }
    }

    fn force_update_filter(&mut self) {
        // 强制重新过滤，不管搜索查询是否改变
        self.filtered_indices = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                entry.matches_query(&self.search_query) && self.matches_tag_filter(entry)
            })
            .map(|(i, _)| i)
            .collect();
        
        self.last_search_query = self.search_query.clone();
        self.last_filter_time = Instant::now();
    }

    fn matches_tag_filter(&self, entry: &FileEntry) -> bool {
        if self.tag_filter.is_empty() {
            return true;
        }
        
        let (hash_tags, _path_tags) = entry.get_tag_categories();
        
        // 支持多标签过滤，用空格分隔
        let filter_tags: Vec<&str> = self.tag_filter.split_whitespace().collect();
        
        for filter_tag in filter_tags {
            let found = if filter_tag.starts_with('#') {
                // 检查hash标签
                hash_tags.iter().any(|tag| tag.contains(filter_tag))
            } else {
                // 如果没有#前缀，也在hash标签中搜索
                hash_tags.iter().any(|tag| tag.to_lowercase().contains(&filter_tag.to_lowercase()))
            };
            
            // 如果任何一个标签没有匹配，则不显示该条目
            if !found {
                return false;
            }
        }
        
        true
    }

    fn save_config(&mut self) -> Result<(), String> {
        // 保存主题设置到配置
        self.config.theme_mode = match self.theme_mode {
            ThemeMode::Light => "Light".to_string(),
            ThemeMode::Dark => "Dark".to_string(),
            ThemeMode::System => "System".to_string(),
        };
        self.config_manager.save_config(&self.config)
    }

    fn save_user_data(&mut self) -> Result<(), String> {
        self.user_data.entries = self.entries.clone();
        self.data_manager.save_data(&self.user_data)
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
        let tags = FileEntry::parse_tags(&self.add_tags_input);
        let description = if self.add_description_input.is_empty() {
            None
        } else {
            Some(self.add_description_input.clone())
        };
        
        let entry = FileEntry::new(path, name, description, tags.clone(), is_directory);
        
        // 更新标签集合
        for tag in &tags {
            self.all_tags.insert(tag.clone());
        }
        
        self.entries.push(entry);
        let _ = self.save_user_data();
        
        // 清空输入
        self.add_path_input.clear();
        self.add_name_input.clear();
        self.add_tags_input.clear();
        self.add_description_input.clear();
        self.show_add_dialog = false;
        
        // 强制重新过滤并更新索引
        self.force_update_filter();
    }

    fn remove_entry(&mut self, index: usize) {
        if index < self.entries.len() {
            let _removed_entry = self.entries.remove(index);
            
            // 更新标签集合，移除不再使用的标签
            self.rebuild_tag_set();
            
            let _ = self.save_user_data();
            self.force_update_filter();
        }
    }

    fn rebuild_tag_set(&mut self) {
        self.all_tags.clear();
        for entry in &self.entries {
            for tag in &entry.tags {
                self.all_tags.insert(tag.clone());
            }
        }
    }

    fn open_path(&self, path: &PathBuf) {
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

    fn edit_entry_tags(&mut self, index: usize) {
        if index < self.entries.len() {
            self.editing_entry_index = Some(index);
            let entry = &self.entries[index];
            self.add_tags_input = entry.tags.join(" ");
            self.add_description_input = entry.description.clone().unwrap_or_default();
            self.show_tag_editor = true;
        }
    }

    fn save_entry_edit(&mut self) {
        if let Some(index) = self.editing_entry_index {
            if index < self.entries.len() {
                let new_tags = FileEntry::parse_tags(&self.add_tags_input);
                let new_description = if self.add_description_input.is_empty() {
                    None
                } else {
                    Some(self.add_description_input.clone())
                };
                
                // 更新条目
                self.entries[index].tags = new_tags.clone();
                self.entries[index].description = new_description;
                
                // 重建标签集合
                self.rebuild_tag_set();
                for tag in &new_tags {
                    self.all_tags.insert(tag.clone());
                }
                
                let _ = self.save_user_data();
                self.force_update_filter();
            }
        }
        
        // 清空编辑状态
        self.show_tag_editor = false;
        self.editing_entry_index = None;
        self.add_tags_input.clear();
        self.add_description_input.clear();
    }

    fn export_data(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON文件", &["json"])
            .set_file_name("file_manager_export.json")
            .save_file()
        {
            let export_data = UserData {
                entries: self.entries.clone(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            };
            
            match serde_json::to_string_pretty(&export_data) {
                Ok(json) => {
                    match std::fs::write(&path, json) {
                        Ok(_) => {
                            self.export_status = format!("✅ 导出成功: {}", path.display());
                        }
                        Err(e) => {
                            self.export_status = format!("❌ 导出失败: {}", e);
                        }
                    }
                }
                Err(e) => {
                    self.export_status = format!("❌ 序列化失败: {}", e);
                }
            }
        }
    }

    fn import_data(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON文件", &["json"])
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_json::from_str::<UserData>(&content) {
                        Ok(import_data) => {
                            let import_count = import_data.entries.len();
                            
                            if self.import_merge_mode {
                                // 合并模式：添加到现有数据
                                for entry in import_data.entries {
                                    // 检查是否已存在相同路径的条目
                                    if !self.entries.iter().any(|e| e.path == entry.path) {
                                        // 更新标签集合
                                        for tag in &entry.tags {
                                            self.all_tags.insert(tag.clone());
                                        }
                                        self.entries.push(entry);
                                    }
                                }
                                self.import_status = format!("✅ 合并导入成功: {} 个条目", import_count);
                            } else {
                                // 替换模式：替换所有数据
                                self.entries = import_data.entries;
                                self.rebuild_tag_set();
                                self.import_status = format!("✅ 替换导入成功: {} 个条目", import_count);
                            }
                            
                            let _ = self.save_user_data();
                            self.force_update_filter();
                        }
                        Err(e) => {
                            // 尝试兼容旧格式
                            if let Ok(entries) = serde_json::from_str::<Vec<FileEntry>>(&content) {
                                let import_count = entries.len();
                                
                                if self.import_merge_mode {
                                    for entry in entries {
                                        if !self.entries.iter().any(|e| e.path == entry.path) {
                                            for tag in &entry.tags {
                                                self.all_tags.insert(tag.clone());
                                            }
                                            self.entries.push(entry);
                                        }
                                    }
                                    self.import_status = format!("✅ 合并导入成功(旧格式): {} 个条目", import_count);
                                } else {
                                    self.entries = entries;
                                    self.rebuild_tag_set();
                                    self.import_status = format!("✅ 替换导入成功(旧格式): {} 个条目", import_count);
                                }
                                
                                let _ = self.save_user_data();
                                self.force_update_filter();
                            } else {
                                self.import_status = format!("❌ 文件格式错误: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    self.import_status = format!("❌ 读取文件失败: {}", e);
                }
            }
        }
    }

    fn batch_add_tags(&mut self, tag_text: &str) {
        let new_tags = FileEntry::parse_tags(tag_text);
        if new_tags.is_empty() {
            return;
        }

        let mut modified_count = 0;
        for i in &self.filtered_indices {
            if let Some(entry) = self.entries.get_mut(*i) {
                let mut entry_modified = false;
                for tag in &new_tags {
                    if !entry.tags.contains(tag) {
                        entry.tags.push(tag.clone());
                        self.all_tags.insert(tag.clone());
                        entry_modified = true;
                    }
                }
                if entry_modified {
                    entry.tags.sort();
                    entry.tags.dedup();
                    modified_count += 1;
                }
            }
        }

        if modified_count > 0 {
            let _ = self.save_user_data();
            self.force_update_filter();
        }
    }

    fn batch_remove_tags(&mut self, tag_text: &str) {
        let remove_tags = FileEntry::parse_tags(tag_text);
        if remove_tags.is_empty() {
            return;
        }

        let mut modified_count = 0;
        for i in &self.filtered_indices {
            if let Some(entry) = self.entries.get_mut(*i) {
                let original_len = entry.tags.len();
                entry.tags.retain(|tag| !remove_tags.contains(tag));
                if entry.tags.len() != original_len {
                    modified_count += 1;
                }
            }
        }

        if modified_count > 0 {
            self.rebuild_tag_set();
            let _ = self.save_user_data();
            self.force_update_filter();
        }
    }

    fn get_tag_usage_stats(&self) -> Vec<(String, usize)> {
        let mut tag_counts = std::collections::HashMap::new();
        
        for entry in &self.entries {
            for tag in &entry.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        
        let mut stats: Vec<(String, usize)> = tag_counts.into_iter().collect();
        stats.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        stats
    }

    fn render_tag_suggestions(&mut self, ui: &mut egui::Ui, input_text: &str) {
        if input_text.is_empty() {
            return;
        }

        let input_lower = input_text.to_lowercase();
        let matching_tags: Vec<String> = self.all_tags
            .iter()
            .filter(|tag| tag.to_lowercase().contains(&input_lower) && !input_text.contains(*tag))
            .cloned()
            .collect();

        if !matching_tags.is_empty() {
            ui.label("💡 标签建议:");
            ui.horizontal_wrapped(|ui| {
                for tag in matching_tags.iter().take(8) {
                    if ui.small_button(tag).clicked() {
                        if !self.add_tags_input.contains(tag) {
                            if self.add_tags_input.is_empty() {
                                self.add_tags_input = tag.clone();
                            } else {
                                self.add_tags_input = format!("{} {}", self.add_tags_input, tag);
                            }
                        }
                    }
                }
            });
        }
    }

    fn render_import_export(&mut self, ui: &mut egui::Ui) {
        ui.heading("数据导入导出");
        ui.separator();

        ui.collapsing("导出数据", |ui| {
            ui.label("将当前所有文件条目导出为JSON文件");
            ui.add_space(8.0);
            
            if ui.button("📤 导出数据").clicked() {
                self.export_data();
            }
            
            if !self.export_status.is_empty() {
                ui.add_space(4.0);
                ui.label(&self.export_status);
                if ui.small_button("清除状态").clicked() {
                    self.export_status.clear();
                }
            }
        });

        ui.add_space(16.0);
        
        ui.collapsing("导入数据", |ui| {
            ui.label("从JSON文件导入文件条目");
            ui.add_space(8.0);
            
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.import_merge_mode, true, "合并模式");
                ui.radio_value(&mut self.import_merge_mode, false, "替换模式");
            });
            
            ui.label(if self.import_merge_mode {
                "💡 合并模式：新数据会添加到现有数据中，相同路径的条目会被忽略"
            } else {
                "⚠️ 替换模式：新数据会完全替换现有数据"
            });
            
            ui.add_space(8.0);
            
            if ui.button("📥 导入数据").clicked() {
                self.import_data();
            }
            
            if !self.import_status.is_empty() {
                ui.add_space(4.0);
                ui.label(&self.import_status);
                if ui.small_button("清除状态").clicked() {
                    self.import_status.clear();
                }
            }
        });

        ui.add_space(16.0);
        
        ui.collapsing("快速操作", |ui| {
            ui.label("批量操作当前搜索结果中的条目");
            ui.add_space(8.0);
            
            ui.horizontal(|ui| {
                ui.label("标签：");
                if ui.text_edit_singleline(&mut self.batch_tag_input).changed() {
                    // 触发标签建议显示
                }
            });
            
            if !self.batch_tag_input.is_empty() {
                self.render_tag_suggestions(ui, &self.batch_tag_input.clone());
            }
            
            ui.horizontal(|ui| {
                if ui.button("➕ 批量添加标签").clicked() && !self.batch_tag_input.is_empty() {
                    self.batch_add_tags(&self.batch_tag_input.clone());
                    self.batch_tag_input.clear();
                }
                
                if ui.button("➖ 批量移除标签").clicked() && !self.batch_tag_input.is_empty() {
                    self.batch_remove_tags(&self.batch_tag_input.clone());
                    self.batch_tag_input.clear();
                }
            });
            
            ui.add_space(4.0);
            ui.label(format!("当前显示 {} 个条目", self.filtered_indices.len()));
        });
    }

    fn render_tag_manager(&mut self, ui: &mut egui::Ui) {
        ui.heading("标签管理");
        ui.separator();

        ui.collapsing("标签统计", |ui| {
            let stats = self.get_tag_usage_stats();
            ui.label(format!("总计 {} 个标签", stats.len()));
            ui.add_space(8.0);
            
            ui.horizontal(|ui| {
                ui.label("筛选：");
                ui.text_edit_singleline(&mut self.tag_cloud_filter);
            });
            
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for (tag, count) in stats {
                        if self.tag_cloud_filter.is_empty() || 
                           tag.to_lowercase().contains(&self.tag_cloud_filter.to_lowercase()) {
                            ui.horizontal(|ui| {
                                let is_selected = self.selected_tags.contains(&tag);
                                let mut selected = is_selected;
                                if ui.checkbox(&mut selected, "").changed() {
                                    if selected {
                                        self.selected_tags.insert(tag.clone());
                                    } else {
                                        self.selected_tags.remove(&tag);
                                    }
                                }
                                
                                if ui.small_button(&tag).clicked() {
                                    self.tag_filter = tag.clone();
                                    self.force_update_filter();
                                }
                                
                                ui.label(format!("({})", count));
                            });
                        }
                    }
                });
        });

        ui.add_space(16.0);
        
        ui.collapsing("快速标签", |ui| {
            ui.label("点击快速添加到搜索过滤");
            ui.add_space(8.0);
            
            let common_tags = ["#工作", "#项目", "#文档", "#图片", "#视频", "#音频", "#重要", "#临时"];
            ui.horizontal_wrapped(|ui| {
                for &tag in &common_tags {
                    if ui.small_button(tag).clicked() {
                        if !self.tag_filter.contains(tag) {
                            if self.tag_filter.is_empty() {
                                self.tag_filter = tag.to_string();
                            } else {
                                self.tag_filter = format!("{} {}", self.tag_filter, tag);
                            }
                            self.force_update_filter();
                        }
                    }
                }
            });
        });

        ui.add_space(16.0);
        
        if !self.selected_tags.is_empty() {
            ui.collapsing("批量标签操作", |ui| {
                ui.label(format!("已选择 {} 个标签", self.selected_tags.len()));
                
                ui.horizontal(|ui| {
                    if ui.button("🔍 按选中标签过滤").clicked() {
                        self.tag_filter = self.selected_tags.iter().cloned().collect::<Vec<_>>().join(" ");
                        self.force_update_filter();
                    }
                    
                    if ui.button("🗑️ 清除选择").clicked() {
                        self.selected_tags.clear();
                    }
                });
            });
        }
    }

    fn render_add_dialog(&mut self, ui: &mut egui::Ui) {
        ui.heading("添加文件");
        ui.separator();

        ui.label("路径:");
        ui.text_edit_singleline(&mut self.add_path_input);
        
        ui.horizontal(|ui| {
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
        });

        ui.add_space(8.0);
        ui.label("名称 (可选):");
        ui.text_edit_singleline(&mut self.add_name_input);

        ui.add_space(8.0);
        ui.label("标签 (使用 # 前缀):");
        if ui.text_edit_singleline(&mut self.add_tags_input).changed() {
            self.show_tag_suggestions = !self.add_tags_input.is_empty();
        }
        ui.small("示例: #重要 #工作 #项目 学习");
        
        if self.show_tag_suggestions {
            self.render_tag_suggestions(ui, &self.add_tags_input.clone());
        }

        ui.add_space(8.0);
        ui.label("描述 (可选):");
        ui.text_edit_multiline(&mut self.add_description_input);

        ui.add_space(16.0);
        ui.horizontal(|ui| {
            if ui.button("添加").clicked() {
                self.add_entry();
            }
            if ui.button("取消").clicked() {
                self.show_add_dialog = false;
                self.add_path_input.clear();
                self.add_name_input.clear();
                self.add_tags_input.clear();
                self.add_description_input.clear();
            }
        });
    }

    fn render_tag_editor(&mut self, ui: &mut egui::Ui) {
        ui.heading("编辑标签");
        ui.separator();

        if let Some(index) = self.editing_entry_index {
            if index < self.entries.len() {
                let entry_name = &self.entries[index].name;
                ui.label(format!("编辑: {}", entry_name));
                ui.separator();
            }
        }

        ui.label("标签 (使用 # 前缀):");
        ui.text_edit_singleline(&mut self.add_tags_input);
        ui.small("示例: #重要 #工作 #项目 学习");

        ui.add_space(8.0);
        ui.label("描述:");
        ui.text_edit_multiline(&mut self.add_description_input);

        ui.add_space(16.0);
        ui.horizontal(|ui| {
            if ui.button("保存").clicked() {
                self.save_entry_edit();
            }
            if ui.button("取消").clicked() {
                self.show_tag_editor = false;
                self.editing_entry_index = None;
                self.add_tags_input.clear();
                self.add_description_input.clear();
            }
        });
    }

    fn render_list(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("文件列表");
            ui.separator();
            
            // 标签过滤
            ui.label("🏷️");
            ui.text_edit_singleline(&mut self.tag_filter);
            if ui.small_button("清除").clicked() {
                self.tag_filter.clear();
                self.last_search_query.clear();
            }
        });
        
        ui.separator();

        let mut to_remove = None;
        let mut to_edit = None;
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
                    let (hash_tags, _path_tags) = entry.get_tag_categories();
                    let entry_is_directory = entry.is_directory;
                    let entry_description = entry.description.clone();
                    
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            // 图标
                            let icon = if entry_is_directory { "📁" } else { "📄" };
                            ui.label(icon);
                            
                            // 文件名（可点击打开）
                            if ui.link(&entry_name).clicked() {
                                self.open_path(&entry_path);
                            }
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("删除").clicked() {
                                    to_remove = Some(index);
                                }
                                if ui.small_button("编辑").clicked() {
                                    to_edit = Some(index);
                                }
                            });
                        });
                        
                        // 显示标签
                        if !hash_tags.is_empty() {
                            ui.horizontal_wrapped(|ui| {
                                ui.small("🏷️");
                                // 显示 # 标签，点击可以添加到过滤器
                                for tag in &hash_tags {
                                    if ui.small_button(egui::RichText::new(tag).color(egui::Color32::BLUE)).clicked() {
                                        if !self.tag_filter.contains(tag) {
                                            if self.tag_filter.is_empty() {
                                                self.tag_filter = tag.clone();
                                            } else {
                                                self.tag_filter = format!("{} {}", self.tag_filter, tag);
                                            }
                                            self.force_update_filter();
                                        }
                                    }
                                }
                            });
                        }
                        
                        // 显示描述
                        if let Some(desc) = &entry_description {
                            ui.small(egui::RichText::new(desc).italics());
                        }
                    });
                    
                    ui.add_space(4.0);
                }
                
                if filtered_indices.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        if self.entries.is_empty() {
                            ui.label("还没有添加任何文件");
                            ui.small("点击'添加'按钮开始");
                        } else {
                            ui.label("没有找到匹配的结果");
                            ui.small("尝试调整搜索条件或标签过滤器");
                        }
                    });
                }
            });

        if let Some(index) = to_remove {
            self.remove_entry(index);
        }
        
        if let Some(index) = to_edit {
            self.edit_entry_tags(index);
        }
    }

    fn render_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("设置");
        ui.separator();

        ui.label("主题:");
        ui.horizontal(|ui| {
            let old_theme = self.theme_mode;
            ui.selectable_value(&mut self.theme_mode, ThemeMode::Light, "浅色");
            ui.selectable_value(&mut self.theme_mode, ThemeMode::Dark, "深色");
            ui.selectable_value(&mut self.theme_mode, ThemeMode::System, "系统");
            
            // 如果主题改变，保存配置
            if self.theme_mode != old_theme {
                let _ = self.save_config();
            }
        });

        ui.add_space(16.0);
        ui.label(format!("文件数量: {}", self.entries.len()));
        ui.label(format!("标签数量: {}", self.all_tags.len()));
        
        ui.add_space(16.0);
        ui.collapsing("数据备份", |ui| {
            ui.label("快速备份当前数据");
            ui.add_space(8.0);
            
            ui.horizontal(|ui| {
                if ui.button("🔄 创建备份").clicked() {
                    let backup_name = format!("backup_{}.json", 
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs());
                    
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("JSON文件", &["json"])
                        .set_file_name(&backup_name)
                        .save_file()
                    {
                        let backup_data = UserData {
                            entries: self.entries.clone(),
                            version: env!("CARGO_PKG_VERSION").to_string(),
                        };
                        
                        match serde_json::to_string_pretty(&backup_data) {
                            Ok(json) => {
                                match std::fs::write(&path, json) {
                                    Ok(_) => {
                                        self.export_status = format!("✅ 备份成功: {}", path.display());
                                    }
                                    Err(e) => {
                                        self.export_status = format!("❌ 备份失败: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                self.export_status = format!("❌ 备份序列化失败: {}", e);
                            }
                        }
                    }
                }
                
                if ui.button("📥 从备份恢复").clicked() {
                    self.import_data();
                }
            });
            
            if !self.export_status.is_empty() {
                ui.add_space(4.0);
                ui.label(&self.export_status);
                if ui.small_button("清除状态").clicked() {
                    self.export_status.clear();
                }
            }
            
            ui.add_space(4.0);
            ui.label("💡 提示: 建议定期备份数据以防丢失");
        });

        ui.add_space(16.0);
        ui.collapsing("应用配置文件", |ui| {
            ui.label("配置文件格式: JSON");
            ui.label(format!("当前位置: {}", self.config_manager.get_config_path().display()));
            
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label("自定义配置路径:");
                ui.text_edit_singleline(&mut self.custom_config_path);
            });
            
            ui.horizontal(|ui| {
                if ui.button("📁 选择位置").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("JSON文件", &["json"])
                        .set_file_name("file_manager_config.json")
                        .save_file()
                    {
                        self.custom_config_path = path.to_string_lossy().to_string();
                    }
                }
                
                if ui.button("✅ 应用配置路径").clicked() && !self.custom_config_path.is_empty() {
                    let new_path = PathBuf::from(&self.custom_config_path);
                    self.config_manager = ConfigManager::new_with_path(new_path);
                    if let Err(e) = self.save_config() {
                        ui.label(format!("❌ 保存配置失败: {}", e));
                    } else {
                        ui.label("✅ 配置路径已更新");
                    }
                }
                
                if ui.button("🔄 重置配置路径").clicked() {
                    self.config_manager = ConfigManager::new();
                    self.custom_config_path.clear();
                    let _ = self.save_config();
                }
            });
        });

        ui.add_space(8.0);
        ui.collapsing("用户数据文件", |ui| {
            ui.label("数据文件格式: JSON");
            ui.label(format!("当前位置: {}", self.data_manager.get_data_path().display()));
            
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label("自定义数据路径:");
                ui.text_edit_singleline(&mut self.custom_data_path);
            });
            
            ui.horizontal(|ui| {
                if ui.button("📁 选择位置").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("JSON文件", &["json"])
                        .set_file_name("file_manager_data.json")
                        .save_file()
                    {
                        self.custom_data_path = path.to_string_lossy().to_string();
                    }
                }
                
                if ui.button("✅ 应用数据路径").clicked() && !self.custom_data_path.is_empty() {
                    let new_path = PathBuf::from(&self.custom_data_path);
                    
                    // 先保存当前数据到新位置
                    let old_manager = std::mem::replace(&mut self.data_manager, DataManager::new_with_path(new_path));
                    self.config.data_file_path = Some(self.custom_data_path.clone());
                    
                    if let Err(e) = self.save_user_data() {
                        // 如果保存失败，恢复原来的数据管理器
                        self.data_manager = old_manager;
                        self.config.data_file_path = None;
                        ui.label(format!("❌ 保存数据失败: {}", e));
                    } else {
                        // 保存配置中的数据路径
                        let _ = self.save_config();
                        ui.label("✅ 数据路径已更新");
                    }
                }
                
                if ui.button("🔄 重置数据路径").clicked() {
                    self.data_manager = DataManager::new();
                    self.custom_data_path.clear();
                    self.config.data_file_path = None;
                    let _ = self.save_config();
                    let _ = self.save_user_data();
                }
            });
            
            ui.add_space(8.0);
            ui.label("💡 提示:");
            ui.label("• 用户数据(文件列表)与应用配置分开保存");
            ui.label("• 数据以JSON格式保存，便于备份和迁移");
            ui.label("• 重新打开应用后数据会自动恢复");
        });

        ui.add_space(8.0);
        ui.collapsing("标签概览", |ui| {
            let tag_stats = self.get_tag_usage_stats();
            
            if !tag_stats.is_empty() {
                ui.label(egui::RichText::new("标签使用统计:").strong());
                ui.add_space(4.0);
                
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for (tag, count) in tag_stats.iter().take(20) {
                            ui.horizontal(|ui| {
                                if ui.small_button(tag).clicked() {
                                    self.tag_filter = tag.clone();
                                    self.force_update_filter();
                                }
                                ui.label(format!("({} 个文件)", count));
                            });
                        }
                    });
                    
                if tag_stats.len() > 20 {
                    ui.label(format!("...还有 {} 个标签", tag_stats.len() - 20));
                }
            } else {
                ui.label("还没有标签");
                ui.small("在添加文件时可以为它们设置标签");
            }
        });

        ui.add_space(16.0);
        ui.add_space(16.0);
        if ui.button("清空所有用户数据").clicked() {
            self.entries.clear();
            self.all_tags.clear();
            let _ = self.save_user_data();
            self.force_update_filter();
        }
    }
}

impl eframe::App for FileManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 只在第一次设置字体
        self.setup_fonts_once(ctx);
        
        // 应用主题
        self.apply_theme(ctx);

        // 处理拖拽文件
        ctx.input(|i| {
            for file in &i.raw.dropped_files {
                if let Some(path) = &file.path {
                    // 直接添加拖拽的文件，而不是只设置到输入框
                    let path_buf = path.clone();
                    let name = path_buf.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("未命名")
                        .to_string();
                    let is_directory = path_buf.is_dir();
                    
                    let entry = FileEntry::new(path_buf, name, None, Vec::new(), is_directory);
                    self.entries.push(entry);
                    let _ = self.save_user_data();
                    
                    // 强制重新过滤并更新索引
                    self.force_update_filter();
                }
            }
        });

        // 顶部面板
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🗂️ 文件管理器");
                ui.separator();
                
                ui.label("🔍");
                ui.text_edit_singleline(&mut self.search_query);
                
                ui.separator();
                
                if ui.button("添加").clicked() {
                    self.show_add_dialog = !self.show_add_dialog;
                    self.show_tag_editor = false;
                    self.show_settings = false;
                    self.show_import_export = false;
                    self.show_tag_manager = false;
                }
                
                if ui.button("导入导出").clicked() {
                    self.show_import_export = !self.show_import_export;
                    self.show_add_dialog = false;
                    self.show_tag_editor = false;
                    self.show_settings = false;
                    self.show_tag_manager = false;
                }
                
                if ui.button("标签管理").clicked() {
                    self.show_tag_manager = !self.show_tag_manager;
                    self.show_add_dialog = false;
                    self.show_tag_editor = false;
                    self.show_settings = false;
                    self.show_import_export = false;
                }
                
                if ui.button("设置").clicked() {
                    self.show_settings = !self.show_settings;
                    self.show_add_dialog = false;
                    self.show_tag_editor = false;
                    self.show_import_export = false;
                    self.show_tag_manager = false;
                }
            });
        });

        // 侧边面板
        if self.show_add_dialog || self.show_tag_editor || self.show_settings || self.show_import_export || self.show_tag_manager {
            egui::SidePanel::right("side").show(ctx, |ui| {
                if self.show_add_dialog {
                    self.render_add_dialog(ui);
                } else if self.show_tag_editor {
                    self.render_tag_editor(ui);
                } else if self.show_import_export {
                    self.render_import_export(ui);
                } else if self.show_tag_manager {
                    self.render_tag_manager(ui);
                } else if self.show_settings {
                    self.render_settings(ui);
                }
            });
        }

        // 主面板
        egui::CentralPanel::default().show(ctx, |ui| {
            // 标签过滤显示
            if !self.tag_filter.is_empty() {
                ui.horizontal(|ui| {
                    ui.label("🏷️ 标签过滤:");
                    ui.label(egui::RichText::new(&self.tag_filter).color(egui::Color32::BLUE));
                    if ui.small_button("❌").clicked() {
                        self.tag_filter.clear();
                        self.force_update_filter();
                    }
                });
                ui.separator();
            }
            
            // 快速标签过滤输入
            ui.horizontal(|ui| {
                ui.label("🏷️ 标签过滤:");
                if ui.text_edit_singleline(&mut self.tag_filter).changed() {
                    self.force_update_filter();
                }
                if !self.tag_filter.is_empty() && ui.small_button("清除").clicked() {
                    self.tag_filter.clear();
                    self.force_update_filter();
                }
            });
            
            // 显示常用标签作为快速过滤选项
            if !self.all_tags.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    ui.label("快速过滤:");
                    let sorted_tags: Vec<String> = self.all_tags.iter().take(10).cloned().collect();
                    for tag in sorted_tags {
                        if ui.small_button(&tag).clicked() {
                            if !self.tag_filter.contains(&tag) {
                                if self.tag_filter.is_empty() {
                                    self.tag_filter = tag.clone();
                                } else {
                                    self.tag_filter = format!("{} {}", self.tag_filter, tag);
                                }
                                self.force_update_filter();
                            }
                        }
                    }
                });
            }
            ui.add_space(8.0);
            
            self.update_filter();
            self.render_list(ui);
        });
    }
}
