use crate::file_entry::FileEntry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// 插件接口定义
pub trait Plugin: Send + Sync {
    /// 插件名称
    fn name(&self) -> &str;

    /// 插件版本
    fn version(&self) -> &str;

    /// 插件描述
    fn description(&self) -> &str;

    /// 插件作者
    fn author(&self) -> &str;

    /// 插件初始化
    fn initialize(&mut self, context: &mut PluginContext) -> Result<(), String>;

    /// 插件销毁
    fn shutdown(&mut self) -> Result<(), String>;

    /// 处理文件条目（可选）
    fn process_entry(&self, _entry: &FileEntry) -> Option<FileEntry> {
        None
    }

    /// 自定义UI渲染（可选）
    fn render_ui(&self, _ui: &mut egui::Ui, _context: &PluginContext) {}

    /// 处理快捷键（可选）
    fn handle_shortcut(&self, _key: &egui::Key, _modifiers: &egui::Modifiers) -> bool {
        false
    }

    /// 提供上下文菜单项（可选）
    fn context_menu_items(&self) -> Vec<ContextMenuItem> {
        Vec::new()
    }

    /// 处理上下文菜单点击
    fn handle_context_menu(&self, _item_id: &str, _entry: &FileEntry) -> Result<(), String> {
        Ok(())
    }

    /// 插件配置
    fn get_config(&self) -> Option<PluginConfig> {
        None
    }

    /// 设置插件配置
    fn set_config(&mut self, _config: PluginConfig) -> Result<(), String> {
        Ok(())
    }
}

/// 插件上下文 - 提供插件与主应用程序交互的接口
#[derive(Clone)]
pub struct PluginContext {
    /// 应用程序数据目录
    pub app_data_dir: PathBuf,
    /// 插件数据目录
    pub plugin_data_dir: PathBuf,
    /// 共享数据
    pub shared_data: HashMap<String, String>,
    /// 事件回调
    callbacks: HashMap<String, Arc<dyn Fn(&str) + Send + Sync>>,
}

impl std::fmt::Debug for PluginContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginContext")
            .field("app_data_dir", &self.app_data_dir)
            .field("plugin_data_dir", &self.plugin_data_dir)
            .field("shared_data", &self.shared_data)
            .field("callbacks", &format!("{} callbacks", self.callbacks.len()))
            .finish()
    }
}

impl PluginContext {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let plugin_data_dir = app_data_dir.join("plugins");
        Self {
            app_data_dir,
            plugin_data_dir,
            shared_data: HashMap::new(),
            callbacks: HashMap::new(),
        }
    }

    /// 设置共享数据
    pub fn set_shared_data(&mut self, key: &str, value: &str) {
        self.shared_data.insert(key.to_string(), value.to_string());
    }

    /// 获取共享数据
    pub fn get_shared_data(&self, key: &str) -> Option<&String> {
        self.shared_data.get(key)
    }

    /// 注册事件回调
    pub fn register_callback<F>(&mut self, event: &str, callback: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.callbacks.insert(event.to_string(), Arc::new(callback));
    }

    /// 触发事件
    pub fn trigger_event(&self, event: &str, data: &str) {
        if let Some(callback) = self.callbacks.get(event) {
            callback(data);
        }
    }

    /// 获取插件数据目录
    pub fn get_plugin_data_dir(&self, plugin_name: &str) -> PathBuf {
        self.plugin_data_dir.join(plugin_name)
    }
}

/// 上下文菜单项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMenuItem {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub shortcut: Option<String>,
    pub enabled: bool,
}

impl ContextMenuItem {
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            icon: None,
            shortcut: None,
            enabled: true,
        }
    }

    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    pub fn with_shortcut(mut self, shortcut: &str) -> Self {
        self.shortcut = Some(shortcut.to_string());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// 插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub settings: HashMap<String, serde_json::Value>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            settings: HashMap::new(),
        }
    }
}

impl PluginConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置配置项
    pub fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), String> {
        let json_value =
            serde_json::to_value(value).map_err(|e| format!("序列化配置失败: {}", e))?;
        self.settings.insert(key.to_string(), json_value);
        Ok(())
    }

    /// 获取配置项
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>, String> {
        if let Some(value) = self.settings.get(key) {
            let result: T = serde_json::from_value(value.clone())
                .map_err(|e| format!("反序列化配置失败: {}", e))?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    /// 获取配置项或默认值
    pub fn get_or_default<T: for<'de> Deserialize<'de> + Default>(&self, key: &str) -> T {
        self.get(key).unwrap_or_default().unwrap_or_default()
    }
}

/// 插件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub enabled: bool,
    pub loaded: bool,
}

