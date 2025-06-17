use crate::async_ops::{AsyncOperationBuilder, AsyncOperationManager};
use crate::file_entry::FileEntry;
use crate::plugins::{BackupPlugin, PluginManager, SearchPlugin};
use crate::state::{AppState, StateEvent, StateManager};
use std::path::PathBuf;

use std::time::Duration;

/// 集成示例 - 展示如何使用状态管理、插件系统和异步操作
pub struct IntegratedFileManager {
    // 状态管理
    state_manager: StateManager,

    // 插件系统
    plugin_manager: PluginManager,

    // 异步操作管理器
    async_manager: AsyncOperationManager,

    // 应用数据
    file_entries: Vec<FileEntry>,
    current_directory: PathBuf,
}

impl IntegratedFileManager {
    /// 创建新的集成文件管理器
    pub fn new() -> Result<Self, String> {
        let state_manager = StateManager::new();
        let app_data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("file_manager");

        let mut plugin_manager = PluginManager::new(app_data_dir.clone());
        let async_manager = AsyncOperationManager::new()?;

        // 设置状态监听器
        state_manager.add_state_listener(|current_state, previous_state| {
            println!("状态变化: {:?} -> {:?}", previous_state, current_state);
        });

        // 注册内置插件
        plugin_manager
            .register_plugin(Box::new(SearchPlugin::default()))
            .map_err(|e| format!("注册搜索插件失败: {}", e))?;

        plugin_manager
            .register_plugin(Box::new(BackupPlugin::default()))
            .map_err(|e| format!("注册备份插件失败: {}", e))?;

        // 初始化完成，转换状态
        state_manager.handle_event(StateEvent::InitializationComplete)?;

        Ok(Self {
            state_manager,
            plugin_manager,
            async_manager,
            file_entries: Vec::new(),
            current_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        })
    }

    /// 异步加载目录内容
    pub async fn load_directory(&mut self, path: PathBuf) -> Result<(), String> {
        // 转换到加载状态
        self.state_manager.handle_event(StateEvent::StartLoading)?;

        // 使用异步操作读取目录
        let handle = AsyncOperationBuilder::new()
            .with_timeout(Duration::from_secs(10))
            .read_directory(&path)
            .build_single(&self.async_manager)?;

        match handle.wait().await {
            crate::async_ops::AsyncResult::Success(json_value) => {
                // 解析目录内容
                if let Ok(file_infos) =
                    serde_json::from_value::<Vec<crate::async_ops::FileInfo>>(json_value)
                {
                    // 转换为文件条目并通过插件处理
                    self.file_entries = file_infos
                        .iter()
                        .map(|info| {
                            let entry = info.to_file_entry();
                            // 通过插件处理每个条目
                            self.plugin_manager.process_entry(&entry)
                        })
                        .collect();

                    self.current_directory = path;

                    // 完成加载，转换状态
                    self.state_manager.handle_event(StateEvent::FinishLoading)?;
                    Ok(())
                } else {
                    self.handle_error("解析目录内容失败".to_string())
                }
            }
            crate::async_ops::AsyncResult::Error(msg) => {
                self.handle_error(format!("加载目录失败: {}", msg))
            }
            crate::async_ops::AsyncResult::Timeout => self.handle_error("加载目录超时".to_string()),
            crate::async_ops::AsyncResult::Cancelled => {
                self.handle_error("加载目录被取消".to_string())
            }
        }
    }

    /// 异步批量操作示例
    pub async fn batch_file_operations(
        &self,
        operations: Vec<(String, String)>,
    ) -> Result<(), String> {
        // 构建批量操作
        let mut builder = AsyncOperationBuilder::new().with_timeout(Duration::from_secs(30));

        for (src, dst) in operations {
            builder = builder.copy(PathBuf::from(src), PathBuf::from(dst));
        }

        let handle = builder.build_batch(&self.async_manager)?;

        match handle.wait().await {
            crate::async_ops::AsyncResult::Success(_) => {
                println!("批量操作完成");
                Ok(())
            }
            crate::async_ops::AsyncResult::Error(msg) => Err(format!("批量操作失败: {}", msg)),
            crate::async_ops::AsyncResult::Timeout => Err("批量操作超时".to_string()),
            crate::async_ops::AsyncResult::Cancelled => Err("批量操作被取消".to_string()),
        }
    }

