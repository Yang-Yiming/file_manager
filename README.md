# 文件快速访问器 (File Manager)

一个高性能的文件和文件夹快速访问工具，支持标签管理和搜索功能。

## 功能特性

- 📁 快速添加和管理文件/文件夹路径
- 🏷️ 为每个路径添加自定义标签
- 🔍 实时搜索和过滤
- 🖱️ 拖拽文件直接添加
- ⚡ 超高性能设计
- 🌏 可选中文字体支持

## 安装和运行

```bash
# 克隆项目
git clone <repository-url>
cd file_manager

# 构建运行 (调试模式)
cargo run

# 构建发布版本 (最高性能)
cargo build --release
./target/release/file_manager
```

## 使用说明

### 基本操作

1. **添加路径**: 在输入框中输入文件或文件夹路径，添加标签（可选），点击"添加/更新"
2. **拖拽添加**: 直接将文件或文件夹拖拽到应用窗口中
3. **搜索**: 在搜索框中输入关键词，实时过滤结果
4. **编辑**: 点击 📝 按钮编辑现有条目
5. **删除**: 点击 🗑️ 按钮删除条目
6. **打开**: 点击路径链接直接打开文件或文件夹

### 中文字体支持

默认情况下，应用以最高性能运行，但中文字符可能显示为小方框。

**启用中文字体**:
1. 勾选界面上的 "🔧 中文字体支持" 选项
2. 重启应用使设置生效

**注意**: 启用中文字体会轻微影响启动性能，但运行时性能影响极小。

### 系统兼容性

应用会自动检测并使用系统中文字体：

- **Windows**: 微软雅黑、黑体、宋体
- **macOS**: PingFang、STHeiti Light
- **Linux**: Noto Sans CJK、文泉驿微米黑

## 配置文件

配置自动保存在：
- **Windows**: `%APPDATA%\file_manager_config.json`
- **macOS**: `~/Library/Application Support/file_manager_config.json`
- **Linux**: `~/.config/file_manager_config.json`

配置文件格式：
```json
{
  "enable_chinese_font": false,
  "entries": [
    {
      "path": "/path/to/file",
      "tags": ["tag1", "tag2"],
      "is_directory": false,
      "added_time": "2024-01-01T00:00:00Z"
    }
  ]
}
```

## 性能优化

- 默认关闭中文字体以确保最高性能
- 使用高效的搜索算法
- 最小化内存占用
- 原生 Rust 实现，无 GC 开销

## 技术栈

- **Rust** - 高性能系统编程语言
- **egui** - 即时模式 GUI 框架
- **eframe** - 跨平台应用框架
- **serde** - 序列化/反序列化
- **chrono** - 时间处理

## 许可证

本项目基于 MIT 许可证开源。