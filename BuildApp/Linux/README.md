# Linux 构建脚本说明

## 📋 脚本概览

| 脚本 | 用途 | 速度 | 推荐场景 |
|------|------|------|----------|
| `build.sh` | 完整构建 | ⚡⚡ | 首次构建、正式发布 |
| `update.sh` | 智能更新 | ⚡⚡⚡ | 日常开发、版本管理 |

## 🚀 快速使用

### 日常开发
```bash
# 快速重新编译
./update.sh --quick

# 或简写
./update.sh -q
```

### 版本发布
```bash
# 更新版本号并构建
./update.sh --version 0.3.0

# 或简写
./update.sh -v 0.3.0
```

### 首次构建
```bash
# 完整构建（推荐）
./build.sh
```

## ⚙️ update.sh 选项

```bash
-q, --quick         # 快速模式（仅重新编译）
-v, --version VER   # 更新版本号
-b, --no-backup     # 跳过备份
-h, --help          # 显示帮助
```

## 📝 使用示例

```bash
# 开发时快速迭代
./update.sh -q

# 版本更新
./update.sh -v 1.0.0

# 跳过备份的快速更新
./update.sh -q -b

# 查看所有选项
./update.sh --help
```

## 🔄 典型工作流

1. **修改代码** → `./update.sh -q` → **测试**
2. **功能完成** → `./update.sh -v 0.x.0` → **发布**
3. **首次部署** → `./build.sh` → **分发**

## 📦 输出说明

- 构建成功后生成 `FileManager` 文件夹
- 包含 `bin/file_manager` 主程序
- 包含 `file_manager.sh` 启动脚本
- 包含 `install.sh` 系统安装脚本
- 包含桌面集成文件和图标
- 自动备份旧版本（除非使用 `-b`）

## 🖥️ 系统要求

- Linux (X11 支持)
- Rust 1.70+ (https://rustup.rs/)
- GTK+ 3.0 开发包
- X11 开发包

### 安装依赖

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install build-essential libgtk-3-dev libx11-dev
```

**Fedora/RHEL:**
```bash
sudo dnf install gcc-c++ gtk3-devel libX11-devel
```

**Arch Linux:**
```bash
sudo pacman -S base-devel gtk3 libx11
```

**openSUSE:**
```bash
sudo zypper install gcc-c++ gtk3-devel libX11-devel
```

## 💡 提示

- 开发期间使用 `./update.sh -q` 最快
- 发布前使用 `./update.sh -v X.Y.Z`
- 脚本会自动检查系统依赖
- 支持便携式和系统安装两种模式
- 自动生成桌面集成文件

## 🛠️ 故障排除

### Rust 未安装
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 缺少系统依赖
```bash
# 脚本会自动检测并提示安装命令
# 按照提示安装对应发行版的开发包
```

### 编译错误
```bash
# 清理并重新构建
cargo clean
./build.sh
```

### 权限问题
```bash
# 确保脚本有执行权限
chmod +x *.sh

# 系统安装需要 sudo
sudo ./FileManager/install.sh
```

## 📁 文件结构

构建完成后的 `FileManager` 目录结构：
```
FileManager/
├── bin/
│   └── file_manager              # 主程序
├── share/
│   ├── applications/
│   │   └── file-manager.desktop  # 桌面文件
│   └── icons/
│       └── file-manager.png      # 应用图标
├── file_manager.sh               # 启动脚本
├── install.sh                    # 系统安装脚本
└── README.txt                    # 使用说明
```

## 🚀 使用方法

### 便携模式
```bash
# 直接运行（推荐）
./FileManager/file_manager.sh

# 或直接运行二进制文件
./FileManager/bin/file_manager
```

### 系统安装
```bash
# 安装到系统（需要 sudo）
sudo ./FileManager/install.sh

# 安装后可在任意位置运行
file_manager

# 或在应用菜单中找到 "文件快速访问器"
```