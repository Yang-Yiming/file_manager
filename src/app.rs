use crate::config::{AppConfig, ConfigManager, DataManager, UserData};
use crate::file_entry::FileEntry;
use crate::fonts::setup_chinese_fonts;
use crate::theme::{ModernTheme, ThemeMode};
use eframe::egui;
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Instant;

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
    compact_mode: bool,
    expanded_entries: HashSet<usize>,

    // 筛选相关
    filtered_indices: Vec<usize>,
    last_search_query: String,
    last_filter_time: Instant,

    // 添加对话框相关
    add_path_input: String,
    add_name_input: String,
    add_nickname_input: String,
    add_tags_input: String,
    add_description_input: String,
    show_add_dialog: bool,
    add_entry_type: crate::file_entry::EntryType,

    // 标签编辑相关
    show_tag_editor: bool,
    editing_entry_index: Option<usize>,

    // 配置路径相关
    custom_config_path: String,
    custom_data_path: String,

    // 导入导出相关
    show_import_export: bool,
    import_merge_mode: bool,
    export_status: String,
    import_status: String,

    // 标签管理相关
    show_tag_manager: bool,
    tag_cloud_filter: String,
    selected_tags: HashSet<String>,
    batch_tag_input: String,
    show_tag_suggestions: bool,

    // 删除确认对话框相关
    show_delete_confirm: bool,
    delete_entry_index: Option<usize>,
    delete_entry_name: String,

    // 集合管理相关
    show_collection_manager: bool,
    editing_collection_index: Option<usize>,
    collection_child_selection: HashSet<usize>,
    
    // 多选相关
    selected_entries: HashSet<usize>,
    show_batch_collection_dialog: bool,
    batch_collection_name: String,
    
    // 焦点和选中状态
    focused_entry: Option<usize>,
    search_has_focus: bool,
    search_currently_focused: bool,
    multi_select_mode: bool,
}

impl Default for FileManagerApp {
    fn default() -> Self {
        Self::new()
    }
}

impl FileManagerApp {
    /// 迁移旧的索引系统到新的ID系统
    fn migrate_entries_to_id_system(entries: &mut Vec<FileEntry>) {
        // 第一步：确保所有条目都有ID
        for entry in entries.iter_mut() {
            if entry.id.is_empty() {
                entry.id = uuid::Uuid::new_v4().to_string();
            }
        }
        
        // 第二步：迁移集合的子项目引用从索引到ID
        let mut collections_to_migrate = Vec::new();
        for (i, entry) in entries.iter().enumerate() {
            if entry.entry_type == crate::file_entry::EntryType::Collection && entry.has_legacy_child_entries() {
                collections_to_migrate.push(i);
            }
        }
        
        for collection_idx in collections_to_migrate {
            let legacy_indices = entries[collection_idx].get_legacy_child_entries().clone();
            let mut new_child_ids = Vec::new();
            
            for &legacy_index in &legacy_indices {
                if legacy_index < entries.len() {
                    new_child_ids.push(entries[legacy_index].id.clone());
                }
            }
            
            entries[collection_idx].child_entries = new_child_ids;
            entries[collection_idx].clear_legacy_child_entries();
        }
    }

    fn toggle_panel(&mut self, panel: &str) {
        // 关闭所有面板
        self.show_add_dialog = false;
        self.show_tag_editor = false;
        self.show_settings = false;
        self.show_import_export = false;
        self.show_tag_manager = false;
        self.show_collection_manager = false;
        self.show_batch_collection_dialog = false;

        // 打开指定面板
        match panel {
            "add_dialog" => self.show_add_dialog = true,
            "tag_editor" => self.show_tag_editor = true,
            "settings" => self.show_settings = true,
            "import_export" => self.show_import_export = true,
            "tag_manager" => self.show_tag_manager = true,
            "collection_manager" => self.show_collection_manager = true,
            "batch_collection_dialog" => self.show_batch_collection_dialog = true,
            _ => {}
        }
    }

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
        let mut entries = user_data.entries.clone();
        
        // 执行数据迁移：为旧数据添加ID，转换集合的索引引用为ID引用
        Self::migrate_entries_to_id_system(&mut entries);

        let mut all_tags = HashSet::new();
        for entry in &entries {
            for tag in &entry.tags {
                all_tags.insert(tag.clone());
            }
        }

        let filtered_indices: Vec<usize> = (0..entries.len()).collect();

        // 从配置中恢复主题模式和紧凑模式
        let theme_mode = match config.theme_mode.as_str() {
            "Dark" => ThemeMode::Dark,
            "System" => ThemeMode::System,
            _ => ThemeMode::Light,
        };
        let _compact_mode = config.compact_mode;

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
            compact_mode: true, // 默认使用紧凑模式
            expanded_entries: HashSet::new(),
            filtered_indices,
            last_search_query: String::new(),
            last_filter_time: Instant::now(),
            add_path_input: String::new(),
            add_name_input: String::new(),
            add_nickname_input: String::new(),
            add_tags_input: String::new(),
            add_description_input: String::new(),
            show_add_dialog: false,
            show_tag_editor: false,
            editing_entry_index: None,
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
            add_entry_type: crate::file_entry::EntryType::File,

            // 删除确认对话框相关
            show_delete_confirm: false,
            delete_entry_index: None,
            delete_entry_name: String::new(),

            show_collection_manager: false,
            editing_collection_index: None,
            collection_child_selection: HashSet::new(),
            
            selected_entries: HashSet::new(),
            show_batch_collection_dialog: false,
            batch_collection_name: String::new(),
            
