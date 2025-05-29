#!/bin/bash

# 快速更新脚本 - 一键重新构建应用程序
# 适用于日常开发中的快速迭代

echo "🔄 快速更新文件管理器..."

# 检查是否在正确目录
if [ ! -f "Cargo.toml" ]; then
    echo "❌ 请在项目根目录运行此脚本"
    exit 1
fi

# 快速构建 release 版本
echo "⚡ 编译中..."
cargo build --release --quiet

# 更新应用程序包
echo "📦 更新应用程序包..."
rm -rf FileManager.app
mkdir -p FileManager.app/Contents/MacOS

# 复制可执行文件
cp target/release/file_manager FileManager.app/Contents/MacOS/
chmod +x FileManager.app/Contents/MacOS/file_manager

# 获取版本号
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/' | tr -d '"')

# 创建 Info.plist
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
    <string>$VERSION</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
</dict>
</plist>
EOF

echo "✅ 更新完成！版本: $VERSION"
echo "💡 双击 FileManager.app 即可运行"