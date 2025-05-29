# 应用程序更新指南

## 📋 概述

本指南介绍如何更新和维护文件管理器应用程序，包括代码修改后的重新构建、版本管理和分发流程。

## 🔄 更新工作流程

### 快速更新（推荐）

当你修改了代码并想要重新打包应用程序时：

```bash
# 简单重新构建
./update_app.sh

# 快速模式（跳过交互）
./update_app.sh --quick

# 更新版本号并构建
./update_app.sh --version 0.3.0
```

### 手动更新步骤

1. **修改代码**
   - 编辑源代码文件
   - 测试功能是否正常

2. **更新版本号**（可选）
   ```bash
   # 编辑 Cargo.toml 中的版本号
   version = "0.3.0"
   ```

3. **重新构建**
   ```bash
   ./build_app.sh  # 完整构建
   # 或者
   ./build.sh      # 简单构建
   ```

## 🏷️ 版本管理

### 版本号规则

采用语义化版本控制（Semantic Versioning）：

- **主版本号**：不兼容的API修改
- **次版本号**：向后兼容的功能性新增
- **修订号**：向后兼容的问题修正

示例：`1.2.3`

### 版本更新命令

```bash
# 修订版本（bug 修复）
./update_app.sh --version 0.2.1

# 次版本（新功能）
./update_app.sh --version 0.3.0

# 主版本（重大更改）
./update_app.sh --version 1.0.0
```

## 📦 构建选项

### 构建脚本对比

| 脚本 | 特点 | 适用场景 |
|------|------|----------|
| `build.sh` | 简单快速 | 日常开发测试 |
| `build_app.sh` | 功能完整 | 正式发布 |
| `update_app.sh` | 智能更新 | 维护更新 |

### 构建参数

```bash
# update_app.sh 参数
--version VERSION    # 更新版本号
--no-backup         # 跳过备份
--quick             # 快速模式
--help              # 显示帮助
```

## 🔧 开发工作流程

### 日常开发

```bash
# 1. 修改代码
vim src/main.rs

# 2. 快速测试
cargo run

# 3. 构建应用程序
./update_app.sh --quick
```

### 发布准备

```bash
# 1. 确保代码质量
cargo test
cargo clippy

# 2. 更新版本并构建
./update_app.sh --version 1.0.0

# 3. 测试应用程序
open FileManager.app

# 4. 提交到版本控制
git add .
git commit -m "Release v1.0.0"
git tag v1.0.0
```

## 📁 文件结构说明

```
file_manager/
├── src/                    # 源代码
├── res/                    # 资源文件
├── FileManager.app/        # 构建的应用程序包
├── build.sh               # 简单构建脚本
├── build_app.sh           # 完整构建脚本
├── update_app.sh          # 更新脚本
└── UPDATE_GUIDE.md        # 本指南
```

## 🚀 部署选项

### 本地使用

```bash
# 直接运行
open FileManager.app

# 或双击 FileManager.app
```

### 系统安装

```bash
# 复制到 Applications 文件夹
cp -r FileManager.app /Applications/

# 或使用更新脚本的交互选项
./update_app.sh
# 选择选项 2：复制到 Applications 文件夹
```

### 创建桌面快捷方式

```bash
# 创建符号链接到桌面
ln -sf "$(pwd)/FileManager.app" ~/Desktop/
```

## 🐛 故障排除

### 常见问题

**Q: 应用程序无法启动**
```bash
# 检查可执行权限
chmod +x FileManager.app/Contents/MacOS/file_manager

# 重新构建
./update_app.sh --no-backup
```

**Q: 版本号未更新**
```bash
# 检查 Cargo.toml 版本号
grep version Cargo.toml

# 强制更新版本
./update_app.sh --version NEW_VERSION
```

**Q: 构建失败**
```bash
# 清理并重新构建
cargo clean
./build_app.sh
```

### 日志和调试

```bash
# 查看构建详细信息
RUST_LOG=debug cargo build --release

# 运行调试版本
cargo run
```

## 📊 性能优化

### 减小应用程序大小

1. **启用链接时优化**（已配置）
   ```toml
   [profile.release]
   lto = "thin"
   strip = true
   ```

2. **移除调试信息**（已配置）
   ```toml
   [profile.release]
   debug = false
   ```

### 加快构建速度

```bash
# 使用增量编译（开发时）
export CARGO_INCREMENTAL=1

# 并行构建
cargo build --release -j $(nproc)
```

## 🔐 安全最佳实践

1. **代码签名**（macOS）
   ```bash
   # 如果有开发者证书
   codesign --sign "Developer ID Application: Your Name" FileManager.app
   ```

2. **公证**（macOS分发）
   ```bash
   # 提交公证
   xcrun notarytool submit FileManager.app --keychain-profile "notarytool-password"
   ```

## 📈 版本历史示例

| 版本 | 日期 | 更改内容 |
|------|------|----------|
| 0.1.0 | 2024-01-01 | 初始版本 |
| 0.2.0 | 2024-02-01 | 添加文件搜索功能 |
| 0.2.1 | 2024-02-15 | 修复搜索Bug |
| 0.3.0 | 2024-03-01 | 添加主题支持 |

## 🆘 获取帮助

- 查看构建脚本帮助：`./update_app.sh --help`
- 检查Rust文档：`cargo doc --open`
- 查看项目依赖：`cargo tree`

---

**提示**：建议在每次重要更新后创建备份，更新脚本会自动处理这个过程。