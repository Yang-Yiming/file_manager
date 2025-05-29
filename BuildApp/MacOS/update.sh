#!/bin/bash

# 文件管理器智能更新工具
# 支持快速更新、版本管理、备份等功能

set -e

# 确定项目根目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 日志函数
log_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
log_success() { echo -e "${GREEN}✅ $1${NC}"; }
log_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
log_error() { echo -e "${RED}❌ $1${NC}"; }

# 全局变量
QUICK_MODE=false
SKIP_BACKUP=false
NEW_VERSION=""
AUTO_INSTALL=false

# 显示帮助信息
show_help() {
    cat << EOF
文件管理器更新工具

用法: $0 [选项]

选项:
  -q, --quick         快速模式（仅重新编译）
  -v, --version VER   更新版本号
  -b, --no-backup     跳过备份
  -i, --install       自动安装到 Applications
  -h, --help          显示此帮助信息

示例:
  $0                    # 标准更新
  $0 --quick           # 快速更新
  $0 --version 0.3.0   # 更新版本并构建
  $0 -q -i             # 快速更新并安装

EOF
}

# 解析命令行参数
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -q|--quick)
                QUICK_MODE=true
                shift
                ;;
            -v|--version)
                NEW_VERSION="$2"
                shift 2
                ;;
            -b|--no-backup)
                SKIP_BACKUP=true
                shift
                ;;
            -i|--install)
                AUTO_INSTALL=true
                shift
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            *)
                log_error "未知选项: $1"
                show_help
                exit 1
                ;;
        esac
    done
}

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

# 获取当前版本号
get_version() {
    grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/' | tr -d '"'
}

# 更新版本号
update_version() {
    local new_version=$1
    log_info "更新版本号: $(get_version) -> $new_version"
    
    # 更新 Cargo.toml
    sed -i '' "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
    
    log_success "版本号已更新"
}

# 备份当前应用
backup_app() {
    if [ "$SKIP_BACKUP" = true ]; then
        return 0
    fi
    
    if [ -d "FileManager.app" ]; then
        local backup_name="FileManager_backup_$(date +%Y%m%d_%H%M%S).app"
        log_info "备份当前应用: $backup_name"
        cp -r FileManager.app "$backup_name"
        log_success "备份完成"
    fi
}

# 快速更新模式
quick_build() {
    local version=$(get_version)
    log_info "快速更新模式 v$version"
    
    # 编译
    log_info "编译中..."
    cargo build --release --quiet
    
    # 更新应用包
    log_info "更新应用包..."
    rm -rf FileManager.app
    mkdir -p FileManager.app/Contents/MacOS
    
    # 复制可执行文件
    cp target/release/file_manager FileManager.app/Contents/MacOS/
    chmod +x FileManager.app/Contents/MacOS/file_manager
    
    # 创建简化的 Info.plist
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
</dict>
</plist>
EOF

    log_success "快速更新完成！版本: $version"
}

# 完整构建模式
full_build() {
    log_info "执行完整构建..."
    
    if [ -f "BuildApp/MacOS/build.sh" ]; then
        bash BuildApp/MacOS/build.sh
    else
        log_error "未找到构建脚本: BuildApp/MacOS/build.sh"
        exit 1
    fi
}

# 验证构建结果
verify_build() {
    if [ ! -x "FileManager.app/Contents/MacOS/file_manager" ]; then
        log_error "构建失败：可执行文件不存在"
        exit 1
    fi
    
    local version=$(get_version)
    local size=$(du -sh FileManager.app | cut -f1)
    
    log_success "构建验证通过"
    echo "  版本: $version"
    echo "  大小: $size"
}

# 自动安装
auto_install() {
    if [ "$AUTO_INSTALL" = true ]; then
        log_info "安装到 Applications 文件夹..."
        rm -rf /Applications/FileManager.app
        cp -r FileManager.app /Applications/
        log_success "已安装到 /Applications/FileManager.app"
    fi
}

# 提供安装选项
offer_install() {
    if [ "$QUICK_MODE" = true ] || [ "$AUTO_INSTALL" = true ]; then
        return 0
    fi
    
    echo ""
    echo "安装选项："
    echo "  1. 当前位置运行"
    echo "  2. 安装到 Applications"
    echo "  3. 创建桌面快捷方式"
    echo "  4. 跳过"
    echo ""
    
    read -p "请选择 (1-4): " -n 1 -r choice
    echo ""
    
    case $choice in
        1)
            log_info "可直接双击 FileManager.app 运行"
            ;;
        2)
            log_info "安装到 Applications..."
            rm -rf /Applications/FileManager.app
            cp -r FileManager.app /Applications/
            log_success "已安装到 Applications"
            ;;
        3)
            log_info "创建桌面快捷方式..."
            ln -sf "$(pwd)/FileManager.app" ~/Desktop/
            log_success "桌面快捷方式已创建"
            ;;
        4|*)
            log_info "跳过安装"
            ;;
    esac
}

# 检查 Git 状态
check_git() {
    if [ "$QUICK_MODE" = true ]; then
        return 0
    fi
    
    if git rev-parse --git-dir >/dev/null 2>&1; then
        if git status --porcelain | grep -q "^"; then
            log_warning "检测到未提交的更改"
            git status --short
            echo ""
            read -p "是否继续？(y/n): " -n 1 -r
            echo ""
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                log_info "操作已取消"
                exit 0
            fi
        fi
    fi
}

# Git 提交选项
offer_git_commit() {
    if [ "$QUICK_MODE" = true ]; then
        return 0
    fi
    
    if git rev-parse --git-dir >/dev/null 2>&1; then
        echo ""
        read -p "是否提交到 Git？(y/n): " -n 1 -r
        echo ""
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            local version=$(get_version)
            git add .
            git commit -m "Update to v$version"
            if git tag "v$version" 2>/dev/null; then
                log_success "已提交并创建标签 v$version"
            else
                log_warning "标签 v$version 已存在"
            fi
        fi
    fi
}

# 主函数
main() {
    local start_time=$(date +%s)
    
    echo "🔄 文件管理器更新工具"
    echo "======================"
    echo ""
    
    parse_args "$@"
    check_environment
    
    local current_version=$(get_version)
    log_info "当前版本: $current_version"
    
    # 更新版本号（如果指定）
    if [ -n "$NEW_VERSION" ]; then
        update_version "$NEW_VERSION"
    fi
    
    # 检查 Git 状态
    check_git
    
    # 备份
    backup_app
    
    # 构建
    if [ "$QUICK_MODE" = true ]; then
        quick_build
    else
        full_build
    fi
    
    # 验证
    verify_build
    
    # 安装
    auto_install
    offer_install
    
    # Git 操作
    offer_git_commit
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    echo ""
    log_success "更新完成！耗时: ${duration}s"
    
    if [ "$QUICK_MODE" = false ]; then
        echo ""
        echo "💡 提示："
        echo "  • 使用 --quick 可快速更新"
        echo "  • 使用 --help 查看所有选项"
    fi
}

# 错误处理
trap 'log_error "更新被中断"; exit 1' INT TERM

# 运行主函数
main "$@"