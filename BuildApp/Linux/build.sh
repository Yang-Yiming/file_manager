#!/bin/bash

# 文件管理器 Linux 应用构建脚本
# 简洁高效的构建工具

set -e

# 确定项目根目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
log_success() { echo -e "${GREEN}✅ $1${NC}"; }
log_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
log_error() { echo -e "${RED}❌ $1${NC}"; }

# 检查环境
check_environment() {
    if [ ! -f "Cargo.toml" ]; then
        log_error "无法找到项目根目录（Cargo.toml 不存在）"
        log_error "当前目录: $(pwd)"
        exit 1
    fi
    
    if ! command -v cargo >/dev/null 2>&1; then
        log_error "未找到 Rust/Cargo，请先安装"
        log_error "运行: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
}

# 获取版本号
get_version() {
    grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/' | tr -d '"'
}

# 检查依赖
check_dependencies() {
    log_info "检查系统依赖..."
    
    # 检查必要的开发包
    missing_deps=()
    
    if ! pkg-config --exists gtk+-3.0 2>/dev/null; then
        missing_deps+=("libgtk-3-dev")
    fi
    
    if ! pkg-config --exists x11 2>/dev/null; then
        missing_deps+=("libx11-dev")
    fi
    
    if [ ${#missing_deps[@]} -gt 0 ]; then
        log_warning "缺少以下依赖包："
        for dep in "${missing_deps[@]}"; do
            echo "  - $dep"
        done
        echo ""
        log_info "请运行以下命令安装："
        echo "  Ubuntu/Debian: sudo apt-get install ${missing_deps[*]}"
        echo "  Fedora/RHEL:   sudo dnf install $(echo ${missing_deps[*]} | sed 's/libgtk-3-dev/gtk3-devel/g' | sed 's/libx11-dev/libX11-devel/g')"
        echo "  Arch Linux:    sudo pacman -S $(echo ${missing_deps[*]} | sed 's/lib\([^-]*\)-dev/\1/g' | sed 's/gtk+-3/gtk3/g')"
        echo ""
        read -p "是否继续构建？(y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
}

# 构建应用程序
build_app() {
    local version=$(get_version)
    log_info "构建文件管理器 v$version..."
    
    # 清理并构建
    log_info "编译 Release 版本..."
    cargo build --release --quiet
    
    # 创建应用程序目录
    log_info "创建应用程序包..."
    rm -rf FileManager
    mkdir -p FileManager/{bin,share/{applications,icons,doc}}
    
    # 复制可执行文件
    cp target/release/file_manager FileManager/bin/
    chmod +x FileManager/bin/file_manager
    
    # 创建桌面文件
    create_desktop_file "$version"
    
    # 复制图标（如果存在）
    if [ -f "res/icon.png" ]; then
        cp res/icon.png FileManager/share/icons/file-manager.png
    elif [ -f "res/icon.svg" ]; then
        cp res/icon.svg FileManager/share/icons/file-manager.svg
    fi
    
    # 创建安装脚本
    create_install_script "$version"
    
    # 创建启动脚本
    create_launcher_script
    
    # 创建说明文件
    create_readme "$version"
}

# 创建桌面文件
create_desktop_file() {
    local version=$1
    cat > FileManager/share/applications/file-manager.desktop << EOF
[Desktop Entry]
Name=文件快速访问器
Name[en]=File Manager
Comment=简洁高效的文件快速访问管理器
Comment[en]=A simple and efficient file quick access manager
Exec=file_manager
Icon=file-manager
Type=Application
Categories=Utility;FileManager;
StartupNotify=true
Version=$version
EOF
}

# 创建安装脚本
create_install_script() {
    local version=$1
    cat > FileManager/install.sh << 'EOF'
#!/bin/bash

# 文件管理器安装脚本

set -e

INSTALL_DIR="/opt/file-manager"
BIN_DIR="/usr/local/bin"
DESKTOP_DIR="/usr/share/applications"
ICON_DIR="/usr/share/icons"

# 检查权限
if [ "$EUID" -ne 0 ]; then
    echo "请使用 sudo 运行此脚本"
    exit 1
fi

echo "🚀 安装文件管理器..."

# 创建安装目录
mkdir -p "$INSTALL_DIR"
mkdir -p "$DESKTOP_DIR"
mkdir -p "$ICON_DIR"

# 复制文件
cp bin/file_manager "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/file_manager"

# 创建符号链接
ln -sf "$INSTALL_DIR/file_manager" "$BIN_DIR/file_manager"

# 安装桌面文件
if [ -f "share/applications/file-manager.desktop" ]; then
    cp share/applications/file-manager.desktop "$DESKTOP_DIR/"
    # 更新桌面文件中的可执行文件路径
    sed -i "s|Exec=file_manager|Exec=$BIN_DIR/file_manager|" "$DESKTOP_DIR/file-manager.desktop"
fi

# 安装图标
if [ -f "share/icons/file-manager.png" ]; then
    cp share/icons/file-manager.png "$ICON_DIR/"
elif [ -f "share/icons/file-manager.svg" ]; then
    cp share/icons/file-manager.svg "$ICON_DIR/"
fi

# 更新桌面数据库
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
fi

echo "✅ 安装完成！"
echo ""
echo "使用方法："
echo "  • 命令行运行: file_manager"
echo "  • 或在应用菜单中找到 '文件快速访问器'"
echo ""
echo "卸载方法："
echo "  sudo rm -f $BIN_DIR/file_manager"
echo "  sudo rm -f $DESKTOP_DIR/file-manager.desktop"
echo "  sudo rm -f $ICON_DIR/file-manager.png"
echo "  sudo rm -f $ICON_DIR/file-manager.svg"
echo "  sudo rm -rf $INSTALL_DIR"
EOF
    chmod +x FileManager/install.sh
}

# 创建启动脚本
create_launcher_script() {
    cat > FileManager/file_manager.sh << 'EOF'
#!/bin/bash

# 文件管理器启动脚本

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 运行程序
exec "$SCRIPT_DIR/bin/file_manager" "$@"
EOF
    chmod +x FileManager/file_manager.sh
}

# 创建说明文件
create_readme() {
    local version=$1
    cat > FileManager/README.txt << EOF
文件快速访问管理器 v$version (Linux)
================================

这是一个简洁高效的文件快速访问管理器，支持智能标签管理和数据管理。

🚀 运行方法:

1. 直接运行 (便携模式):
   ./file_manager.sh

2. 系统安装:
   sudo ./install.sh
   然后可以在任意位置运行: file_manager

🔧 系统要求:

- Linux (X11)
- GTK+ 3.0 (通常已预装)

📦 包含文件:

- bin/file_manager          主程序
- file_manager.sh          启动脚本
- install.sh               系统安装脚本
- share/applications/      桌面集成文件
- share/icons/             应用图标
- README.txt               本说明文件

💾 数据存储:

用户数据保存在: ~/.local/share/file_manager/

🛠️ 故障排除:

如果程序无法启动，请检查是否安装了必要的依赖：

Ubuntu/Debian:
sudo apt-get install libgtk-3-0 libx11-6

Fedora/RHEL:
sudo dnf install gtk3 libX11

Arch Linux:
sudo pacman -S gtk3 libx11

📄 许可证: MIT License
🌐 项目主页: https://github.com/user/file_manager
EOF
}

# 验证构建结果
verify_build() {
    if [ ! -x "FileManager/bin/file_manager" ]; then
        log_error "构建失败：可执行文件不存在"
        exit 1
    fi
    
    local version=$(get_version)
    local size=$(du -sh FileManager | cut -f1)
    
    log_success "构建完成！"
    echo "  版本: $version"
    echo "  大小: $size"
    echo "  位置: $(pwd)/FileManager"
    echo ""
    log_info "使用方法："
    echo "  • 便携运行: ./FileManager/file_manager.sh"
    echo "  • 系统安装: sudo ./FileManager/install.sh"
    echo "  • 查看说明: cat FileManager/README.txt"
}

# 主函数
main() {
    echo "🚀 文件管理器构建工具 (Linux)"
    echo "=============================="
    
    check_environment
    check_dependencies
    build_app
    verify_build
    
    log_success "全部完成！"
}

# 处理中断信号
trap 'log_error "构建被中断"; exit 1' INT TERM

# 运行主函数
main "$@"