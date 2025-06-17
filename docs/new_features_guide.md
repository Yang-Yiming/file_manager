# 文件管理器新功能使用指南

本文档介绍了文件管理器中新增的三个核心功能：状态管理、插件系统和异步操作。这些功能使应用程序更加规范化、可扩展和高效。

## 目录

1. [状态管理系统](#状态管理系统)
2. [插件系统](#插件系统)
3. [异步操作](#异步操作)
4. [集成使用示例](#集成使用示例)
5. [最佳实践](#最佳实践)

## 状态管理系统

### 概述

状态管理系统采用状态机模式，提供了清晰的应用程序状态流转机制。

### 主要特性

- **状态机模式**: 明确定义状态和转换规则
- **状态历史**: 跟踪状态变化历史
- **状态监听器**: 响应状态变化事件
- **线程安全**: 支持多线程环境

### 支持的状态

```rust
pub enum AppState {
    Initializing,        // 初始化状态
    Running,            // 正常运行状态
    Loading,            // 加载中状态
    Settings,           // 设置界面状态
    AddingEntry,        // 添加条目状态
    EditingEntry,       // 编辑条目状态
    TagManager,         // 标签管理状态
    CollectionManager,  // 集合管理状态
    ImportExport,       // 导入导出状态
    Error(String),      // 错误状态
    Exiting,            // 退出状态
}
```

### 使用示例

```rust
use crate::state::{StateManager, StateEvent, AppState};

// 创建状态管理器
let mut state_manager = StateManager::new();

// 添加状态监听器
state_manager.add_state_listener(|current_state, previous_state| {
    println!("状态变化: {:?} -> {:?}", previous_state, current_state);
});

// 处理状态事件
state_manager.handle_event(StateEvent::InitializationComplete)?;
state_manager.handle_event(StateEvent::EnterSettings)?;

// 检查当前状态
if state_manager.is_in_state(&AppState::Settings) {
    println!("当前处于设置状态");
}

// 获取错误信息（如果处于错误状态）
if let Some(error_msg) = state_manager.get_error_message() {
    println!("错误: {}", error_msg);
}
```

### 状态转换规则

状态机定义了严格的转换规则，例如：
- `Initializing` → `Running`: 初始化完成
- `Running` → `Settings`: 进入设置
- `Settings` → `Running`: 退出设置
- 任何状态 → `Error`: 发生错误

## 插件系统

### 概述

插件系统提供了灵活的扩展机制，允许添加新功能而不修改核心代码。

### 主要特性

- **插件接口**: 标准化的插件开发接口
- **生命周期管理**: 插件的初始化、运行和关闭
- **配置管理**: 每个插件的独立配置
- **上下文菜单**: 插件可以添加自定义菜单项
- **快捷键处理**: 插件可以响应快捷键
- **UI 渲染**: 插件可以渲染自定义 UI

### 插件接口

```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    fn author(&self) -> &str;
    
    fn initialize(&mut self, context: &mut PluginContext) -> Result<(), String>;
    fn shutdown(&mut self) -> Result<(), String>;
    
    // 可选方法
    fn process_entry(&self, entry: &FileEntry) -> Option<FileEntry>;
    fn render_ui(&self, ui: &mut egui::Ui, context: &PluginContext);
    fn handle_shortcut(&self, key: &egui::Key, modifiers: &egui::Modifiers) -> bool;
    fn context_menu_items(&self) -> Vec<ContextMenuItem>;
    fn handle_context_menu(&self, item_id: &str, entry: &FileEntry) -> Result<(), String>;
}
```

### 创建自定义插件

```rust
use crate::plugins::{Plugin, PluginContext, ContextMenuItem, PluginConfig};

pub struct MyCustomPlugin {
    name: String,
    config: PluginConfig,
}

impl Plugin for MyCustomPlugin {
    fn name(&self) -> &str {
        "My Custom Plugin"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> &str {
        "这是一个自定义插件示例"
    }
    
    fn author(&self) -> &str {
        "Your Name"
    }
    
    fn initialize(&mut self, context: &mut PluginContext) -> Result<(), String> {
        println!("自定义插件已初始化");
        Ok(())
    }
    
    fn shutdown(&mut self) -> Result<(), String> {
        println!("自定义插件已关闭");
        Ok(())
    }
    
    fn render_ui(&self, ui: &mut egui::Ui, context: &PluginContext) {
        ui.label("自定义插件 UI");
        if ui.button("执行操作").clicked() {
            println!("自定义插件操作被执行");
        }
    }
    
    fn context_menu_items(&self) -> Vec<ContextMenuItem> {
        vec![
            ContextMenuItem::new("custom_action", "自定义操作")
                .with_icon("⚡")
                .with_shortcut("Ctrl+Alt+C"),
        ]
    }
}
```

### 插件管理

```rust
use crate::plugins::PluginManager;

// 创建插件管理器
let mut plugin_manager = PluginManager::new(app_data_dir);

// 注册插件
plugin_manager.register_plugin(Box::new(MyCustomPlugin::default()))?;

// 启用/禁用插件
plugin_manager.enable_plugin("My Custom Plugin")?;
plugin_manager.disable_plugin("My Custom Plugin")?;

// 获取插件列表
let plugins = plugin_manager.get_plugin_list();
for plugin in plugins {
    println!("插件: {} v{} - {}", plugin.name, plugin.version, plugin.description);
}

// 处理文件条目
let processed_entry = plugin_manager.process_entry(&file_entry);

// 渲染插件 UI
plugin_manager.render_plugins_ui(ui);
```

## 异步操作

### 概述

异步操作系统基于 Tokio 运行时，提供高效的文件系统操作，避免阻塞主 UI 线程。

### 主要特性

- **非阻塞操作**: 文件系统操作不会阻塞 UI
- **超时控制**: 每个操作都有超时机制
- **任务取消**: 支持取消正在进行的操作
- **批量操作**: 支持批量文件操作
- **结果回调**: 异步获取操作结果

### 支持的操作

```rust
pub enum AsyncOperation {
    PathExists(PathBuf),              // 检查路径是否存在
    GetFileInfo(PathBuf),             // 获取文件信息
    ReadDirectory(PathBuf),           // 读取目录内容
    CreateDirectory(PathBuf),         // 创建目录
    Delete(PathBuf),                  // 删除文件或目录
    Copy(PathBuf, PathBuf),          // 复制文件或目录
    Move(PathBuf, PathBuf),          // 移动文件或目录
    GetFileSize(PathBuf),            // 获取文件大小
    GetModifiedTime(PathBuf),        // 获取文件修改时间
    Batch(Vec<AsyncOperation>),      // 批量操作
}
```

### 使用示例

#### 基本异步操作

```rust
use crate::async_ops::{AsyncOperationManager, AsyncOperation, AsyncOperationBuilder};
use std::time::Duration;

// 创建异步操作管理器
let async_manager = AsyncOperationManager::new()?;

// 单个操作
let handle = async_manager.submit_task(
    AsyncOperation::PathExists(PathBuf::from("/some/path")),
    Some(Duration::from_secs(5))
)?;

match handle.wait().await {
    AsyncResult::Success(result) => {
        println!("操作成功: {:?}", result);
    }
    AsyncResult::Error(msg) => {
        println!("操作失败: {}", msg);
    }
    AsyncResult::Timeout => {
        println!("操作超时");
    }
    AsyncResult::Cancelled => {
        println!("操作被取消");
    }
}
```

#### 使用构建器模式

```rust
// 单个操作
let handle = AsyncOperationBuilder::new()
    .with_timeout(Duration::from_secs(10))
    .check_path_exists("/some/path")
    .build_single(&async_manager)?;

// 批量操作
let handle = AsyncOperationBuilder::new()
    .with_timeout(Duration::from_secs(30))
    .copy("/src1", "/dst1")
    .copy("/src2", "/dst2")
    .create_directory("/new_dir")
    .build_batch(&async_manager)?;
```

#### 便利函数

```rust
use crate::async_ops::convenience;

// 快速检查路径
if convenience::path_exists("/some/path").await {
    println!("路径存在");
}

// 获取文件大小
let size = convenience::get_file_size("/some/file").await?;
println!("文件大小: {} 字节", size);

// 快速读取目录
let entries = convenience::quick_read_dir("/some/directory").await?;
for entry in entries {
    println!("文件: {}", entry);
}
```

## 集成使用示例

### 完整集成示例

```rust
use crate::integration_example::IntegratedFileManager;

// 创建集成文件管理器
let mut manager = IntegratedFileManager::new()?;

// 异步加载目录
manager.load_directory(PathBuf::from("/some/directory")).await?;

// 状态管理
manager.enter_settings()?;
manager.exit_settings()?;

// 添加文件条目
manager.start_adding_entry()?;
let entry = FileEntry::new("path".to_string(), "name".to_string(), EntryType::File);
manager.finish_adding_entry(entry)?;

// 批量文件操作
let operations = vec![
    ("src1.txt".to_string(), "dst1.txt".to_string()),
    ("src2.txt".to_string(), "dst2.txt".to_string()),
];
manager.batch_file_operations(operations).await?;

// 渲染 UI
manager.render_ui(ui);

// 关闭应用程序
manager.shutdown()?;
```

### 在现有应用中集成

如果你有现有的 `FileManagerApp`，可以这样集成新功能：

```rust
// 在 FileManagerApp 中添加字段
pub struct FileManagerApp {
    // 现有字段...
    
    // 新增字段
    state_manager: StateManager,
    plugin_manager: PluginManager,
    async_manager: AsyncOperationManager,
}

impl FileManagerApp {
    pub fn new() -> Self {
        let state_manager = StateManager::new();
        let plugin_manager = PluginManager::new(app_data_dir);
        let async_manager = AsyncOperationManager::new().unwrap();
        
        // 注册插件
        plugin_manager.register_plugin(Box::new(SearchPlugin::default())).unwrap();
        plugin_manager.register_plugin(Box::new(BackupPlugin::default())).unwrap();
        
        Self {
            // 现有字段初始化...
            state_manager,
            plugin_manager,
            async_manager,
        }
    }
}

impl eframe::App for FileManagerApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // 处理快捷键
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key { key, modifiers, pressed: true, .. } = event {
                    if self.plugin_manager.handle_shortcut(key, modifiers) {
                        return;
                    }
                }
            }
        });
        
        // 根据状态渲染不同的 UI
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.state_manager.current_state() {
                AppState::Running => self.render_main_ui(ui),
                AppState::Settings => self.render_settings_ui(ui),
                AppState::Loading => {
                    ui.label("正在加载...");
                    ui.spinner();
                }
                // 其他状态...
                _ => {}
            }
            
            // 渲染插件 UI
            self.plugin_manager.render_plugins_ui(ui);
        });
    }
}
```

## 最佳实践

### 状态管理最佳实践

1. **明确状态职责**: 每个状态应该有明确的职责和行为
2. **合理的状态转换**: 确保状态转换逻辑清晰合理
3. **错误处理**: 及时处理错误状态，提供恢复机制
4. **状态监听**: 使用状态监听器记录日志或执行副作用

### 插件开发最佳实践

1. **单一职责**: 每个插件应该专注于一个特定功能
2. **配置管理**: 提供合理的配置选项
3. **错误处理**: 优雅地处理错误，不影响主应用程序
4. **资源管理**: 在 shutdown 方法中清理资源
5. **用户体验**: 提供清晰的 UI 和有用的快捷键

### 异步操作最佳实践

1. **超时设置**: 为每个操作设置合理的超时时间
2. **错误处理**: 处理所有可能的异步操作结果
3. **进度反馈**: 对于长时间运行的操作，提供进度反馈
4. **资源清理**: 及时取消不需要的操作
5. **批量操作**: 对于多个相关操作，使用批量模式提高效率

### 性能优化建议

1. **避免阻塞**: 所有文件系统操作都应该使用异步接口
2. **合理缓存**: 缓存常用的文件信息
3. **延迟加载**: 只在需要时加载数据
4. **插件延迟初始化**: 只初始化启用的插件
5. **状态最小化**: 保持状态管理器的状态最小化

### 调试和测试

1. **状态日志**: 使用状态监听器记录状态变化
2. **插件测试**: 为每个插件编写单元测试
3. **异步测试**: 使用 `#[tokio::test]` 测试异步操作
4. **集成测试**: 测试各个系统之间的集成

## 常见问题

### Q: 如何添加新的应用状态？

A: 在 `AppState` 枚举中添加新状态，然后在 `StateMachine::setup_transitions()` 中定义相应的转换规则。

### Q: 插件之间如何通信？

A: 插件可以通过 `PluginContext` 的共享数据机制进行通信，或者触发事件让其他插件响应。

### Q: 如何处理异步操作的错误？

A: 异步操作返回 `AsyncResult` 枚举，包含 `Success`、`Error`、`Timeout` 和 `Cancelled` 四种情况，应该针对每种情况进行处理。

### Q: 可以禁用某些插件吗？

A: 可以，使用 `PluginManager::disable_plugin()` 方法禁用插件，使用 `enable_plugin()` 重新启用。

### Q: 异步操作是否线程安全？

A: 是的，`AsyncOperationManager` 使用 `Arc<Mutex<>>` 确保线程安全，可以在多线程环境中使用。

## 总结

这三个新功能大大提升了文件管理器的架构质量：

- **状态管理系统**：提供了清晰的状态流转机制，使应用程序行为更加可预测
- **插件系统**：实现了功能的模块化，支持第三方扩展
- **异步操作**：避免了 UI 阻塞，提供了更好的用户体验

通过合理使用这些功能，可以构建出更加健壮、可扩展和用户友好的文件管理器应用程序。