/// 插件管理器
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    plugin_configs: HashMap<String, PluginConfig>,
    context: PluginContext,
    plugin_order: Vec<String>,
}

impl PluginManager {
    pub fn new(app_data_dir: PathBuf) -> Self {
        Self {
            plugins: HashMap::new(),
            plugin_configs: HashMap::new(),
            context: PluginContext::new(app_data_dir),
            plugin_order: Vec::new(),
        }
    }

    /// 注册插件
    pub fn register_plugin(&mut self, mut plugin: Box<dyn Plugin>) -> Result<(), String> {
        let name = plugin.name().to_string();

        // 检查插件是否已存在
        if self.plugins.contains_key(&name) {
            return Err(format!("插件 '{}' 已存在", name));
        }

        // 加载插件配置
        let config = self.plugin_configs.get(&name).cloned().unwrap_or_default();
        plugin.set_config(config.clone())?;

        // 如果插件启用，则初始化
        if config.enabled {
            plugin.initialize(&mut self.context)?;
        }

        // 添加到插件列表
        self.plugins.insert(name.clone(), plugin);
        self.plugin_configs.insert(name.clone(), config);
        self.plugin_order.push(name);

        Ok(())
    }

    /// 卸载插件
    pub fn unregister_plugin(&mut self, name: &str) -> Result<(), String> {
        if let Some(mut plugin) = self.plugins.remove(name) {
            plugin.shutdown()?;
            self.plugin_configs.remove(name);
            self.plugin_order.retain(|n| n != name);
            Ok(())
        } else {
            Err(format!("插件 '{}' 不存在", name))
        }
    }

    /// 启用插件
    pub fn enable_plugin(&mut self, name: &str) -> Result<(), String> {
        if let Some(plugin) = self.plugins.get_mut(name) {
            if let Some(config) = self.plugin_configs.get_mut(name) {
                if !config.enabled {
                    plugin.initialize(&mut self.context)?;
                    config.enabled = true;
                }
                Ok(())
            } else {
                Err(format!("插件 '{}' 配置不存在", name))
            }
        } else {
            Err(format!("插件 '{}' 不存在", name))
        }
    }

    /// 禁用插件
    pub fn disable_plugin(&mut self, name: &str) -> Result<(), String> {
        if let Some(plugin) = self.plugins.get_mut(name) {
            if let Some(config) = self.plugin_configs.get_mut(name) {
                if config.enabled {
                    plugin.shutdown()?;
                    config.enabled = false;
                }
                Ok(())
            } else {
                Err(format!("插件 '{}' 配置不存在", name))
            }
        } else {
            Err(format!("插件 '{}' 不存在", name))
        }
    }

    /// 获取插件列表
    pub fn get_plugin_list(&self) -> Vec<PluginInfo> {
        self.plugins
            .iter()
            .map(|(name, plugin)| {
                let config = self.plugin_configs.get(name).cloned().unwrap_or_default();
                PluginInfo {
                    name: plugin.name().to_string(),
                    version: plugin.version().to_string(),
                    description: plugin.description().to_string(),
                    author: plugin.author().to_string(),
                    enabled: config.enabled,
                    loaded: true,
                }
            })
            .collect()
    }

    /// 处理文件条目
    pub fn process_entry(&self, entry: &FileEntry) -> FileEntry {
        let mut processed_entry = entry.clone();

        for name in &self.plugin_order {
            if let Some(plugin) = self.plugins.get(name) {
                let default_config = PluginConfig::default();
                let config = self.plugin_configs.get(name).unwrap_or(&default_config);
                if config.enabled {
                    if let Some(new_entry) = plugin.process_entry(&processed_entry) {
                        processed_entry = new_entry;
                    }
                }
            }
        }

        processed_entry
    }

    /// 渲染插件UI
    pub fn render_plugins_ui(&self, ui: &mut egui::Ui) {
        for name in &self.plugin_order {
            if let Some(plugin) = self.plugins.get(name) {
                let default_config = PluginConfig::default();
                let config = self.plugin_configs.get(name).unwrap_or(&default_config);
                if config.enabled {
                    ui.group(|ui| {
                        ui.label(format!("插件: {}", plugin.name()));
                        plugin.render_ui(ui, &self.context);
                    });
                }
            }
        }
    }

    /// 处理快捷键
    pub fn handle_shortcut(&self, key: &egui::Key, modifiers: &egui::Modifiers) -> bool {
        for name in &self.plugin_order {
            if let Some(plugin) = self.plugins.get(name) {
                let default_config = PluginConfig::default();
                let config = self.plugin_configs.get(name).unwrap_or(&default_config);
                if config.enabled && plugin.handle_shortcut(key, modifiers) {
                    return true;
                }
            }
        }
        false
    }

