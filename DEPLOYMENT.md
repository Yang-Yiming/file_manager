# 文件快速访问器 - 部署指南

## 构建类似原生应用程序的版本

### ✅ 已完成的优化

1. **去除控制台窗口**
   - 添加了 `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` 属性
   - Release 版本不再显示黑色控制台窗口

2. **应用程序优化**
   - 启用了 LTO (Link Time Optimization)
   - 优化了编译设置
   - 添加了应用程序元数据

3. **统一 UI 设计**
   - 采用类似 Zed 编辑器的简洁风格
   - 移除了所有 Emoji，使用纯文字
   - 统一的按钮颜色和样式

4. **主题系统**
   - 支持浅色/深色/跟随系统主题
   - 跨平台主题检测

## 构建和部署

### 开发版本（带控制台调试）
```bash
cargo run
```

### 生产版本（无控制台窗口）
```bash
cargo build --release
```

生成的可执行文件位于：`target/release/file_manager` (macOS/Linux) 或 `target/release/file_manager.exe` (Windows)

### 分发准备

1. **单文件分发**
   - 可执行文件是独立的，可以直接复制到目标机器
   - 配置文件会自动在用户目录下创建

2. **Windows 用户**
   - 可以将 `file_manager.exe` 复制到任意位置
   - 双击即可运行，无需额外安装

3. **macOS 用户**
   - 可以将 `file_manager` 复制到 `/Applications` 或任意位置
   - 可能需要在首次运行时允许运行（系统安全设置）

4. **Linux 用户**
   - 复制到 `/usr/local/bin` 或个人 `bin` 目录
   - 确保有执行权限：`chmod +x file_manager`

## 应用程序图标（可选）

如需自定义图标：

1. 准备图标文件：
   - Windows: 创建 `res/icon.ico` 文件
   - 建议尺寸：16x16, 32x32, 48x48, 256x256 像素

2. 重新构建：
   ```bash
   cargo build --release
   ```

## 配置文件位置

- **Windows**: `%APPDATA%/file_manager/config.json`
- **macOS**: `~/Library/Application Support/file_manager/config.json`
- **Linux**: `~/.config/file_manager/config.json`

## 性能特点

- 启动时间：< 1 秒
- 内存占用：约 20-30 MB
- 无需额外运行时依赖
- 支持高分辨率显示器
- 原生系统主题跟随

## 故障排除

1. **权限问题**：确保对配置目录有写入权限
2. **字体问题**：程序会自动检测系统字体
3. **主题问题**：检查系统主题设置是否正确