            focused_entry: None,
            search_has_focus: false,
            search_currently_focused: false,
            multi_select_mode: false,
        }
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        ModernTheme::apply_theme(ctx, self.theme_mode);
    }

    fn setup_fonts_once(&mut self, ctx: &egui::Context) {
        if self.font_loaded {
            return;
        }

        setup_chinese_fonts(ctx);
        self.font_loaded = true;
    }

    fn update_filter(&mut self) {
        // 只有搜索查询改变时才重新过滤
        if self.search_query != self.last_search_query {
            self.filtered_indices = self
                .entries
                .iter()
                .enumerate()
                .filter(|(_, entry)| self.matches_search_query(entry))
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
            .filter(|(_, entry)| self.matches_search_query(entry))
            .map(|(i, _)| i)
            .collect();

        self.last_search_query = self.search_query.clone();
        self.last_filter_time = Instant::now();
    }

    // 统一的搜索匹配函数，支持文件名、标签和描述搜索
    fn matches_search_query(&self, entry: &FileEntry) -> bool {
        if self.search_query.is_empty() {
            return true;
        }

        let query_lower = self.search_query.to_lowercase();
        let query_parts: Vec<&str> = query_lower.split_whitespace().collect();

        for part in query_parts {
            let found = if part.starts_with('#') {
                // 标签搜索
                let (hash_tags, _) = entry.get_tag_categories();
                hash_tags
                    .iter()
                    .any(|tag| tag.to_lowercase().contains(part))
            } else {
                // 普通搜索：文件名、昵称、描述
                entry.matches_query(part)
            };

            // 如果任何一个搜索词没有匹配，则不显示该条目
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
        self.config.compact_mode = self.compact_mode;
        self.config_manager.save_config(&self.config)
    }

    fn save_user_data(&mut self) -> Result<(), String> {
        self.user_data.entries = self.entries.clone();
        self.data_manager.save_data(&self.user_data)
    }

    fn add_entry(&mut self) {
        // 对于集合类型，不需要路径检查
        if self.add_entry_type != crate::file_entry::EntryType::Collection
            && self.add_path_input.is_empty()
        {
            return;
        }

        // 对于集合类型，名称是必需的
        if self.add_entry_type == crate::file_entry::EntryType::Collection
            && self.add_name_input.is_empty()
        {
            return;
        }

        let tags = FileEntry::parse_tags(&self.add_tags_input);
        let description = if self.add_description_input.is_empty() {
            None
        } else {
            Some(self.add_description_input.clone())
        };

        let nickname = if self.add_nickname_input.is_empty() {
            None
        } else {
            Some(self.add_nickname_input.clone())
        };

        let entry = match self.add_entry_type {
            crate::file_entry::EntryType::WebLink => {
                let name = if self.add_name_input.is_empty() {
                    // 从URL中提取网站名称作为默认名称
                    self.extract_site_name(&self.add_path_input)
                } else {
                    self.add_name_input.clone()
                };

                FileEntry::new_web_link(
                    name,
                    self.add_path_input.clone(),
                    nickname,
                    description,
                    tags.clone(),
                )
            }
            crate::file_entry::EntryType::Collection => {
                let mut child_entry_ids = Vec::new();
                for &idx in &self.collection_child_selection {
                    if let Some(entry) = self.entries.get(idx) {
                        child_entry_ids.push(entry.id.clone());
                    }
                }
                FileEntry::new_collection(
                    self.add_name_input.clone(),
                    nickname,
                    description,
                    tags.clone(),
                    child_entry_ids,
                )
            }
            _ => {
                let path = PathBuf::from(&self.add_path_input);
                let name = if self.add_name_input.is_empty() {
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("未命名")
                        .to_string()
                } else {
                    self.add_name_input.clone()
                };

                let is_directory = match self.add_entry_type {
                    crate::file_entry::EntryType::Directory => true,
                    _ => path.is_dir(),
                };

                FileEntry::new_with_nickname(
                    path,
                    name,
                    nickname,
                    description,
                    tags.clone(),
                    is_directory,
                )
            }
        };

        // 更新标签集合
        for tag in &tags {
            self.all_tags.insert(tag.clone());
        }

        self.entries.push(entry);
        let _ = self.save_user_data();

        // 清空输入框
        self.add_path_input.clear();
        self.add_name_input.clear();
        self.add_nickname_input.clear();
        self.add_tags_input.clear();
        self.add_description_input.clear();
        self.add_entry_type = crate::file_entry::EntryType::File;
        self.collection_child_selection.clear();
        self.show_add_dialog = false;

        // 强制重新过滤并更新索引
        self.force_update_filter();
    }

    fn remove_entry(&mut self, index: usize) {
        if index < self.entries.len() {
            let removed_entry = self.entries.remove(index);
            let removed_id = removed_entry.id.clone();

            // 从所有集合中移除对此条目的引用
            for entry in &mut self.entries {
                if entry.entry_type == crate::file_entry::EntryType::Collection {
                    entry.child_entries.retain(|id| id != &removed_id);
                }
            }

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
            let _ = std::process::Command::new("explorer").arg(path).spawn();
        }

        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open").arg(path).spawn();
        }

        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open").arg(path).spawn();
        }
    }

    fn open_entry(&self, entry: &FileEntry) {
        match entry.entry_type {
            crate::file_entry::EntryType::WebLink => {
                if let Some(url) = &entry.url {
                    self.open_url(url);
                }
            }
            crate::file_entry::EntryType::Collection => {
                self.open_collection(entry);
            }
            _ => {
                self.open_path(&entry.path);
            }
        }
    }

    fn open_url(&self, url: &str) {
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("cmd")
                .args(&["/C", "start", url])
                .spawn();
        }

        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open").arg(url).spawn();
        }

        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open").arg(url).spawn();
        }
    }

    fn open_collection(&self, collection: &FileEntry) {
        // 依次打开集合中的所有子项目，现在使用ID而不是索引
        for child_id in &collection.child_entries {
            if let Some(child_entry) = self.entries.iter().find(|e| &e.id == child_id) {
                self.open_entry(child_entry);

                // 在打开多个项目之间添加短暂延迟，避免系统过载
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    }

    fn edit_entry_tags(&mut self, index: usize) {
        if index < self.entries.len() {
            self.editing_entry_index = Some(index);
            let entry = &self.entries[index];
            self.add_tags_input = entry.tags.join(" ");
            self.add_nickname_input = entry.nickname.clone().unwrap_or_default();
            self.add_description_input = entry.description.clone().unwrap_or_default();
            self.show_tag_editor = true;
        }
    }

    fn save_entry_edit(&mut self) {
        if let Some(index) = self.editing_entry_index {
            if index < self.entries.len() {
                let new_tags = FileEntry::parse_tags(&self.add_tags_input);
                let new_nickname = if self.add_nickname_input.is_empty() {
                    None
                } else {
                    Some(self.add_nickname_input.clone())
                };
                let new_description = if self.add_description_input.is_empty() {
                    None
                } else {
                    Some(self.add_description_input.clone())
                };

                // 更新条目
                self.entries[index].tags = new_tags.clone();
                self.entries[index].nickname = new_nickname;
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
                Ok(json) => match std::fs::write(&path, json) {
                    Ok(_) => {
                        self.export_status = format!("导出成功: {}", path.display());
                    }
                    Err(e) => {
                        self.export_status = format!("导出失败: {}", e);
                    }
                },
                Err(e) => {
                    self.export_status = format!("序列化失败: {}", e);
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
                                self.import_status =
                                    format!("合并导入成功: {} 个条目", import_count);
                            } else {
                                // 替换模式：替换所有数据
                                self.entries = import_data.entries;
                                self.rebuild_tag_set();
                                self.import_status =
                                    format!("替换导入成功: {} 个条目", import_count);
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
                                    self.import_status =
                                        format!("合并导入成功(旧格式): {} 个条目", import_count);
                                } else {
                                    self.entries = entries;
                                    self.rebuild_tag_set();
                                    self.import_status =
                                        format!("替换导入成功(旧格式): {} 个条目", import_count);
                                }

                                let _ = self.save_user_data();
                                self.force_update_filter();
                            } else {
                                self.import_status = format!("文件格式错误: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    self.import_status = format!("读取文件失败: {}", e);
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
        let matching_tags: Vec<String> = self
            .all_tags
            .iter()
            .filter(|tag| tag.to_lowercase().contains(&input_lower) && !input_text.contains(*tag))
            .cloned()
            .collect();

        if !matching_tags.is_empty() {
            ui.small("建议:");
            ui.horizontal_wrapped(|ui| {
                for tag in matching_tags.iter().take(6) {
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
        ui.horizontal(|ui| {
            ui.heading("数据导入导出");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("×").clicked() {
                    self.show_import_export = false;
                }
            });
        });
        ui.separator();

        ui.label("导出数据:");
        if ui.button("导出").clicked() {
            self.export_data();
        }

        if !self.export_status.is_empty() {
            ui.horizontal(|ui| {
                ui.label(&self.export_status);
                if ui.small_button("×").clicked() {
                    self.export_status.clear();
                }
            });
        }

        ui.add_space(12.0);

        ui.label("导入数据:");
        ui.horizontal(|ui| {
            ui.radio_value(&mut self.import_merge_mode, true, "合并");
            ui.radio_value(&mut self.import_merge_mode, false, "替换");
        });
        ui.small(if self.import_merge_mode {
            "合并模式：新数据添加到现有数据"
        } else {
            "替换模式：清空现有数据"
        });

        if ui.button("导入").clicked() {
            self.import_data();
        }

        if !self.import_status.is_empty() {
            ui.horizontal(|ui| {
                ui.label(&self.import_status);
                if ui.small_button("×").clicked() {
                    self.import_status.clear();
                }
            });
        }

        ui.add_space(12.0);

        ui.label("批量操作:");
        ui.label("标签:");
        if ui.text_edit_singleline(&mut self.batch_tag_input).changed()
            && !self.batch_tag_input.is_empty()
        {
            self.render_tag_suggestions(ui, &self.batch_tag_input.clone());
        }

        ui.horizontal(|ui| {
            if ui.button("批量添加").clicked() && !self.batch_tag_input.is_empty() {
                self.batch_add_tags(&self.batch_tag_input.clone());
                self.batch_tag_input.clear();
            }
            if ui.button("批量移除").clicked() && !self.batch_tag_input.is_empty() {
                self.batch_remove_tags(&self.batch_tag_input.clone());
                self.batch_tag_input.clear();
            }
        });

        ui.label(format!("当前显示: {} 个条目", self.filtered_indices.len()));
    }

    fn render_tag_manager(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("标签管理");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("×").clicked() {
                    self.show_tag_manager = false;
                }
            });
        });
        ui.separator();

        let stats = self.get_tag_usage_stats();
        ui.label(format!("总计: {} 个标签", stats.len()));

        ui.label("筛选:");
        ui.text_edit_singleline(&mut self.tag_cloud_filter);

        ui.add_space(8.0);
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for (tag, count) in stats {
                    if self.tag_cloud_filter.is_empty()
                        || tag
                            .to_lowercase()
                            .contains(&self.tag_cloud_filter.to_lowercase())
                    {
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

                            if ui.button(&tag).clicked() {
                                self.search_query = format!("#{}", tag.trim_start_matches('#'));
                                self.force_update_filter();
                            }

                            ui.label(format!("({})", count));
                        });
                    }
                }
            });

        ui.add_space(12.0);

        ui.label("常用标签:");
        let common_tags = [
            "#工作", "#项目", "#文档", "#图片", "#视频", "#音频", "#重要", "#临时",
        ];
        ui.horizontal_wrapped(|ui| {
            for &tag in &common_tags {
                if ui.small_button(tag).clicked() {
                    let tag_query = format!("#{}", tag.trim_start_matches('#'));
                    if !self.search_query.contains(&tag_query) {
                        self.search_query = if self.search_query.is_empty() {
                            tag_query
                        } else {
                            format!("{} {}", self.search_query, tag_query)
                        };
                        self.force_update_filter();
                    }
                }
            }
        });
    }

    fn render_collection_manager(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("集合管理器");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("×").clicked() {
                    self.show_collection_manager = false;
                }
            });
        });
        ui.separator();

        // 选择要编辑的集合
        ui.label("选择集合:");
        let collections: Vec<(usize, &FileEntry)> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.entry_type == crate::file_entry::EntryType::Collection)
            .collect();

        if collections.is_empty() {
            ui.label("还没有创建任何集合");
            ui.small("请先添加一个集合类型的条目");
            return;
        }

        let mut selected_collection_name = self
            .editing_collection_index
            .and_then(|idx| collections.iter().find(|(i, _)| *i == idx))
            .map(|(_, entry)| entry.name.clone())
            .unwrap_or_else(|| "选择集合...".to_string());

        egui::ComboBox::from_label("")
            .selected_text(&selected_collection_name)
            .show_ui(ui, |ui| {
                for (index, entry) in &collections {
                    let response = ui.selectable_value(
                        &mut selected_collection_name,
                        entry.name.clone(),
                        &entry.name,
                    );
                    if response.clicked() {
                        self.editing_collection_index = Some(*index);
                        // 初始化子项选择状态，现在使用ID而不是索引
                        self.collection_child_selection.clear();
                        for child_id in &entry.child_entries {
                            // 找到对应ID的条目索引
                            if let Some(child_idx) = self.entries.iter().position(|e| &e.id == child_id) {
                                self.collection_child_selection.insert(child_idx);
                            }
                        }
                    }
                }
            });

        if let Some(collection_idx) = self.editing_collection_index {
            ui.add_space(12.0);

            if collection_idx < self.entries.len() {
                let collection_name = self.entries[collection_idx].name.clone();
                ui.label(format!("编辑集合: {}", collection_name));
                ui.separator();

                ui.label("选择要包含在集合中的项目:");

                // 显示可选择的项目（排除集合类型本身）
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for (idx, entry) in self.entries.iter().enumerate() {
                            if idx == collection_idx
                                || entry.entry_type == crate::file_entry::EntryType::Collection
                            {
                                continue; // 跳过当前编辑的集合和其他集合
                            }

                            let mut is_selected = self.collection_child_selection.contains(&idx);
                            let entry_icon = match entry.entry_type {
                                crate::file_entry::EntryType::File => "[F]",
                                crate::file_entry::EntryType::Directory => "[D]",
                                crate::file_entry::EntryType::WebLink => "[L]",
                                _ => "[?]",
                            };

                            ui.horizontal(|ui| {
                                if ui.checkbox(&mut is_selected, "").changed() {
                                    if is_selected {
                                        self.collection_child_selection.insert(idx);
                                    } else {
                                        self.collection_child_selection.remove(&idx);
                                    }
                                }
                                ui.label(format!("{} {}", entry_icon, entry.name));
                                if let Some(nickname) = &entry.nickname {
                                    ui.label(format!("({})", nickname));
                                }
                            });
                        }
                    });

                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    if ui.button("保存集合").clicked() {
                        // 更新集合的子项目，现在使用ID而不是索引
                        let mut child_ids = Vec::new();
                        for &selected_idx in &self.collection_child_selection {
                            if let Some(entry) = self.entries.get(selected_idx) {
                                child_ids.push(entry.id.clone());
                            }
                        }
                        if let Some(collection) = self.entries.get_mut(collection_idx) {
                            collection.child_entries = child_ids;
                            let _ = self.save_user_data();
                        }
                    }

                    if ui.button("取消").clicked() {
                        self.editing_collection_index = None;
                        self.collection_child_selection.clear();
                    }
                });

                // 显示当前集合信息
                if let Some(collection) = self.entries.get(collection_idx) {
                    ui.add_space(12.0);
                    ui.label(format!(
                        "当前集合包含 {} 个项目:",
                        collection.child_entries.len()
                    ));
                    for child_id in &collection.child_entries {
                        // 通过ID查找对应的条目
                        if let Some(child_entry) = self.entries.iter().find(|e| &e.id == child_id) {
                            let entry_icon = match child_entry.entry_type {
                                crate::file_entry::EntryType::File => "[F]",
                                crate::file_entry::EntryType::Directory => "[D]",
                                crate::file_entry::EntryType::WebLink => "[L]",
                                _ => "[?]",
                            };
                            ui.label(format!("  {} {}", entry_icon, child_entry.name));
                        } else {
                            // 如果找不到对应的条目，说明可能已被删除
                            ui.label(format!("  [已删除] ID: {}", child_id));
                        }
                    }
                }
            }
        }
    }

    fn render_batch_collection_dialog(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("批量创建集合");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("×").clicked() {
                    self.show_batch_collection_dialog = false;
                }
            });
        });
        ui.separator();

        ui.label(format!("将要创建包含 {} 个项目的集合:", self.selected_entries.len()));
        
        // 显示选中的项目
        egui::ScrollArea::vertical()
            .max_height(150.0)
            .show(ui, |ui| {
                for &idx in &self.selected_entries {
                    if let Some(entry) = self.entries.get(idx) {
                        let entry_icon = match entry.entry_type {
                            crate::file_entry::EntryType::File => "[F]",
                            crate::file_entry::EntryType::Directory => "[D]",
                            crate::file_entry::EntryType::WebLink => "[L]",
                            crate::file_entry::EntryType::Collection => "[C]",
                        };
                        ui.horizontal(|ui| {
                            ui.label(format!("{} {}", entry_icon, entry.name));
                            if let Some(nickname) = &entry.nickname {
                                ui.small(format!("({})", nickname));
                            }
                        });
                    }
                }
            });

        ui.add_space(12.0);
        ui.label("集合名称:");
        ui.text_edit_singleline(&mut self.batch_collection_name);

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if ui.button("创建集合").clicked() && !self.batch_collection_name.is_empty() {
                // 创建新集合，现在使用ID而不是索引
                let mut child_entry_ids = Vec::new();
                for &idx in &self.selected_entries {
                    if let Some(entry) = self.entries.get(idx) {
                        child_entry_ids.push(entry.id.clone());
                    }
                }
                let collection = FileEntry::new_collection(
                    self.batch_collection_name.clone(),
                    None,
                    None,
                    Vec::new(),
                    child_entry_ids,
                );
                
                self.entries.push(collection);
                let _ = self.save_user_data();
                
                // 清理状态
                self.batch_collection_name.clear();
                self.selected_entries.clear();
                self.multi_select_mode = false;
                self.show_batch_collection_dialog = false;
                
                // 更新过滤
                self.force_update_filter();
            }
            
            if ui.button("取消").clicked() {
                self.batch_collection_name.clear();
                self.show_batch_collection_dialog = false;
            }
        });
    }

    fn render_add_dialog(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("添加条目");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("×").clicked() {
                    self.show_add_dialog = false;
                }
            });
        });
        ui.separator();

        // 条目类型选择
        ui.label("类型:");
        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.add_entry_type,
                crate::file_entry::EntryType::File,
                "文件",
            );
            ui.radio_value(
                &mut self.add_entry_type,
                crate::file_entry::EntryType::Directory,
                "文件夹",
            );
            ui.radio_value(
                &mut self.add_entry_type,
                crate::file_entry::EntryType::WebLink,
                "网页链接",
            );
            ui.radio_value(
                &mut self.add_entry_type,
                crate::file_entry::EntryType::Collection,
                "集合",
            );
        });

        ui.add_space(8.0);

        // 根据类型显示不同的输入字段
        match self.add_entry_type {
            crate::file_entry::EntryType::WebLink => {
                ui.label("网页地址:");
                if ui.text_edit_singleline(&mut self.add_path_input).changed() {
                    // 当URL改变时，如果名称为空，自动填充网站名称
                    if self.add_name_input.is_empty() && self.is_valid_url(&self.add_path_input) {
                        self.add_name_input = self.extract_site_name(&self.add_path_input);
                    }
                }
                ui.small("请输入完整的URL，如: https://www.example.com");

                // URL验证提示
                if !self.add_path_input.is_empty() && !self.is_valid_url(&self.add_path_input) {
                    ui.colored_label(
                        egui::Color32::from_rgb(200, 50, 50),
                        "请输入有效的URL地址",
                    );
                } else if !self.add_path_input.is_empty() && self.is_valid_url(&self.add_path_input)
                {
                    ui.colored_label(egui::Color32::from_rgb(50, 150, 50), "URL格式正确");
                }
            }
            crate::file_entry::EntryType::Collection => {
                ui.label("集合信息:");
                ui.small("选择要包含在集合中的项目:");
                
                // 不需要路径输入，集合使用虚拟路径
                self.add_path_input.clear();
                
                ui.add_space(8.0);
                
                // 显示可选择的项目（排除集合类型）
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for (idx, entry) in self.entries.iter().enumerate() {
                            if entry.entry_type == crate::file_entry::EntryType::Collection {
                                continue; // 跳过其他集合
                            }

                            let mut is_selected = self.collection_child_selection.contains(&idx);
                            let entry_icon = match entry.entry_type {
                                crate::file_entry::EntryType::File => "📄",
                                crate::file_entry::EntryType::Directory => "📁",
                                crate::file_entry::EntryType::WebLink => "🌐",
                                _ => "📋",
                            };

                            ui.horizontal(|ui| {
                                if ui.checkbox(&mut is_selected, "").changed() {
                                    if is_selected {
                                        self.collection_child_selection.insert(idx);
                                    } else {
                                        self.collection_child_selection.remove(&idx);
                                    }
                                }
                                ui.label(format!("{} {}", entry_icon, entry.name));
                                if let Some(nickname) = &entry.nickname {
                                    ui.small(format!("({})", nickname));
                                }
                            });
                        }
                        
                        if self.entries.iter().all(|e| e.entry_type == crate::file_entry::EntryType::Collection) {
                            ui.label("没有可选择的项目");
                            ui.small("请先添加一些文件、文件夹或网页链接");
                        }
                    });
                
                if !self.collection_child_selection.is_empty() {
                    ui.add_space(4.0);
                    ui.label(format!("已选择 {} 个项目", self.collection_child_selection.len()));
                }
            }
            _ => {
                ui.label("路径:");
                ui.text_edit_singleline(&mut self.add_path_input);

                ui.horizontal(|ui| {
                    if ui.button("选择文件").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.add_path_input = path.to_string_lossy().to_string();
                            self.add_entry_type = crate::file_entry::EntryType::File;
                        }
                    }
                    if ui.button("选择文件夹").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.add_path_input = path.to_string_lossy().to_string();
                            self.add_entry_type = crate::file_entry::EntryType::Directory;
                        }
                    }
                });
            }
        }

        ui.add_space(8.0);
        ui.label("名称:");
        ui.text_edit_singleline(&mut self.add_name_input);

        ui.add_space(8.0);
        ui.label("昵称:");
        ui.text_edit_singleline(&mut self.add_nickname_input);
        ui.small("支持拼音搜索，如\"wdxm\"可搜索\"我的项目\"");

        ui.add_space(8.0);
        ui.label("标签:");
        if ui.text_edit_singleline(&mut self.add_tags_input).changed() {
            self.show_tag_suggestions = !self.add_tags_input.is_empty();
        }
        ui.small("使用 # 前缀，如: #重要 #工作");

        if self.show_tag_suggestions {
            self.render_tag_suggestions(ui, &self.add_tags_input.clone());
        }

        ui.add_space(8.0);
        ui.label("描述:");
        ui.text_edit_multiline(&mut self.add_description_input);

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            let can_add = match self.add_entry_type {
                crate::file_entry::EntryType::WebLink => {
                    !self.add_path_input.is_empty() && self.is_valid_url(&self.add_path_input)
                }
                crate::file_entry::EntryType::Collection => {
                    !self.add_name_input.is_empty()
                }
                _ => !self.add_path_input.is_empty(),
            };

            ui.add_enabled_ui(can_add, |ui| {
                if ui.button("添加").clicked() {
                    self.add_entry();
                }
            });
            if ui.button("取消").clicked() {
                self.show_add_dialog = false;
                self.add_path_input.clear();
                self.add_name_input.clear();
                self.add_nickname_input.clear();
                self.add_tags_input.clear();
                self.add_description_input.clear();
                self.add_entry_type = crate::file_entry::EntryType::File;
                self.collection_child_selection.clear();
            }
        });
    }

    fn render_tag_editor(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("编辑标签");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("×").clicked() {
                    self.show_tag_editor = false;
                }
            });
        });
        ui.separator();

        if let Some(index) = self.editing_entry_index {
            if index < self.entries.len() {
                let entry_name = &self.entries[index].name;
                ui.label(format!("编辑: {}", entry_name));
                ui.separator();
            }
        }

        ui.label("昵称 (可选):");
        ui.text_edit_singleline(&mut self.add_nickname_input);
        ui.small("昵称支持拼音搜索，例如：文件夹\"我是谁\"可以通过\"woshi\"搜索到");

        ui.add_space(8.0);
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
                self.add_nickname_input.clear();
                self.add_description_input.clear();
            }
        });
    }

    fn render_list(&mut self, ui: &mut egui::Ui) {
        let mut to_edit: Option<usize> = None;
        let mut to_expand: Option<usize> = None;
        let mut to_collapse: Option<usize> = None;
        let mut to_open: Option<usize> = None;
        let mut search_update: Option<String> = None;
        let mut remove_from_collection: Option<(usize, usize)> = None;
        let mut edit_collection: Option<usize> = None;

        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 50.0)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 4.0;

                for &index in &self.filtered_indices {
                    if index >= self.entries.len() {
                        continue;
                    }
                    let entry = &self.entries[index];
                    let entry_name = entry.name.clone();
                    let entry_nickname = entry.nickname.clone();
                    let (hash_tags, _path_tags) = entry.get_tag_categories();
                    let entry_description = entry.description.clone();
                    let entry_type = entry.entry_type.clone();
                    let entry_path = entry.path.clone();
                    let child_entries = entry.child_entries.clone();

                    let is_expanded = self.expanded_entries.contains(&index);

                    if self.compact_mode && !is_expanded {
                        // 紧凑模式：单行显示
                        let is_focused = self.focused_entry == Some(index);
                        let is_selected = self.selected_entries.contains(&index);
                        let item_bg = if is_selected {
                            if ui.ctx().style().visuals.dark_mode {
                                egui::Color32::from_rgb(90, 90, 140)
                            } else {
                                egui::Color32::from_rgb(160, 160, 250)
                            }
                        } else if is_focused {
                            if ui.ctx().style().visuals.dark_mode {
                                egui::Color32::from_rgb(60, 60, 80)
                            } else {
                                egui::Color32::from_rgb(220, 220, 240)
                            }
                        } else {
                            egui::Color32::TRANSPARENT
                        };
                        
                        let item_frame = egui::Frame::none()
                            .fill(item_bg)
                            .rounding(egui::Rounding::same(4.0))
                            .inner_margin(egui::Margin::same(4.0))
                            .stroke(if is_selected {
                                egui::Stroke::new(3.0, egui::Color32::from_rgb(80, 120, 200))
                            } else if self.multi_select_mode {
                                egui::Stroke::new(1.0, egui::Color32::from_gray(150))
                            } else {
                                egui::Stroke::NONE
                            });
                            
                        let item_response = item_frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // 多选模式下显示选择状态
                                if self.multi_select_mode {
                                    let mut checkbox_selected = is_selected;
                                    if ui.checkbox(&mut checkbox_selected, "").changed() {
                                        if checkbox_selected {
                                            self.selected_entries.insert(index);
                                        } else {
                                            self.selected_entries.remove(&index);
                                        }
                                    }
                                }
                                
                                // 展开按钮
                                if ui.small_button("+").clicked() {
                                    to_expand = Some(index);
                                }

                                let icon = match entry_type {
                                    crate::file_entry::EntryType::Directory => "[D]",
                                    crate::file_entry::EntryType::WebLink => "[L]",
                                    crate::file_entry::EntryType::Collection => "[C]",
                                    _ => "[F]",
                                };
                                ui.label(icon);

                                // 文件名/昵称
                                if let Some(nickname) = &entry_nickname {
                                    if ui.link(nickname).clicked() {
                                        to_open = Some(index);
                                    }
                                } else {
                                    if ui.link(&entry_name).clicked() {
                                        to_open = Some(index);
                                    }
                                }

                                // 标签（只显示第一个）
                                if !hash_tags.is_empty() {
                                    ui.small(&hash_tags[0]);
                                    if hash_tags.len() > 1 {
                                        ui.small(format!("+{}", hash_tags.len() - 1));
                                    }
                                }

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.small_button("×").clicked() {
                                            self.show_delete_confirm = true;
                                            self.delete_entry_index = Some(index);
                                            self.delete_entry_name =
                                                if let Some(nickname) = &entry_nickname {
                                                    nickname.clone()
                                                } else {
                                                    entry_name.clone()
                                                };
                                        }
                                        if ui.small_button("编辑").clicked() {
                                            to_edit = Some(index);
                                        }
                                    },
                                );
                            })
                        }).response;
                        
                        // 处理点击聚焦（多选模式下不处理，由checkbox控制）
                        if item_response.clicked() && !self.multi_select_mode {
                            self.focused_entry = Some(index);
                        }
                        
                        // 右键菜单
                        item_response.context_menu(|ui| {
                            if self.multi_select_mode && !self.selected_entries.is_empty() {
                                // 多选模式的右键菜单
                                ui.label(format!("已选择 {} 个项目", self.selected_entries.len()));
                                ui.separator();
                                
                                if ui.button("创建集合").clicked() {
                                    self.batch_collection_name.clear();
                                    self.show_batch_collection_dialog = true;
                                    ui.close_menu();
                                }
                                
                                if ui.button("删除选中项目").clicked() {
                                    // 这里可以实现批量删除逻辑
                                    ui.close_menu();
                                }
                                
                                ui.separator();
                                if ui.button("退出多选模式").clicked() {
                                    self.multi_select_mode = false;
                                    self.selected_entries.clear();
                                    ui.close_menu();
                                }
                            } else {
                                // 单选模式的右键菜单
                                if ui.button("打开").clicked() {
                                    to_open = Some(index);
                                    ui.close_menu();
                                }
                                if ui.button("编辑").clicked() {
                                    to_edit = Some(index);
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui.button("多选").clicked() {
                                    self.multi_select_mode = true;
                                    self.selected_entries.insert(index);
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui.button("删除").clicked() {
                                    self.show_delete_confirm = true;
                                    self.delete_entry_index = Some(index);
                                    self.delete_entry_name = if let Some(nickname) = &entry_nickname {
                                        nickname.clone()
                                    } else {
                                        entry_name.clone()
                                    };
                                    ui.close_menu();
                                }
                            }
                        });
                    } else {
                        // 展开模式：多行显示
                        let is_focused = self.focused_entry == Some(index);
                        let is_selected = self.selected_entries.contains(&index);
                        let item_bg = if is_selected {
                            if ui.ctx().style().visuals.dark_mode {
                                egui::Color32::from_rgb(90, 90, 140)
                            } else {
                                egui::Color32::from_rgb(160, 160, 250)
                            }
                        } else if is_focused {
                            if ui.ctx().style().visuals.dark_mode {
                                egui::Color32::from_rgb(60, 60, 80)
                            } else {
                                egui::Color32::from_rgb(220, 220, 240)
                            }
                        } else {
                            egui::Color32::TRANSPARENT
                        };
                        
                        let item_frame = egui::Frame::none()
                            .fill(item_bg)
                            .rounding(egui::Rounding::same(6.0))
                            .inner_margin(egui::Margin::same(6.0))
                            .stroke(if is_selected {
                                egui::Stroke::new(3.0, egui::Color32::from_rgb(80, 120, 200))
                            } else if self.multi_select_mode {
                                egui::Stroke::new(1.0, egui::Color32::from_gray(150))
                            } else {
                                egui::Stroke::NONE
                            });
                            
                        let item_response = item_frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // 多选模式下显示选择状态
                                if self.multi_select_mode {
                                    let mut checkbox_selected = is_selected;
                                    if ui.checkbox(&mut checkbox_selected, "").changed() {
                                        if checkbox_selected {
                                            self.selected_entries.insert(index);
                                        } else {
                                            self.selected_entries.remove(&index);
                                        }
                                    }
                                }
                                
                                // 点击收起
                                if is_expanded && ui.small_button("-").clicked() {
                                    to_collapse = Some(index);
                                }

                                // 文件图标
                                let icon = match entry_type {
                                    crate::file_entry::EntryType::Directory => "[D]",
                                    crate::file_entry::EntryType::WebLink => "[L]",
                                    crate::file_entry::EntryType::Collection => "[C]",
                                    _ => "[F]",
                                };
                                ui.label(icon);

                                // 主要信息
                                ui.vertical(|ui| {
                                    ui.spacing_mut().item_spacing.y = 2.0;

                                    // 文件名/昵称
                                    if let Some(nickname) = &entry_nickname {
                                        if ui.link(nickname).clicked() {
                                            to_open = Some(index);
                                        }
                                        ui.small(&entry_name);
                                    } else {
                                        if ui.link(&entry_name).clicked() {
                                            to_open = Some(index);
                                        }
                                    }

                                    // 描述（如果有）
                                    if let Some(desc) = &entry_description {
                                        ui.small(desc);
                                    }

                                    // 标签（完整显示）
                                    if !hash_tags.is_empty() {
                                        ui.horizontal(|ui| {
                                            ui.small("Tags:");
                                            for tag in &hash_tags {
                                                if ui.small_button(tag).clicked() {
                                                    let tag_query = format!("#{}", tag.trim_start_matches('#'));
                                                    if !self.search_query.contains(&tag_query) {
                                                        let new_query = if self.search_query.is_empty() {
                                                            tag_query
                                                        } else {
                                                            format!("{} {}", self.search_query, tag_query)
                                                        };
                                                        search_update = Some(new_query);
                                                    }
                                                }
                                            }
                                        });
                                    }

                                    // 集合子项目显示
                                    if entry_type == crate::file_entry::EntryType::Collection {
                                        ui.add_space(6.0);
                                        
                                        // 集合容器样式
                                        let collection_bg = if ui.ctx().style().visuals.dark_mode {
                                            egui::Color32::from_rgba_unmultiplied(60, 60, 70, 100)
                                        } else {
                                            egui::Color32::from_rgba_unmultiplied(240, 240, 245, 150)
                                        };
                                        
                                        let collection_frame = egui::Frame::none()
                                            .fill(collection_bg)
                                            .rounding(egui::Rounding::same(6.0))
                                            .inner_margin(egui::Margin::same(8.0))
                                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(100)));
                                        
                                        collection_frame.show(ui, |ui| {
                                            if !child_entries.is_empty() {
                                                ui.horizontal(|ui| {
                                                    ui.label("集合:");
                                                    ui.label(egui::RichText::new(format!("包含 {} 个项目", child_entries.len()))
                                                        .size(12.0)
                                                        .color(egui::Color32::from_gray(150)));
                                                });
                                                
                                                ui.add_space(4.0);
                                                
                                                for (i, child_id) in child_entries.iter().enumerate() {
                                                    if let Some(child_entry) = self.entries.iter().find(|e| &e.id == child_id) {
                                                        ui.horizontal(|ui| {
                                                            // 连接线
                                                            if i == child_entries.len() - 1 {
                                                                ui.label("└─");
                                                            } else {
                                                                ui.label("├─");
                                                            }
                                                
                                                            let child_icon = match child_entry.entry_type {
                                                                crate::file_entry::EntryType::File => "[F]",
                                                                crate::file_entry::EntryType::Directory => "[D]",
                                                                crate::file_entry::EntryType::WebLink => "[L]",
                                                                _ => "[?]",
                                                            };
                                                
                                                            // 可点击的子项目链接
                                                            let child_response = ui.add(
                                                                egui::Label::new(
                                                                    egui::RichText::new(format!("{} {}", child_icon, child_entry.name))
                                                                        .size(11.0)
                                                                        .color(egui::Color32::from_rgb(100, 150, 200))
                                                                ).sense(egui::Sense::click())
                                                            );
                                                
                                                            if child_response.clicked() {
                                                                if let Some(child_idx) = self.entries.iter().position(|e| &e.id == child_id) {
                                                                    to_open = Some(child_idx);
                                                                }
                                                            }
                                                
                                                            if let Some(nickname) = &child_entry.nickname {
                                                                ui.label(egui::RichText::new(format!("({})", nickname))
                                                                    .size(10.0)
                                                                    .color(egui::Color32::from_gray(120)));
                                                            }
                                                
                                                            // 使用固定宽度的空间来避免与集合编辑按钮重合
                                                            ui.allocate_ui_with_layout(
                                                                [30.0, 20.0].into(),
                                                                egui::Layout::right_to_left(egui::Align::Center),
                                                                |ui| {
                                                                    if ui.small_button("－").on_hover_text("从集合中移除").clicked() {
                                                                        if let Some(child_idx) = self.entries.iter().position(|e| &e.id == child_id) {
                                                                            remove_from_collection = Some((index, child_idx));
                                                                        }
                                                                    }
                                                                }
                                                            );
                                                        });
                                                    }
                                                }
                                                
                                                ui.add_space(4.0);
                                                ui.separator();
                                                
                                                ui.horizontal(|ui| {
                                                    if ui.button("+ 添加更多").clicked() {
                                                        edit_collection = Some(index);
                                                    }
                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        if ui.small_button("管理集合").clicked() {
                                                            edit_collection = Some(index);
                                                        }
                                                    });
                                                });
                                            } else {
                                                ui.horizontal(|ui| {
                                                    ui.label("集合:");
                                                    ui.label(egui::RichText::new("空集合")
                                                        .size(12.0)
                                                        .color(egui::Color32::from_gray(150)));
                                                });
                                                
                                                ui.add_space(4.0);
                                                
                                                if ui.button("+ 添加项目").clicked() {
                                                    edit_collection = Some(index);
                                                }
                                            }
                                        });
                                    } else {
                                        // 非集合类型显示路径
                                        let display_path = if entry_type == crate::file_entry::EntryType::WebLink {
                                            entry.url.clone().unwrap_or_else(|| entry_path.to_string_lossy().to_string())
                                        } else {
                                            entry_path.to_string_lossy().to_string()
                                        };

                                        if !display_path.is_empty() {
                                            ui.small(format!("Path: {}", display_path));
                                        }
                                    }
                                });

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.small_button("×").clicked() {
                                            self.show_delete_confirm = true;
                                            self.delete_entry_index = Some(index);
                                            self.delete_entry_name = if let Some(nickname) = &entry_nickname {
                                                nickname.clone()
                                            } else {
                                                entry_name.clone()
                                            };
                                        }
                                        if ui.small_button("编辑").clicked() {
                                            to_edit = Some(index);
                                        }
                                    },
                                );
                            })
                        }).response;
                        
                        // 处理点击聚焦（多选模式下不处理，由checkbox控制）
                        if item_response.clicked() && !self.multi_select_mode {
                            self.focused_entry = Some(index);
                        }
                        
                        // 右键菜单
                        item_response.context_menu(|ui| {
                            if self.multi_select_mode && !self.selected_entries.is_empty() {
                                // 多选模式的右键菜单
                                ui.label(format!("已选择 {} 个项目", self.selected_entries.len()));
                                ui.separator();
                                
                                if ui.button("创建集合").clicked() {
                                    self.batch_collection_name.clear();
                                    self.show_batch_collection_dialog = true;
                                    ui.close_menu();
                                }
                                
                                if ui.button("删除选中项目").clicked() {
                                    // 这里可以实现批量删除逻辑
                                    ui.close_menu();
                                }
                                
                                ui.separator();
                                if ui.button("退出多选模式").clicked() {
                                    self.multi_select_mode = false;
                                    self.selected_entries.clear();
                                    ui.close_menu();
                                }
                            } else {
                                // 单选模式的右键菜单
                                if ui.button("打开").clicked() {
                                    to_open = Some(index);
                                    ui.close_menu();
                                }
                                if ui.button("编辑").clicked() {
                                    to_edit = Some(index);
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui.button("多选").clicked() {
                                    self.multi_select_mode = true;
                                    self.selected_entries.insert(index);
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui.button("删除").clicked() {
                                    self.show_delete_confirm = true;
                                    self.delete_entry_index = Some(index);
                                    self.delete_entry_name = if let Some(nickname) = &entry_nickname {
                                        nickname.clone()
                                    } else {
                                        entry_name.clone()
                                    };
                                    ui.close_menu();
                                }
                            }
                        });
                    }

                    ui.separator();
                }

                if self.filtered_indices.is_empty() {
                    ui.add_space(40.0);
                    ui.vertical_centered(|ui| {
                        if self.entries.is_empty() {
                            ui.label("还没有添加任何文件");
                            ui.small("点击'添加'按钮开始");
                        } else {
                            ui.label("没有找到匹配的结果");
                            ui.small("尝试调整搜索条件");
                        }
                    });
                }
            });

        // 处理延迟操作
        if let Some(index) = to_expand {
            self.expanded_entries.insert(index);
        }
        if let Some(index) = to_collapse {
            self.expanded_entries.remove(&index);
        }
        if let Some(index) = to_open {
            if let Some(entry) = self.entries.get(index) {
                self.open_entry(entry);
            }
        }
        if let Some(index) = to_edit {
            self.edit_entry_tags(index);
        }
        if let Some(query) = search_update {
            self.search_query = query;
            self.force_update_filter();
        }
        if let Some((collection_idx, child_idx)) = remove_from_collection {
            // 先获取子项目的ID，避免借用冲突
            if let Some(child_entry) = self.entries.get(child_idx) {
                let child_id = child_entry.id.clone();
                if let Some(collection) = self.entries.get_mut(collection_idx) {
                    collection.child_entries.retain(|x| x != &child_id);
                    let _ = self.save_user_data();
                }
            }
        }
        if let Some(collection_idx) = edit_collection {
            if let Some(collection_entry) = self.entries.get(collection_idx) {
                self.editing_collection_index = Some(collection_idx);
                self.collection_child_selection.clear();
                for child_id in &collection_entry.child_entries {
                    if let Some(child_idx) = self.entries.iter().position(|e| &e.id == child_id) {
                        self.collection_child_selection.insert(child_idx);
                    }
                }
                self.show_collection_manager = true;
            }
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            let cmd = if cfg!(target_os = "macos") {
                i.modifiers.mac_cmd
            } else {
                i.modifiers.ctrl
            };

            // Cmd/Ctrl+N: 添加新条目
            if cmd && i.key_pressed(egui::Key::N) && !self.search_currently_focused {
                self.toggle_panel("add_dialog");
            }

            // Cmd/Ctrl+F: 聚焦搜索框
            if cmd && i.key_pressed(egui::Key::F) {
                self.search_has_focus = true;
            }

            // Enter: 打开选中的条目
            if i.key_pressed(egui::Key::Enter) && !self.search_currently_focused {
                if let Some(focused_idx) = self.focused_entry {
                    if let Some(entry) = self.entries.get(focused_idx) {
                        self.open_entry(entry);
                    }
                }
            }

            // Delete: 删除选中的条目
            if i.key_pressed(egui::Key::Delete) && !self.search_currently_focused {
                if let Some(focused_idx) = self.focused_entry {
                    self.show_delete_confirm = true;
                    self.delete_entry_index = Some(focused_idx);
                    if let Some(entry) = self.entries.get(focused_idx) {
                        self.delete_entry_name = entry.nickname.clone().unwrap_or_else(|| entry.name.clone());
                    }
                }
            }

            // Cmd/Ctrl+E: 编辑选中的条目
            if cmd && i.key_pressed(egui::Key::E) && !self.search_currently_focused {
                if let Some(focused_idx) = self.focused_entry {
                    self.edit_entry_tags(focused_idx);
                }
            }

            // Cmd/Ctrl+R: 打开/关闭右侧面板
            if cmd && i.key_pressed(egui::Key::R) && !self.search_currently_focused {
                let any_panel_open = self.show_add_dialog
                    || self.show_tag_editor
                    || self.show_settings
                    || self.show_import_export
                    || self.show_tag_manager
                    || self.show_collection_manager
                    || self.show_batch_collection_dialog;
                
                if any_panel_open {
                    // 关闭所有面板
                    self.show_add_dialog = false;
                    self.show_tag_editor = false;
                    self.show_settings = false;
                    self.show_import_export = false;
                    self.show_tag_manager = false;
                    self.show_collection_manager = false;
                    self.show_batch_collection_dialog = false;
                } else {
                    // 打开设置面板作为默认
                    self.show_settings = true;
                }
            }

            // Escape: 退出当前模式/关闭对话框
            if i.key_pressed(egui::Key::Escape) {
                if self.multi_select_mode {
                    self.multi_select_mode = false;
                    self.selected_entries.clear();
                } else if self.show_add_dialog || self.show_tag_editor || self.show_settings || 
                         self.show_import_export || self.show_tag_manager || self.show_collection_manager ||
                         self.show_batch_collection_dialog {
                    self.show_add_dialog = false;
                    self.show_tag_editor = false;
                    self.show_settings = false;
                    self.show_import_export = false;
                    self.show_tag_manager = false;
                    self.show_collection_manager = false;
                    self.show_batch_collection_dialog = false;
                }
                self.search_has_focus = false;
            }

            // 上下箭头键：选择条目
            if !self.search_currently_focused && !self.filtered_indices.is_empty() {
                if i.key_pressed(egui::Key::ArrowDown) {
                    if let Some(current) = self.focused_entry {
                        if let Some(pos) = self.filtered_indices.iter().position(|&x| x == current) {
                            if pos + 1 < self.filtered_indices.len() {
                                self.focused_entry = Some(self.filtered_indices[pos + 1]);
                            }
                        }
                    } else if !self.filtered_indices.is_empty() {
                        self.focused_entry = Some(self.filtered_indices[0]);
                    }
                }
                
                if i.key_pressed(egui::Key::ArrowUp) {
                    if let Some(current) = self.focused_entry {
                        if let Some(pos) = self.filtered_indices.iter().position(|&x| x == current) {
                            if pos > 0 {
                                self.focused_entry = Some(self.filtered_indices[pos - 1]);
                            }
                        }
                    } else if !self.filtered_indices.is_empty() {
                        self.focused_entry = Some(self.filtered_indices[self.filtered_indices.len() - 1]);
                    }
                }
            }
        });
    }

    /// 验证URL格式
    fn is_valid_url(&self, url: &str) -> bool {
        if url.is_empty() {
            return false;
        }

        // 基本URL格式验证
        if !(url.starts_with("http://") || url.starts_with("https://") || url.starts_with("ftp://"))
        {
            return false;
        }

        // 检查是否包含域名
        if let Some(domain_start) = url.find("://") {
            let remaining = &url[domain_start + 3..];
            if remaining.is_empty() || remaining.starts_with('/') {
                return false;
            }
            // 简单检查域名是否包含点号
            let domain_part = remaining.split('/').next().unwrap_or("");
            return domain_part.contains('.') && domain_part.len() > 3;
        }

        false
    }

    /// 从URL提取网站名称
    fn extract_site_name(&self, url: &str) -> String {
        if let Some(domain_start) = url.find("://") {
            let remaining = &url[domain_start + 3..];
            if let Some(domain_end) = remaining.find('/') {
                let domain = &remaining[..domain_end];
                // 移除 www. 前缀
                if domain.starts_with("www.") {
                    domain[4..].to_string()
                } else {
                    domain.to_string()
                }
            } else {
                // 移除 www. 前缀
                if remaining.starts_with("www.") {
                    remaining[4..].to_string()
                } else {
                    remaining.to_string()
                }
            }
        } else {
            url.to_string()
        }
    }

    fn render_settings(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("设置");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("×").clicked() {
                    self.show_settings = false;
                }
            });
        });
        ui.separator();

        ui.label("主题:");
        let old_theme = self.theme_mode;
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.theme_mode, ThemeMode::Light, "浅色");
            ui.selectable_value(&mut self.theme_mode, ThemeMode::Dark, "深色");
            ui.selectable_value(&mut self.theme_mode, ThemeMode::System, "跟随系统");
        });

        if self.theme_mode != old_theme {
            let _ = self.save_config();
        }

        ui.add_space(16.0);
        ui.label(format!("文件数量: {}", self.entries.len()));
        ui.label(format!("标签数量: {}", self.all_tags.len()));

        ui.add_space(16.0);
        ui.label("显示模式:");
        let old_compact = self.compact_mode;
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.compact_mode, false, "普通");
            ui.selectable_value(&mut self.compact_mode, true, "紧凑");
        });

        if self.compact_mode != old_compact {
            let _ = self.save_config();
        }

        ui.add_space(16.0);
        ui.collapsing("数据备份", |ui| {
            ui.label("快速备份当前数据");
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("创建备份").clicked() {
                    let backup_name = format!(
                        "backup_{}.json",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                    );

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
                            Ok(json) => match std::fs::write(&path, json) {
                                Ok(_) => {
                                    self.export_status = format!("备份成功: {}", path.display());
                                }
                                Err(e) => {
                                    self.export_status = format!("备份失败: {}", e);
                                }
                            },
                            Err(e) => {
                                self.export_status = format!("备份序列化失败: {}", e);
                            }
                        }
                    }
                }

                if ui.button("从备份恢复").clicked() {
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
            ui.label("提示: 建议定期备份数据以防丢失");
        });

        ui.add_space(16.0);
        ui.collapsing("应用配置文件", |ui| {
            ui.label("配置文件格式: JSON");
            ui.label(format!(
                "当前位置: {}",
                self.config_manager.get_config_path().display()
            ));

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label("自定义配置路径:");
                ui.text_edit_singleline(&mut self.custom_config_path);
            });

            ui.horizontal(|ui| {
                if ui.button("选择位置").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("JSON文件", &["json"])
                        .set_file_name("file_manager_config.json")
                        .save_file()
                    {
                        self.custom_config_path = path.to_string_lossy().to_string();
                    }
                }

                if ui.button("应用配置路径").clicked() && !self.custom_config_path.is_empty()
                {
                    let new_path = PathBuf::from(&self.custom_config_path);
                    self.config_manager = ConfigManager::new_with_path(new_path);
                    if let Err(e) = self.save_config() {
                        ui.label(format!("保存配置失败: {}", e));
                    } else {
                        ui.label("配置路径已更新");
                    }
                }

                if ui.button("重置配置路径").clicked() {
                    self.config_manager = ConfigManager::new();
                    self.custom_config_path.clear();
                    let _ = self.save_config();
                }
            });
        });

        ui.add_space(8.0);
        ui.collapsing("用户数据文件", |ui| {
            ui.label("数据文件格式: JSON");
            ui.label(format!(
                "当前位置: {}",
                self.data_manager.get_data_path().display()
            ));

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label("自定义数据路径:");
                ui.text_edit_singleline(&mut self.custom_data_path);
            });

            ui.horizontal(|ui| {
                if ui.button("选择位置").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("JSON文件", &["json"])
                        .set_file_name("file_manager_data.json")
                        .save_file()
                    {
                        self.custom_data_path = path.to_string_lossy().to_string();
                    }
                }

                if ui.button("应用数据路径").clicked() && !self.custom_data_path.is_empty() {
                    let new_path = PathBuf::from(&self.custom_data_path);

                    // 先保存当前数据到新位置
                    let old_manager = std::mem::replace(
                        &mut self.data_manager,
                        DataManager::new_with_path(new_path),
                    );
                    self.config.data_file_path = Some(self.custom_data_path.clone());

                    if let Err(e) = self.save_user_data() {
                        // 如果保存失败，恢复原来的数据管理器
                        self.data_manager = old_manager;
                        self.config.data_file_path = None;
                        ui.label(format!("保存数据失败: {}", e));
                    } else {
                        // 保存配置中的数据路径
                        let _ = self.save_config();
                        ui.label("数据路径已更新");
                    }
                }

                if ui.button("重置数据路径").clicked() {
                    self.data_manager = DataManager::new();
                    self.custom_data_path.clear();
                    self.config.data_file_path = None;
                    let _ = self.save_config();
                    let _ = self.save_user_data();
                }
            });

            ui.add_space(8.0);
            ui.label("提示:");
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
                                    let tag_query = format!("#{}", tag.trim_start_matches('#'));
                                    self.search_query = tag_query;
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

    fn render_delete_confirm_dialog(&mut self, ctx: &egui::Context) {
        if self.show_delete_confirm {
            egui::Window::new("确认删除")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.label(format!("确定要删除 \"{}\" 吗？", self.delete_entry_name));
                        ui.add_space(10.0);
                        ui.label("此操作无法撤销。");
                        ui.add_space(20.0);

                        ui.horizontal(|ui| {
                            if ui.button("取消").clicked() {
                                self.show_delete_confirm = false;
                                self.delete_entry_index = None;
                                self.delete_entry_name.clear();
                            }

                            ui.add_space(20.0);

                            if ui.button("确认删除").clicked() {
                                if let Some(index) = self.delete_entry_index {
                                    self.remove_entry(index);
                                }
                                self.show_delete_confirm = false;
                                self.delete_entry_index = None;
                                self.delete_entry_name.clear();
                            }
                        });
                        ui.add_space(10.0);
                    });
                });
        }
    }
}