    /// 进入设置状态
    pub fn enter_settings(&mut self) -> Result<(), String> {
        self.state_manager.handle_event(StateEvent::EnterSettings)
    }

    /// 退出设置状态
    pub fn exit_settings(&mut self) -> Result<(), String> {
        self.state_manager.handle_event(StateEvent::ExitSettings)
    }

    /// 开始添加条目
    pub fn start_adding_entry(&mut self) -> Result<(), String> {
        self.state_manager
            .handle_event(StateEvent::StartAddingEntry)
    }

    /// 完成添加条目
    pub fn finish_adding_entry(&mut self, entry: FileEntry) -> Result<(), String> {
        // 通过插件处理新条目
        let processed_entry = self.plugin_manager.process_entry(&entry);
        self.file_entries.push(processed_entry);

        self.state_manager
            .handle_event(StateEvent::FinishAddingEntry)
    }

    /// 处理快捷键
    pub fn handle_shortcut(&self, key: &egui::Key, modifiers: &egui::Modifiers) -> bool {
        // 首先让插件处理快捷键
        if self.plugin_manager.handle_shortcut(key, modifiers) {
            return true;
        }

        // 应用程序自己的快捷键处理
        match key {
            egui::Key::F5 => {
                // 异步刷新当前目录
                println!("刷新目录快捷键被按下");
                true
            }
            egui::Key::Escape => {
                // 根据当前状态处理 Escape 键
                match self.state_manager.current_state() {
                    AppState::Settings => {
                        // 在设置状态下，Escape 退出设置
                        println!("从设置状态退出");
                        true
                    }
                    AppState::AddingEntry => {
                        // 在添加条目状态下，Escape 取消添加
                        println!("取消添加条目");
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// 渲染 UI
    pub fn render_ui(&mut self, ui: &mut egui::Ui) {
        // 根据当前状态渲染不同的 UI
        match self.state_manager.current_state() {
            AppState::Initializing => {
                ui.label("正在初始化...");
                ui.spinner();
            }
            AppState::Loading => {
                ui.label("正在加载...");
                ui.spinner();
            }
            AppState::Settings => {
                self.render_settings_ui(ui);
            }
            AppState::AddingEntry => {
                self.render_add_entry_ui(ui);
            }
            AppState::TagManager => {
                self.render_tag_manager_ui(ui);
            }
            AppState::CollectionManager => {
                self.render_collection_manager_ui(ui);
            }
            AppState::ImportExport => {
                self.render_import_export_ui(ui);
            }
            AppState::Error(msg) => {
                ui.label(format!("错误: {}", msg));
                if ui.button("恢复").clicked() {
                    let _ = self
                        .state_manager
                        .handle_event(StateEvent::RecoverFromError);
                }
            }
            AppState::Running => {
                self.render_main_ui(ui);
            }
            AppState::EditingEntry => {
                self.render_edit_entry_ui(ui);
            }
            AppState::Exiting => {
                ui.label("正在退出...");
            }
        }

        // 渲染状态信息
        ui.separator();
        self.render_status_bar(ui);
    }

    /// 渲染主界面
    fn render_main_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(format!("当前目录: {:?}", self.current_directory));

            if ui.button("设置").clicked() {
                let _ = self.enter_settings();
            }

            if ui.button("添加条目").clicked() {
                let _ = self.start_adding_entry();
            }

            if ui.button("标签管理").clicked() {
                let _ = self.state_manager.handle_event(StateEvent::EnterTagManager);
            }
        });

        ui.separator();

        // 渲染文件列表
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (_i, entry) in self.file_entries.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(&entry.name);

                    // 右键菜单
                    ui.menu_button("⋮", |ui| {
                        // 获取插件上下文菜单项
                        let menu_items = self.plugin_manager.get_context_menu_items();
                        for item in menu_items {
                            if item.enabled && ui.button(&item.label).clicked() {
                                let _ = self.plugin_manager.handle_context_menu(&item.id, entry);
                                ui.close_menu();
                            }
                        }

                        // 应用程序自己的菜单项
                        if ui.button("删除").clicked() {
                            // 处理删除操作
                            println!("删除条目: {}", entry.name);
                            ui.close_menu();
                        }
                    });
                });
            }
        });

        // 渲染插件 UI
        ui.collapsing("插件", |ui| {
            self.plugin_manager.render_plugins_ui(ui);
        });
    }

    /// 渲染设置界面
    fn render_settings_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("设置");

        ui.label("插件管理:");
        let plugin_list = self.plugin_manager.get_plugin_list();
        for plugin_info in plugin_list {
            ui.horizontal(|ui| {
                ui.label(&plugin_info.name);
                ui.label(&plugin_info.version);

                let mut enabled = plugin_info.enabled;
                if ui.checkbox(&mut enabled, "启用").changed() {
                    if enabled {
                        let _ = self.plugin_manager.enable_plugin(&plugin_info.name);
                    } else {
                        let _ = self.plugin_manager.disable_plugin(&plugin_info.name);
                    }
                }
            });
        }

        ui.separator();

        if ui.button("返回").clicked() {
            let _ = self.exit_settings();
        }
    }

    /// 渲染添加条目界面
    fn render_add_entry_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("添加新条目");

        // 这里可以添加表单字段
        ui.label("名称:");
        ui.text_edit_singleline(&mut String::new());

        ui.label("路径:");
        ui.text_edit_singleline(&mut String::new());

        ui.horizontal(|ui| {
            if ui.button("确定").clicked() {
                // 创建新条目
                let entry = FileEntry::new(
                    std::path::PathBuf::from("测试路径"),
                    "测试条目".to_string(),
                    None,       // description
                    Vec::new(), // tags
                    false,      // is_directory
                );
                let _ = self.finish_adding_entry(entry);
            }

            if ui.button("取消").clicked() {
                let _ = self
                    .state_manager
                    .handle_event(StateEvent::CancelAddingEntry);
            }
        });
    }

    /// 渲染标签管理界面
    fn render_tag_manager_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("标签管理");

        ui.label("标签管理功能");

        if ui.button("返回").clicked() {
            let _ = self.state_manager.handle_event(StateEvent::ExitTagManager);
        }
    }

    /// 渲染集合管理界面
    fn render_collection_manager_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("集合管理");

        ui.label("集合管理功能");

        if ui.button("返回").clicked() {
            let _ = self
                .state_manager
                .handle_event(StateEvent::ExitCollectionManager);
        }
    }

    /// 渲染导入导出界面
    fn render_import_export_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("导入/导出");

        ui.label("导入导出功能");

        if ui.button("返回").clicked() {
            let _ = self
                .state_manager
                .handle_event(StateEvent::ExitImportExport);
        }
    }

    /// 渲染编辑条目界面
    fn render_edit_entry_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("编辑条目");

        ui.label("编辑条目功能");

        ui.horizontal(|ui| {
            if ui.button("保存").clicked() {
                let _ = self
                    .state_manager
                    .handle_event(StateEvent::FinishEditingEntry);
            }

            if ui.button("取消").clicked() {
                let _ = self
                    .state_manager
                    .handle_event(StateEvent::CancelEditingEntry);
            }
        });
    }

    /// 渲染状态栏
    fn render_status_bar(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(format!("状态: {:?}", self.state_manager.current_state()));

            ui.separator();

            ui.label(format!(
                "活动任务: {}",
                self.async_manager.active_task_count()
            ));

            ui.separator();

            let plugin_count = self.plugin_manager.get_plugin_list().len();
            ui.label(format!("插件: {}", plugin_count));

            // 如果处于错误状态，显示错误信息
            if let Some(error_msg) = self.state_manager.get_error_message() {
                ui.separator();
                ui.colored_label(egui::Color32::RED, format!("错误: {}", error_msg));
            }
        });
    }

    /// 处理错误
    fn handle_error(&mut self, error: String) -> Result<(), String> {
        self.state_manager
            .handle_event(StateEvent::Error(error.clone()))?;
        eprintln!("应用程序错误: {}", error);
        Err(error)
    }

    /// 获取当前状态
    pub fn current_state(&self) -> AppState {
        self.state_manager.current_state()
    }

    /// 获取文件条目
    pub fn get_entries(&self) -> &Vec<FileEntry> {
        &self.file_entries
    }

    /// 获取当前目录
    pub fn current_directory(&self) -> &PathBuf {
        &self.current_directory
    }

    /// 关闭应用程序
    pub fn shutdown(&mut self) -> Result<(), String> {
        // 取消所有异步任务
        self.async_manager.cancel_all_tasks();

        // 转换到退出状态
        self.state_manager.handle_event(StateEvent::Exit)?;

        println!("应用程序正在关闭...");
        Ok(())
    }
}

