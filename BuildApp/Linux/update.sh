#!/bin/bash

# 文件管理器 Linux 智能更新脚本
# 支持快速模式、版本管理等功能

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

# 默认参数
QUICK_MODE=false
NEW_VERSION=""
NO_BACKUP=false
SHOW_HELP=false

# 显示帮助信息
show_help() {
    echo ""
    echo "文件管理器 Linux 智能更新工具"
    echo ""
    echo "用法: $0 [选项]"
    echo ""
    echo "选项:"
    echo "  -q, --quick         快速模式（仅重新编译）"
    echo "  -v, --version VER   更新版本号到 VER"
    echo "  -b, --no-backup     跳过备份"
    echo "  -h, --help          显示此帮助信息"
    echo ""
    echo "示例:"
    echo "  $0 -q                快速重新编译"
    echo "  $0 -v 1.0.0          更新到版本 1.0.0"
    echo "  $0 -v 1.0.0 -b       更新版本且跳过备份"
    echo ""
}

# 解析命令行参数
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
            NO_BACKUP=true
            shift
            ;;
        -h|--help)
            SHOW_HELP=true
            shift
            ;;
        *)
            log_error "未知选项: $1"
            show_help
            exit 1
            ;;
    esac
done

# 显示帮助并退出
if [ "$SHOW_HELP" = true ]; then
    show_help
    exit 0
fi

echo "🚀 文件管理器智能更新工具 (Linux)"
echo "=================================="

# 检查环境
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

# 获取当前版本号
get_version() {
    grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/' | tr -d '"'
}

CURRENT_VERSION=$(get_version)

# 更新版本号（如果需要）
if [ -n "$NEW_VERSION" ]; then
    log_info "更新版本号从 $CURRENT_VERSION 到 $NEW_VERSION..."
    sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
    CURRENT_VERSION="$NEW_VERSION"
fi

log_info "当前版本: $CURRENT_VERSION"

# 备份现有版本（如果存在且未禁用备份）
if [ "$NO_BACKUP" = false ] && [ -d "FileManager" ]; then
    TIMESTAMP=$(date "+%Y%m%d_%H%M%S")
    BACKUP_NAME="FileManager_backup_${CURRENT_VERSION}_${TIMESTAMP}"
    log_info "备份当前版本..."
    mv FileManager "$BACKUP_NAME"
    log_success "备份到 $BACKUP_NAME"
fi

# 快速模式或完整构建
if [ "$QUICK_MODE" = true ]; then
    log_info "⚡ 快速模式：仅重新编译..."
    cargo build --release --quiet
    
    # 创建最小目录结构
    mkdir -p FileManager/bin
    cp target/release/file_manager FileManager/bin/
    chmod +x FileManager/bin/file_manager
    
    # 创建快速启动脚本
    cat > FileManager/file_manager.sh << 'EOF'
#!/bin/bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$SCRIPT_DIR/bin/file_manager" "$@"
EOF
    chmod +x FileManager/file_manager.sh
    
    log_success "快速更新完成！"
else
    log_info "完整构建模式..."
    if ! "$SCRIPT_DIR/build.sh"; then
        log_error "构建失败"
        exit 1
    fi
fi

# 验证结果
if [ ! -x "FileManager/bin/file_manager" ]; then
    log_error "更新失败：可执行文件不存在"
    exit 1
fi

echo ""
log_success "更新完成！"
echo "  版本: $CURRENT_VERSION"
if [ "$QUICK_MODE" = true ]; then
    echo "  模式: 快速模式"
else
    echo "  模式: 完整构建"
fi
echo "  位置: $(pwd)/FileManager"
echo ""
log_info "提示: 可以运行 ./FileManager/file_manager.sh 启动程序"