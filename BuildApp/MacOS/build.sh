#!/bin/bash

# 文件管理器 macOS 应用构建脚本
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
        exit 1
    fi
}

# 获取版本号
get_version() {
    grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/' | tr -d '"'
}

# 构建应用程序
build_app() {
    local version=$(get_version)
    log_info "构建文件管理器 v$version..."
    
    # 清理并构建
    log_info "编译 Release 版本..."
    cargo build --release --quiet
    
    # 创建应用程序包
    log_info "创建应用程序包..."
    rm -rf FileManager.app
    mkdir -p FileManager.app/Contents/{MacOS,Resources}
    
    # 复制可执行文件
    cp target/release/file_manager FileManager.app/Contents/MacOS/
    chmod +x FileManager.app/Contents/MacOS/file_manager
    
    # 创建 Info.plist
    create_info_plist "$version"
    
    # 复制图标（如果存在）
    if [ -f "res/icon.icns" ]; then
        cp res/icon.icns FileManager.app/Contents/Resources/
    fi
    
    # 设置应用程序属性
    touch FileManager.app
    SetFile -a B FileManager.app 2>/dev/null || true
    find FileManager.app -name ".DS_Store" -delete 2>/dev/null || true
}

# 创建 Info.plist
create_info_plist() {
    local version=$1
    cat > FileManager.app/Contents/Info.plist << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>file_manager</string>
    <key>CFBundleIdentifier</key>
    <string>com.filemanager.app</string>
    <key>CFBundleName</key>
    <string>文件快速访问器</string>
    <key>CFBundleDisplayName</key>
    <string>文件快速访问器</string>
    <key>CFBundleVersion</key>
    <string>$version</string>
    <key>CFBundleShortVersionString</key>
    <string>$version</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.utilities</string>
EOF

    # 添加图标引用（如果图标存在）
    if [ -f "res/icon.icns" ]; then
        cat >> FileManager.app/Contents/Info.plist << EOF
    <key>CFBundleIconFile</key>
    <string>icon.icns</string>
EOF
    fi

    cat >> FileManager.app/Contents/Info.plist << EOF
</dict>
</plist>
EOF
}

# 验证构建结果
verify_build() {
    if [ ! -x "FileManager.app/Contents/MacOS/file_manager" ]; then
        log_error "构建失败：可执行文件不存在"
        exit 1
    fi
    
    local version=$(get_version)
    local size=$(du -sh FileManager.app | cut -f1)
    
    log_success "构建完成！"
    echo "  版本: $version"
    echo "  大小: $size"
    echo "  位置: $(pwd)/FileManager.app"
    echo ""
    log_info "使用方法："
    echo "  • 双击 FileManager.app 运行"
    echo "  • 或拖拽到 Applications 文件夹安装"
}

# 主函数
main() {
    echo "🚀 文件管理器构建工具"
    echo "======================="
    
    check_environment
    build_app
    verify_build
    
    log_success "全部完成！"
}

# 处理中断信号
trap 'log_error "构建被中断"; exit 1' INT TERM

# 运行主函数
main "$@"