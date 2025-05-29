#!/bin/bash

# 文件管理器 macOS App 构建脚本
# 自动构建并打包成 .app 应用程序

set -e

# 从 Cargo.toml 获取版本号
get_version() {
    grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/' | tr -d '"'
}

VERSION=$(get_version)
echo "🚀 开始构建文件管理器应用程序 v$VERSION..."

# 清理之前的构建
echo "🧹 清理之前的构建..."
cargo clean
rm -rf FileManager.app

# 构建 release 版本
echo "🔨 构建 release 版本..."
cargo build --release

# 创建 .app 目录结构
echo "📁 创建应用程序包结构..."
mkdir -p FileManager.app/Contents/{MacOS,Resources}

# 复制可执行文件
echo "📋 复制可执行文件..."
cp target/release/file_manager FileManager.app/Contents/MacOS/file_manager
chmod +x FileManager.app/Contents/MacOS/file_manager

# 创建 Info.plist
echo "📄 创建 Info.plist..."
cat > FileManager.app/Contents/Info.plist << 'EOF'
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
    <string>$VERSION</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright © 2024 File Manager Team. All rights reserved.</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.utilities</string>
    <key>NSRequiresAquaSystemAppearance</key>
    <false/>
    <key>LSUIElement</key>
    <false/>
</dict>
</plist>
EOF

# 复制图标（如果存在）
if [ -f "res/icon.icns" ]; then
    echo "🎨 复制应用程序图标..."
    cp res/icon.icns FileManager.app/Contents/Resources/
    # 在 Info.plist 中添加图标引用
    /usr/libexec/PlistBuddy -c "Add :CFBundleIconFile string icon.icns" FileManager.app/Contents/Info.plist 2>/dev/null || true
fi

# 设置应用程序属性
echo "⚙️  设置应用程序属性..."
touch FileManager.app
SetFile -a B FileManager.app 2>/dev/null || true

# 优化应用程序包
echo "🔧 优化应用程序包..."
find FileManager.app -name ".DS_Store" -delete 2>/dev/null || true

# 验证构建
echo "✅ 验证构建结果..."
if [ -x "FileManager.app/Contents/MacOS/file_manager" ]; then
    echo "✨ 构建成功！"
    echo "📱 应用程序已保存到: $(pwd)/FileManager.app"
    echo ""
    echo "🎯 使用方法："
    echo "   1. 双击 FileManager.app 直接运行"
    echo "   2. 或者拖拽到 Applications 文件夹安装"
    echo "   3. 使用 ./update_app.sh 进行后续更新"
    echo ""
    echo "📊 应用程序信息："
    echo "   版本: $VERSION"
    ls -la FileManager.app/Contents/MacOS/file_manager
    echo "   应用程序包大小: $(du -sh FileManager.app | cut -f1)"
else
    echo "❌ 构建失败：可执行文件不存在或无执行权限"
    exit 1
fi

echo "🎉 构建完成！"