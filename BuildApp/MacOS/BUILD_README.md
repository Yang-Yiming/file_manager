# 构建和更新指南

## 🚀 快速开始

### 首次构建
```bash
./build_app.sh  # 完整构建（推荐）
# 或
./build.sh      # 简单构建
```

### 日常更新
```bash
./quick_update.sh   # 最快速的更新方式
# 或
./update_app.sh     # 完整的更新流程
```

## 📋 构建脚本说明

| 脚本 | 用途 | 速度 | 功能 |
|------|------|------|------|
| `quick_update.sh` | 日常开发 | ⚡⚡⚡ | 快速重编译和打包 |
| `build.sh` | 简单构建 | ⚡⚡ | 基础应用程序包 |
| `build_app.sh` | 完整构建 | ⚡ | 包含图标、优化等 |
| `update_app.sh` | 智能更新 | ⚡ | 版本管理、备份、安装 |

## 🔄 典型工作流程

### 修改代码后
```bash
# 1. 快速重新构建
./quick_update.sh

# 2. 测试应用程序
open FileManager.app
```

### 准备发布
```bash
# 1. 更新版本号并构建
./update_app.sh --version 0.3.0

# 2. 选择安装到 Applications 文件夹
# 按提示选择选项 2
```

### 快速开发迭代
```bash
# 编辑代码
vim src/main.rs

# 一键更新
./quick_update.sh

# 立即测试
open FileManager.app
```

## 📦 输出说明

构建完成后会生成：
- `FileManager.app` - 可直接双击运行的 macOS 应用程序
- 应用程序包含所有依赖，无需安装 Rust 环境即可运行

## 🎯 使用建议

- **开发期间**：使用 `quick_update.sh`
- **功能完成**：使用 `update_app.sh`
- **首次构建**：使用 `build_app.sh`
- **CI/CD**：使用 `build.sh`

## ⚠️ 注意事项

1. 确保在项目根目录运行脚本
2. 首次运行可能需要下载依赖，时间较长
3. release 模式构建的应用程序不会显示终端窗口
4. 应用程序会自动使用系统主题

## 🔧 故障排除

**应用无法启动**：
```bash
chmod +x FileManager.app/Contents/MacOS/file_manager
```

**构建失败**：
```bash
cargo clean
./build_app.sh
```

**版本号错误**：
```bash
./update_app.sh --version 正确版本号
```