impl eframe::App for FileManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 只在第一次设置字体
        self.setup_fonts_once(ctx);

        // 应用主题
        self.apply_theme(ctx);

        // 处理快捷键
        self.handle_shortcuts(ctx);

        // 处理拖拽文件
        ctx.input(|i| {
            for file in &i.raw.dropped_files {
                if let Some(path) = &file.path {
                    // 直接添加拖拽的文件，而不是只设置到输入框
                    let path_buf = path.clone();
                    let name = path_buf
                        .file_name()
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

        // 简洁的顶部工具栏
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("文件管理器").strong());
                ui.separator();

                // 统一搜索框（支持文件名、标签和描述）
                ui.label("搜索:");
                let search_response = ui.add_sized(
                    [200.0, 20.0],
                    egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("搜索文件、标签...")
                );
                
                // 处理搜索框焦点
                if self.search_has_focus {
                    search_response.request_focus();
                    self.search_has_focus = false;
                }
                
                // 检测搜索框当前是否有焦点，用于确定是否启用快捷键
                self.search_currently_focused = search_response.has_focus();
                if ui.ctx().input(|i| i.key_pressed(egui::Key::Enter) && self.search_currently_focused)
                    || self.search_query != self.last_search_query
                {
                    self.force_update_filter();
                }

                if !self.search_query.is_empty() && ui.small_button("清除").clicked() {
                    self.search_query.clear();
                    self.force_update_filter();
                }

                ui.separator();

                ui.horizontal_wrapped(|ui| {
                    // 主要功能按钮
                    if ui.button("添加").clicked() {
                        self.toggle_panel("add_dialog");
                    }

                    if ui.button("标签").clicked() {
                        self.toggle_panel("tag_manager");
                    }

                    if ui.button("集合").clicked() {
                        self.toggle_panel("collection_manager");
                    }

                    if ui.button("导入导出").clicked() {
                        self.toggle_panel("import_export");
                    }

                    if ui.button("设置").clicked() {
                        self.toggle_panel("settings");
                    }
                });

                // 显示多选状态和批量操作
                if self.multi_select_mode {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(100, 150, 200),
                            format!("多选模式 - 已选择 {} 个项目", self.selected_entries.len())
                        );
                        
                        if !self.selected_entries.is_empty() {
                            if ui.button("创建集合").clicked() {
                                self.batch_collection_name.clear();
                                self.toggle_panel("batch_collection_dialog");
                            }
                        }
                        
                        if ui.button("退出多选").clicked() {
                            self.selected_entries.clear();
                            self.multi_select_mode = false;
                        }
                    });
                }
                
                // 快捷键提示（在底部）
                ui.separator();
                ui.horizontal(|ui| {
                    let cmd_key = if cfg!(target_os = "macos") { "Cmd" } else { "Ctrl" };
                    if self.multi_select_mode {
                        ui.small("多选模式：点击项目切换选择状态，右键查看批量操作");
                    } else {
                        ui.small(format!("右键多选 {}+N:添加 {}+F:搜索 {}+R:面板", cmd_key, cmd_key, cmd_key));
                    }
                });