    /// 获取所有上下文菜单项
    pub fn get_context_menu_items(&self) -> Vec<ContextMenuItem> {
        let mut items = Vec::new();

        for name in &self.plugin_order {
            if let Some(plugin) = self.plugins.get(name) {
                let default_config = PluginConfig::default();
                let config = self.plugin_configs.get(name).unwrap_or(&default_config);
                if config.enabled {
                    items.extend(plugin.context_menu_items());
                }
            }
        }

        items
    }

    /// 处理上下文菜单点击
    pub fn handle_context_menu(&self, item_id: &str, entry: &FileEntry) -> Result<(), String> {
        for name in &self.plugin_order {
            if let Some(plugin) = self.plugins.get(name) {
                let default_config = PluginConfig::default();
                let config = self.plugin_configs.get(name).unwrap_or(&default_config);
                if config.enabled {
                    if let Ok(_) = plugin.handle_context_menu(item_id, entry) {
                        return Ok(());
                    }
                }
            }
        }
        Err(format!("未找到处理上下文菜单项 '{}' 的插件", item_id))
    }

    /// 获取插件配置
    pub fn get_plugin_config(&self, name: &str) -> Option<&PluginConfig> {
        self.plugin_configs.get(name)
    }

    /// 设置插件配置
    pub fn set_plugin_config(&mut self, name: &str, config: PluginConfig) -> Result<(), String> {
        if let Some(plugin) = self.plugins.get_mut(name) {
            plugin.set_config(config.clone())?;
            self.plugin_configs.insert(name.to_string(), config);
            Ok(())
        } else {
            Err(format!("插件 '{}' 不存在", name))
        }
    }

    /// 保存所有插件配置
    pub fn save_configs(&self, config_dir: &PathBuf) -> Result<(), String> {
        let plugin_config_file = config_dir.join("plugins.json");

        let json = serde_json::to_string_pretty(&self.plugin_configs)
            .map_err(|e| format!("序列化插件配置失败: {}", e))?;

        std::fs::write(&plugin_config_file, json)
            .map_err(|e| format!("保存插件配置失败: {}", e))?;

        Ok(())
    }

    /// 加载所有插件配置
    pub fn load_configs(&mut self, config_dir: &PathBuf) -> Result<(), String> {
        let plugin_config_file = config_dir.join("plugins.json");

        if plugin_config_file.exists() {
            let content = std::fs::read_to_string(&plugin_config_file)
                .map_err(|e| format!("读取插件配置失败: {}", e))?;

            self.plugin_configs =
                serde_json::from_str(&content).map_err(|e| format!("解析插件配置失败: {}", e))?;
        }

        Ok(())
    }

    /// 获取上下文
    pub fn get_context(&self) -> &PluginContext {
        &self.context
    }

    /// 获取可变上下文
    pub fn get_context_mut(&mut self) -> &mut PluginContext {
        &mut self.context
    }
}

/// 内置搜索插件示例
pub struct SearchPlugin {
    name: String,
    version: String,
    config: PluginConfig,
}

impl Default for SearchPlugin {
    fn default() -> Self {
        Self {
            name: "Search Plugin".to_string(),
            version: "1.0.0".to_string(),
            config: PluginConfig::default(),
        }
    }
}

impl Plugin for SearchPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn description(&self) -> &str {
        "提供高级搜索功能"
    }

    fn author(&self) -> &str {
        "File Manager Team"
    }

    fn initialize(&mut self, _context: &mut PluginContext) -> Result<(), String> {
        println!("搜索插件已初始化");
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), String> {
        println!("搜索插件已关闭");
        Ok(())
    }

    fn render_ui(&self, ui: &mut egui::Ui, _context: &PluginContext) {
        ui.horizontal(|ui| {
            ui.label("高级搜索:");
            ui.text_edit_singleline(&mut String::new());
        });
    }

    fn context_menu_items(&self) -> Vec<ContextMenuItem> {
        vec![
            ContextMenuItem::new("search_similar", "搜索相似文件")
                .with_icon("🔍")
                .with_shortcut("Ctrl+Shift+F"),
        ]
    }

    fn handle_context_menu(&self, item_id: &str, entry: &FileEntry) -> Result<(), String> {
        match item_id {
            "search_similar" => {
                println!("搜索与 '{}' 相似的文件", entry.name);
                Ok(())
            }
            _ => Err(format!("未知的上下文菜单项: {}", item_id)),
        }
    }

    fn get_config(&self) -> Option<PluginConfig> {
        Some(self.config.clone())
    }

    fn set_config(&mut self, config: PluginConfig) -> Result<(), String> {
        self.config = config;
        Ok(())
    }
}

