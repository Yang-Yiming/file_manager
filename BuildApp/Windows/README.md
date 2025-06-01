# Windows 构建脚本说明

## 📋 脚本概览

| 脚本 | 用途 | 速度 | 推荐场景 |
|------|------|------|----------|
| `build.bat` | 完整构建 | ⚡⚡ | 首次构建、正式发布 |
| `update.bat` | 智能更新 | ⚡⚡⚡ | 日常开发、版本管理 |

## 🚀 快速使用

### 日常开发
```cmd
REM 快速重新编译
update.bat --quick

REM 或简写
update.bat -q
```

### 版本发布
```cmd
REM 更新版本号并构建
update.bat --version 0.3.0

REM 或简写
update.bat -v 0.3.0
```

### 首次构建
```cmd
REM 完整构建（推荐）
build.bat
```

## ⚙️ update.bat 选项

```cmd
-q, --quick         # 快速模式（仅重新编译）
-v, --version VER   # 更新版本号
-b, --no-backup     # 跳过备份
-h, --help          # 显示帮助
```

## 📝 使用示例

```cmd
REM 开发时快速迭代
update.bat -q

REM 版本更新
update.bat -v 1.0.0

REM 跳过备份的快速更新
update.bat -q -b

REM 查看所有选项
update.bat --help
```

## 🔄 典型工作流

1. **修改代码** → `update.bat -q` → **测试**
2. **功能完成** → `update.bat -v 0.x.0` → **发布**
3. **首次部署** → `build.bat` → **分发**

## 📦 输出说明

- 构建成功后生成 `FileManager` 文件夹
- 包含 `file_manager.exe` 主程序
- 包含 `FileManager.bat` 启动脚本
- 自动备份旧版本（除非使用 `-b`）

## 🖥️ 系统要求

- Windows 7 或更高版本
- Rust 1.70+ (https://rustup.rs/)
- Visual Studio Build Tools (Rust 安装时会提示)

## 💡 提示

- 开发期间使用 `update.bat -q` 最快
- 发布前使用 `update.bat -v X.Y.Z`
- 脚本可从任意目录运行
- 自动处理版本号和依赖关系
- 支持便携式部署（整个 FileManager 文件夹可复制使用）

## 🛠️ 故障排除

### Rust 未安装
```cmd
REM 访问 https://rustup.rs/ 下载安装
REM 或使用 winget（Windows 10/11）
winget install Rustlang.Rustup
```

### 编译错误
```cmd
REM 清理并重新构建
cargo clean
build.bat
```

### 权限问题
- 以管理员身份运行命令提示符
- 或将项目放在用户目录下