// 关闭侧边栏按钮
if self.show_add_dialog
    || self.show_tag_editor
    || self.show_settings
    || self.show_import_export
    || self.show_tag_manager
    || self.show_collection_manager
    || self.show_batch_collection_dialog
{
    if ui.button("×").clicked() {
        self.show_add_dialog = false;
        self.show_tag_editor = false;
        self.show_settings = false;
        self.show_import_export = false;
        self.show_tag_manager = false;
        self.show_collection_manager = false;
        self.show_batch_collection_dialog = false;
    }
}
            });
        });

        // 侧边面板
        if self.show_add_dialog
            || self.show_tag_editor
            || self.show_settings
            || self.show_import_export
            || self.show_tag_manager
            || self.show_collection_manager
            || self.show_batch_collection_dialog
        {
            egui::SidePanel::right("side")
                .width_range(250.0..=300.0)
                .default_width(280.0)
                .show(ctx, |ui| {
                    if self.show_add_dialog {
                        self.render_add_dialog(ui);
                    } else if self.show_tag_editor {
                        self.render_tag_editor(ui);
                    } else if self.show_import_export {
                        self.render_import_export(ui);
                    } else if self.show_tag_manager {
                        self.render_tag_manager(ui);
                    } else if self.show_collection_manager {
                        self.render_collection_manager(ui);
                    } else if self.show_batch_collection_dialog {
                        self.render_batch_collection_dialog(ui);
                    } else if self.show_settings {
                        self.render_settings(ui);
                    }
                });
        }

        // 主面板
        egui::CentralPanel::default().show(ctx, |ui| {
            self.update_filter();
            self.render_list(ui);
        });

        // 删除确认对话框
        self.render_delete_confirm_dialog(ctx);
    }
}
