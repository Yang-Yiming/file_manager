#!/bin/bash

# 文件管理器应用程序更新脚本
# 用于快速更新和重新打包应用程序

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印带颜色的消息
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# 检查是否有未提交的更改
check_git_status() {
    if git status --porcelain | grep -q "^"; then
        print_warning "检测到未提交的更改："
        git status --short
        echo
        read -p "是否继续构建？(y/n): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_info "构建已取消"
            exit 0
        fi
    fi
}

# 从 Cargo.toml 获取版本号
get_version() {
    grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/' | tr -d '"'
}

# 更新版本号
update_version() {
    local new_version=$1
    print_info "更新版本号到 $new_version"
    
    # 更新 Cargo.toml
    sed -i '' "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
    
    # 更新 Info.plist 模板
    if [ -f "build_app.sh" ]; then
        sed -i '' "s/<string>.*<\/string><!-- VERSION -->/<string>$new_version<\/string><!-- VERSION -->/" build_app.sh
    fi
    
    print_success "版本号已更新"
}

# 备份当前应用程序
backup_current_app() {
    if [ -d "FileManager.app" ]; then
        local backup_name="FileManager_backup_$(date +%Y%m%d_%H%M%S).app"
        print_info "备份当前应用程序到 $backup_name"
        cp -r FileManager.app "$backup_name"
        print_success "备份完成"
        return 0
    fi
    return 1
}

# 构建应用程序
build_app() {
    print_info "开始构建应用程序..."
    
    # 清理之前的构建
    cargo clean
    
    # 构建 release 版本
    print_info "编译 release 版本..."
    cargo build --release
    
    # 运行构建脚本
    if [ -f "build_app.sh" ]; then
        print_info "使用完整构建脚本..."
        ./build_app.sh
    elif [ -f "build.sh" ]; then
        print_info "使用简单构建脚本..."
        ./build.sh
    else
        print_error "未找到构建脚本"
        exit 1
    fi
    
    print_success "应用程序构建完成"
}

# 验证构建结果
verify_build() {
    print_info "验证构建结果..."
    
    if [ ! -d "FileManager.app" ]; then
        print_error "应用程序包未找到"
        exit 1
    fi
    
    if [ ! -x "FileManager.app/Contents/MacOS/file_manager" ]; then
        print_error "可执行文件不存在或无执行权限"
        exit 1
    fi
    
    print_success "构建验证通过"
}

# 显示应用程序信息
show_app_info() {
    local version=$(get_version)
    local size=$(du -sh FileManager.app | cut -f1)
    local executable_size=$(ls -lh FileManager.app/Contents/MacOS/file_manager | awk '{print $5}')
    
    echo
    print_success "应用程序信息："
    echo "  版本: $version"
    echo "  应用包大小: $size"
    echo "  可执行文件大小: $executable_size"
    echo "  位置: $(pwd)/FileManager.app"
    echo
}

# 提供快速安装选项
offer_installation() {
    echo "安装选项："
    echo "  1. 在当前位置运行 (双击 FileManager.app)"
    echo "  2. 复制到 Applications 文件夹"
    echo "  3. 创建桌面快捷方式"
    echo "  4. 跳过"
    echo
    
    read -p "请选择安装方式 (1-4): " -n 1 -r
    echo
    
    case $REPLY in
        1)
            print_info "可以直接双击 FileManager.app 运行"
            ;;
        2)
            print_info "复制到 Applications 文件夹..."
            cp -r FileManager.app /Applications/
            print_success "已安装到 /Applications/FileManager.app"
            ;;
        3)
            print_info "创建桌面快捷方式..."
            ln -sf "$(pwd)/FileManager.app" ~/Desktop/
            print_success "桌面快捷方式已创建"
            ;;
        4)
            print_info "跳过安装"
            ;;
        *)
            print_info "无效选择，跳过安装"
            ;;
    esac
}

# Git 提交选项
offer_git_commit() {
    if git rev-parse --git-dir > /dev/null 2>&1; then
        echo
        read -p "是否提交更改到 Git？(y/n): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            local version=$(get_version)
            git add .
            git commit -m "Release v$version: Update application build"
            git tag "v$version" 2>/dev/null || print_warning "标签 v$version 已存在"
            print_success "已提交到 Git 并创建标签 v$version"
        fi
    fi
}

# 主函数
main() {
    echo "🚀 文件管理器应用程序更新工具"
    echo "=================================="
    echo
    
    # 检查是否在正确的目录
    if [ ! -f "Cargo.toml" ]; then
        print_error "请在项目根目录运行此脚本"
        exit 1
    fi
    
    local current_version=$(get_version)
    print_info "当前版本: $current_version"
    
    # 解析命令行参数
    while [[ $# -gt 0 ]]; do
        case $1 in
            --version|-v)
                update_version "$2"
                shift 2
                ;;
            --no-backup)
                SKIP_BACKUP=true
                shift
                ;;
            --quick|-q)
                QUICK_MODE=true
                shift
                ;;
            --help|-h)
                echo "用法: $0 [选项]"
                echo "选项:"
                echo "  -v, --version VERSION    更新版本号"
                echo "  --no-backup             跳过备份"
                echo "  -q, --quick             快速模式（跳过交互）"
                echo "  -h, --help              显示此帮助"
                exit 0
                ;;
            *)
                print_error "未知选项: $1"
                exit 1
                ;;
        esac
    done
    
    # 检查 Git 状态
    if [ "$QUICK_MODE" != "true" ]; then
        check_git_status
    fi
    
    # 备份当前应用程序
    if [ "$SKIP_BACKUP" != "true" ]; then
        backup_current_app
    fi
    
    # 构建应用程序
    build_app
    
    # 验证构建
    verify_build
    
    # 显示信息
    show_app_info
    
    # 交互式选项（非快速模式）
    if [ "$QUICK_MODE" != "true" ]; then
        offer_installation
        offer_git_commit
    fi
    
    print_success "更新完成！"
}

# 运行主函数
main "$@"