/// 内置备份插件示例
pub struct BackupPlugin {
    name: String,
    version: String,
    config: PluginConfig,
}

impl Default for BackupPlugin {
    fn default() -> Self {
        Self {
            name: "Backup Plugin".to_string(),
            version: "1.0.0".to_string(),
            config: PluginConfig::default(),
        }
    }
}

impl Plugin for BackupPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn description(&self) -> &str {
        "提供数据备份和恢复功能"
    }

    fn author(&self) -> &str {
        "File Manager Team"
    }

    fn initialize(&mut self, context: &mut PluginContext) -> Result<(), String> {
        // 创建备份目录
        let backup_dir = context.get_plugin_data_dir(self.name());
        std::fs::create_dir_all(&backup_dir).map_err(|e| format!("创建备份目录失败: {}", e))?;

        println!("备份插件已初始化，备份目录: {:?}", backup_dir);
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), String> {
        println!("备份插件已关闭");
        Ok(())
    }

    fn render_ui(&self, ui: &mut egui::Ui, context: &PluginContext) {
        ui.horizontal(|ui| {
            if ui.button("创建备份").clicked() {
                context.trigger_event("backup_requested", "");
            }
            if ui.button("恢复备份").clicked() {
                context.trigger_event("restore_requested", "");
            }
        });
    }

    fn context_menu_items(&self) -> Vec<ContextMenuItem> {
        vec![
            ContextMenuItem::new("backup_entry", "备份此项")
                .with_icon("💾")
                .with_shortcut("Ctrl+B"),
        ]
    }

    fn handle_context_menu(&self, item_id: &str, entry: &FileEntry) -> Result<(), String> {
        match item_id {
            "backup_entry" => {
                println!("备份条目: '{}'", entry.name);
                Ok(())
            }
            _ => Err(format!("未知的上下文菜单项: {}", item_id)),
        }
    }

    fn get_config(&self) -> Option<PluginConfig> {
        Some(self.config.clone())
    }

    fn set_config(&mut self, config: PluginConfig) -> Result<(), String> {
        self.config = config;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_plugin_config() {
        let mut config = PluginConfig::new();

        // 测试设置和获取配置
        config.set("test_key", "test_value").unwrap();
        let value: Option<String> = config.get("test_key").unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        // 测试获取不存在的配置
        let missing: Option<String> = config.get("missing_key").unwrap();
        assert_eq!(missing, None);

        // 测试默认值
        let default_value: String = config.get_or_default("missing_key");
        assert_eq!(default_value, String::default());
    }

    #[test]
    fn test_plugin_manager() {
        let temp_dir = std::env::temp_dir();
        let mut manager = PluginManager::new(temp_dir);

        // 注册插件
        let plugin = Box::new(SearchPlugin::default());
        let result = manager.register_plugin(plugin);
        assert!(result.is_ok());

        // 获取插件列表
        let plugins = manager.get_plugin_list();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "Search Plugin");

        // 禁用插件
        let result = manager.disable_plugin("Search Plugin");
        assert!(result.is_ok());

        // 启用插件
        let result = manager.enable_plugin("Search Plugin");
        assert!(result.is_ok());

        // 卸载插件
        let result = manager.unregister_plugin("Search Plugin");
        assert!(result.is_ok());

        let plugins = manager.get_plugin_list();
        assert_eq!(plugins.len(), 0);
    }

    #[test]
    fn test_context_menu_item() {
        let item = ContextMenuItem::new("test_id", "Test Item")
            .with_icon("🔍")
            .with_shortcut("Ctrl+T")
            .enabled(true);

        assert_eq!(item.id, "test_id");
        assert_eq!(item.label, "Test Item");
        assert_eq!(item.icon, Some("🔍".to_string()));
        assert_eq!(item.shortcut, Some("Ctrl+T".to_string()));
        assert!(item.enabled);
    }

    #[test]
    fn test_plugin_context() {
        let temp_dir = std::env::temp_dir();
        let mut context = PluginContext::new(temp_dir.clone());

        // 测试共享数据
        context.set_shared_data("key1", "value1");
        assert_eq!(context.get_shared_data("key1"), Some(&"value1".to_string()));

        // 测试插件数据目录
        let plugin_dir = context.get_plugin_data_dir("test_plugin");
        assert_eq!(plugin_dir, temp_dir.join("plugins").join("test_plugin"));
    }
}
