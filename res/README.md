# 资源文件说明

## 图标文件

为了完整的应用程序体验，请在此目录下放置以下文件：

### Windows 图标
**icon.ico**
- 应用程序图标文件
- 建议尺寸：16x16, 32x32, 48x48, 256x256 像素
- 格式：Windows ICO 格式
- 可以使用在线工具或图像编辑软件创建

### macOS 图标
**icon.icns**
- macOS 应用程序图标文件
- 建议尺寸：16x16, 32x32, 128x128, 256x256, 512x512, 1024x1024 像素
- 格式：Apple ICNS 格式
- 用于 .app 应用程序包

### 创建图标的方法

1. **在线工具**：
   - Windows ICO: https://convertio.co/png-ico/, https://www.icoconverter.com/
   - macOS ICNS: https://cloudconvert.com/png-to-icns, https://iconverticons.com/

2. **macOS 本地工具**：
   ```bash
   # 使用 iconutil 创建 .icns 文件
   mkdir icon.iconset
   # 将不同尺寸的 PNG 文件放入 iconset 文件夹
   # 文件命名：icon_16x16.png, icon_32x32.png, icon_128x128.png 等
   iconutil -c icns icon.iconset
   ```

3. **设计建议**：
   - 使用简洁的文件夹或文档图标
   - 采用现代扁平化设计
   - 确保在小尺寸下清晰可见
   - 为 macOS 提供高分辨率版本 (Retina 显示)

4. **替代方案**：
   如果没有自定义图标，构建脚本会跳过图标设置，使用系统默认图标。

## 文件结构

```
res/
├── app.rc          # Windows 资源文件
├── app.manifest    # Windows 应用程序清单
├── icon.ico        # Windows 应用程序图标（需要手动添加）
├── icon.icns       # macOS 应用程序图标（需要手动添加）
└── README.md       # 本说明文件
```

## 使用方法

### macOS 应用程序打包
1. 运行构建脚本：`./build_app.sh` 或 `./build.sh`
2. 生成的 `FileManager.app` 可以直接双击运行
3. 可以拖拽到 Applications 文件夹进行安装

### Windows 可执行文件
1. 运行：`cargo build --release`
2. 可执行文件位于：`target/release/file_manager.exe`