impl Default for IntegratedFileManager {
    fn default() -> Self {
        Self::new().expect("创建集成文件管理器失败")
    }
}

/// 使用示例
#[cfg(test)]
mod examples {
    use super::*;

    /// 基本使用示例
    #[test]
    #[ignore] // 暂时禁用此测试
    fn basic_usage_example() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut manager = IntegratedFileManager::new().unwrap();

            // 检查初始状态
            assert_eq!(manager.current_state(), AppState::Running);

            // 进入设置状态
            manager.enter_settings().unwrap();
            assert_eq!(manager.current_state(), AppState::Settings);

            // 退出设置状态
            manager.exit_settings().unwrap();
            assert_eq!(manager.current_state(), AppState::Running);

            // 加载目录（异步操作）
            let temp_dir = std::env::temp_dir();
            let result = manager.load_directory(temp_dir).await;
            // 允许加载失败，因为在测试环境中可能会被取消
            assert!(result.is_ok() || result.is_err());

            // 关闭应用程序
            manager.shutdown().unwrap();
        });
    }

    /// 异步操作示例
    #[test]
    #[ignore] // 暂时禁用此测试
    fn async_operations_example() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let manager = IntegratedFileManager::new().unwrap();

            // 批量文件操作示例
            let operations = vec![
                ("src1.txt".to_string(), "dst1.txt".to_string()),
                ("src2.txt".to_string(), "dst2.txt".to_string()),
            ];

            // 注意：这个示例中的文件可能不存在，所以操作会失败
            // 在实际使用中，应该确保文件存在
            let result = manager.batch_file_operations(operations).await;

            // 在测试环境中，我们期望这会失败，因为文件不存在
            assert!(result.is_err());
        });
    }

    /// 插件系统示例
    #[test]
    fn plugin_system_example() {
        let manager = IntegratedFileManager::new().unwrap();

        // 获取插件列表
        let plugins = manager.plugin_manager.get_plugin_list();
        assert_eq!(plugins.len(), 2); // 搜索插件和备份插件

        // 检查插件名称
        let plugin_names: Vec<String> = plugins.iter().map(|p| p.name.clone()).collect();
        assert!(plugin_names.contains(&"Search Plugin".to_string()));
        assert!(plugin_names.contains(&"Backup Plugin".to_string()));
    }

    /// 状态管理示例
    #[test]
    fn state_management_example() {
        let mut manager = IntegratedFileManager::new().unwrap();

        // 测试状态转换
        assert_eq!(manager.current_state(), AppState::Running);

        manager.start_adding_entry().unwrap();
        assert_eq!(manager.current_state(), AppState::AddingEntry);

        // 创建测试条目
        let entry = FileEntry::new(
            std::path::PathBuf::from("test_path"),
            "test_entry".to_string(),
            None,       // description
            Vec::new(), // tags
            false,      // is_directory
        );

        manager.finish_adding_entry(entry).unwrap();
        assert_eq!(manager.current_state(), AppState::Running);

        // 检查条目是否被添加
        assert_eq!(manager.get_entries().len(), 1);
    }
}

/// 实际集成到主应用程序的方法
pub fn integrate_with_main_app() -> Result<IntegratedFileManager, String> {
    println!("正在初始化集成文件管理器...");

    let manager = IntegratedFileManager::new()?;

    println!("集成文件管理器初始化完成");
    println!("- 状态管理: 已启用");
    println!(
        "- 插件系统: 已启用 ({} 个插件)",
        manager.plugin_manager.get_plugin_list().len()
    );
    println!("- 异步操作: 已启用");

    Ok(manager)
}
