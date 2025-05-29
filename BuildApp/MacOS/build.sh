#!/bin/bash

# 简单构建脚本
echo "构建文件管理器..."
cargo build --release

echo "创建应用程序包..."
rm -rf FileManager.app
mkdir -p FileManager.app/Contents/MacOS

# 复制可执行文件
cp target/release/file_manager FileManager.app/Contents/MacOS/
chmod +x FileManager.app/Contents/MacOS/file_manager

# 创建 Info.plist
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
    <string>0.2.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
</dict>
</plist>
EOF

echo "✅ 完成！现在可以双击 FileManager.app 运行了"