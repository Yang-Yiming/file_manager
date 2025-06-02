# 文件管理器 - 紧凑简洁的快速访问工具

一个极简高效的文件快速访问管理器，专注于核心功能，界面紧凑不冗余，支持智能标签管理和数据管理。

## ✨ 主要特性

- 🗂️ **快速访问** - 一键打开文件、文件夹和网页链接
- 📋 **集合管理** - 创建集合，将多个项目组合在一起，一键批量打开
- 🏷️ **智能标签系统** - 多标签过滤、标签建议、批量操作
- 📤 **数据导入导出** - JSON格式数据备份和迁移
- 🎨 **简洁UI设计** - 紧凑布局，无冗余元素，支持浅色/深色/系统主题
- 🔍 **强大搜索** - 文件名、昵称、描述、标签全文搜索，支持拼音搜索
- 🏷️ **自定义昵称** - 为目录设置易记的昵称，支持拼音快速查找
- 💾 **数据安全** - 本地存储，支持备份恢复
- ⚡ **高性能** - 精简代码，快速响应，低内存占用

## 🚀 快速开始

### 直接下载使用

1. 下载预构建的 `FileManager.app`
2. 双击运行或拖拽到 Applications 文件夹
3. 开始添加和管理文件

### 从源码构建

#### macOS
```bash
# 克隆项目
git clone <repository-url>
cd file_manager

# 构建应用程序
./BuildApp/MacOS/build.sh

# 或快速更新
./BuildApp/MacOS/update.sh --quick
```

#### Windows
```cmd
REM 克隆项目
git clone <repository-url>
cd file_manager

REM 构建应用程序
BuildApp\Windows\build.bat

REM 或快速更新
BuildApp\Windows\update.bat --quick
```

#### Linux
```bash
# 克隆项目
git clone <repository-url>
cd file_manager

# 构建应用程序
./BuildApp/Linux/build.sh

# 或快速更新
./BuildApp/Linux/update.sh --quick
```

## 🎯 使用方法

### 基础操作

1. **添加条目**：点击"添加"按钮，选择文件、文件夹、网页链接或集合
2. **设置昵称**：为条目设置易记的昵称（支持中文拼音搜索）
3. **设置标签**：使用 `#标签名` 格式，如 `#工作 #重要`
4. **创建集合**：将相关的文件、文件夹和网页链接组合成集合
5. **搜索过滤**：在搜索框输入关键词、拼音或标签
6. **快速打开**：点击名称直接打开文件、网页或整个集合

### 网页链接功能

1. **添加网页链接**：
   - 选择"网页链接"类型
   - 输入完整URL（如：https://www.example.com）
   - 系统会自动提取网站名称作为默认名称
   - 可自定义名称和昵称

2. **链接管理**：
   - 网页链接显示为 `[LINK]` 图标
   - 点击链接名称在默认浏览器中打开
   - 支持常用网站的快速收藏
   - 可为链接添加标签分类

3. **链接示例**：
   ```
   名称: "GitHub"
   昵称: "代码仓库"
   URL: "https://www.github.com"
   标签: #开发 #代码 #工具

   名称: "Google"
   昵称: "谷歌搜索"
   URL: "https://www.google.com"
   标签: #搜索 #工具 #常用
   ```

### 集合功能

1. **创建集合**：
   - 选择"集合"类型
   - 输入集合名称和描述
   - 集合可以包含文件、文件夹和网页链接

2. **管理集合**：
   - 点击"集合"按钮打开集合管理器
   - 选择要编辑的集合
   - 勾选要包含的子项目
   - 保存集合配置

3. **使用集合**：
   - 集合显示为 `[集合]` 图标
   - 点击集合名称会依次打开所有子项目
   - 可为集合添加标签和昵称

4. **集合示例**：
   ```
   名称: "项目A工作环境"
   昵称: "A项目"
   子项目:
   - 项目文档文件夹
   - 代码仓库文件夹
   - GitHub项目页面
   - 设计稿文件夹
   标签: #工作 #项目A #开发环境

   名称: "每日工作"
   昵称: "日常工作"
   子项目:
   - 邮箱网页
   - 工作日志文档
   - 团队协作工具
   - 项目管理系统
   标签: #工作 #日常 #效率
   ```

### 界面说明

**顶部工具栏**
- 统一搜索：支持文件名搜索和标签搜索（使用#前缀）
- 功能按钮：添加、标签、集合、导入导出、设置
- 侧边栏关闭：点击×按钮关闭当前打开的面板

**主要内容区**
- 紧凑模式：可选择单行显示文件，节省空间
- 普通模式：多行详细显示文件信息
- 即时标签：点击标签自动添加到搜索框
- 操作按钮：编辑和删除图标按钮
- 灰暗配色：类似Zed的专业色调

**侧边面板**
- 直接操作：去掉多余包装，直接显示表单
- 即时反馈：操作状态及时显示
- 紧凑布局：最小化面板宽度

### 搜索和标签

**统一搜索框支持：**
- 文件名搜索：直接输入文件名关键词
- 标签搜索：使用#前缀，如 `#工作 #重要`
- 混合搜索：可同时搜索文件名和标签
- 点击标签：在文件列表中点击标签自动添加到搜索框

**搜索示例：**
```
#工作          - 搜索包含"工作"标签的条目
项目           - 搜索名称包含"项目"的条目
#工作 会议      - 搜索包含"工作"标签且名称包含"会议"的条目
#重要 #紧急    - 搜索同时包含"重要"和"紧急"标签的条目
github         - 搜索名称或昵称包含"github"的条目
```

### 数据管理

- **导出数据**：导入导出面板 → 导出数据 → 选择保存位置
- **导入数据**：导入导出面板 → 选择模式 → 导入JSON文件
- **备份数据**：设置面板 → 数据备份 → 创建备份

## 🔧 开发指南

### 环境要求

- Rust 1.70+
- macOS 10.13+
- Xcode Command Line Tools

### 构建脚本

| 脚本 | 用途 | 特点 |
|------|------|------|
| `build.sh` | 完整构建 | 包含图标、优化等 |
| `update.sh` | 智能更新 | 版本管理、备份 |

### 开发工作流

```bash
# 修改代码后快速测试
./BuildApp/MacOS/update.sh --quick

# 更新版本并构建
./BuildApp/MacOS/update.sh --version 0.3.0

# 完整构建并安装
./BuildApp/MacOS/update.sh --install
```

### 常用命令

```bash
# 查看帮助
./BuildApp/MacOS/update.sh --help

# 快速模式（开发时）
./BuildApp/MacOS/update.sh -q

# 更新版本 + 安装
./BuildApp/MacOS/update.sh -v 0.3.0 -i

# 跳过备份的快速更新
./BuildApp/MacOS/update.sh -q -b

# Windows 示例
BuildApp\Windows\update.bat -q
BuildApp\Windows\update.bat -v 1.0.0

# Linux 示例
./BuildApp/Linux/update.sh -q
./BuildApp/Linux/update.sh -v 1.0.0
```

## 📁 项目结构

```
file_manager/
├── src/                    # 源代码
│   ├── main.rs            # 主入口
│   ├── app.rs             # 应用程序逻辑
│   ├── config.rs          # 配置管理
│   ├── file_entry.rs      # 文件条目
│   └── fonts.rs           # 字体设置
├── res/                    # 资源文件
├── BuildApp/               # 跨平台构建脚本
│   ├── MacOS/             # macOS 构建脚本
│   │   ├── build.sh       # 完整构建
│   │   ├── update.sh      # 智能更新
│   │   └── README.md      # macOS 构建说明
│   ├── Windows/           # Windows 构建脚本
│   │   ├── build.bat      # 完整构建
│   │   ├── update.bat     # 智能更新
│   │   └── README.md      # Windows 构建说明
│   └── Linux/             # Linux 构建脚本
│       ├── build.sh       # 完整构建
│       ├── update.sh      # 智能更新
│       └── README.md      # Linux 构建说明
├── FileManager.app/        # macOS 构建输出
├── FileManager/            # Windows/Linux 构建输出
├── Cargo.toml             # 项目配置
├── build.rs               # 构建配置
└── README.md              # 项目文档
```

## 🔧 跨平台开发指南

### 系统要求

| 平台 | 系统版本 | 依赖要求 |
|------|----------|----------|
| **macOS** | 10.13+ | Xcode Command Line Tools |
| **Windows** | Windows 7+ | Visual Studio Build Tools |
| **Linux** | 主流发行版 | GTK+ 3.0, X11 开发包 |

### 构建命令对照

| 操作 | macOS | Windows | Linux |
|------|-------|---------|-------|
| **完整构建** | `./BuildApp/MacOS/build.sh` | `BuildApp\Windows\build.bat` | `./BuildApp/Linux/build.sh` |
| **快速更新** | `./BuildApp/MacOS/update.sh -q` | `BuildApp\Windows\update.bat -q` | `./BuildApp/Linux/update.sh -q` |
| **版本更新** | `./BuildApp/MacOS/update.sh -v 1.0.0` | `BuildApp\Windows\update.bat -v 1.0.0` | `./BuildApp/Linux/update.sh -v 1.0.0` |
| **获取帮助** | `./BuildApp/MacOS/update.sh -h` | `BuildApp\Windows\update.bat -h` | `./BuildApp/Linux/update.sh -h` |

### 输出文件说明

| 平台 | 输出目录 | 主要文件 | 安装方式 |
|------|----------|----------|----------|
| **macOS** | `FileManager.app` | 应用程序包 | 拖拽到 Applications |
| **Windows** | `FileManager/` | `file_manager.exe` | 便携式，直接运行 |
| **Linux** | `FileManager/` | `bin/file_manager` | 便携式或系统安装 |
```

## 🎨 使用技巧

### 标签和昵称最佳实践

```
工作场景：
名称: "项目资料"
昵称: "A项目文档"（可搜索"axiang"或"axiangmudangan"）
标签: #工作 #项目A #重要 #文档

名称: "Meeting Notes 20241201"
昵称: "会议记录"（可搜索"huiyi"）
标签: #工作 #项目A #会议记录

网页链接：
名称: "GitHub"
昵称: "代码仓库"（可搜索"daima"或"cangku"）
URL: "https://www.github.com"
标签: #开发 #代码 #工具

名称: "Stack Overflow"
昵称: "程序员问答"（可搜索"chengxuyuan"）
URL: "https://stackoverflow.com"
标签: #开发 #学习 #问答

集合组合：
名称: "前端开发环境"
昵称: "前端工作"（可搜索"qianduan"）
子项目: VS Code、Node.js文档、GitHub仓库、设计稿文件夹
标签: #开发 #前端 #工作环境

名称: "学习资源"
昵称: "学习材料"（可搜索"xuexi"）
子项目: 电子书文件夹、在线课程网站、笔记文档、练习项目
标签: #学习 #资源 #个人成长

个人文件：
名称: "我的照片"
昵称: "旅行照片"（可搜索"lvxing"）
标签: #个人 #照片 #旅行
```

### 高效搜索

- **名称搜索**：直接输入条目名称关键词（支持文件、文件夹、网页链接）
- **昵称搜索**：搜索自定义昵称
- **拼音搜索**：输入拼音首字母或完整拼音，如"我是谁"可搜索"woshi"或"woshishui"
- **标签过滤**：使用 `#标签名` 精确过滤
- **组合搜索**：同时使用名称、昵称和标签
- **多标签**：用空格分隔多个标签
- **URL搜索**：可搜索网页链接的URL地址

### 数据迁移

1. **导出当前数据**到 JSON 文件
2. **在新设备上安装**应用程序
3. **导入 JSON 文件**恢复数据
4. **验证数据完整性**

## 🔄 版本历史
- **v0.3.1** 大幅优化集合功能
  - 添加多个快捷键，右键方式
  - 优化了ui界面

- **v0.3.0** - 📋 新增集合功能
  - 支持创建集合，将多个文件、文件夹、网页链接组合在一起
  - 集合管理器界面，可视化选择和管理子项目
  - 一键打开集合中的所有项目，提升工作效率
  - 集合专用图标 `[集合]` 和虚拟路径管理
  - 支持为集合添加标签、昵称和描述
  - 完全向后兼容现有数据格式

- **v0.2.3** - 🌐 新增网页链接支持
  - 支持添加、管理和打开网页链接
  - 自动从URL提取网站名称
  - URL格式验证和状态提示
  - 网页链接专用图标 `[LINK]`
  - 在默认浏览器中打开链接
  - 向后兼容现有数据格式

- **v0.2.2** - 🎯 紧凑简洁UI重构
  - 统一搜索功能：单一搜索框支持文件名和标签（#标签名）
  - 去掉重复的标签搜索栏，界面更加紧凑
  - 添加紧凑模式选项，文件列表可选择单行显示
  - 类似Zed的灰暗色调，护眼且专业
  - 侧边栏可正常关闭，优化交互体验
  - 简化UI组件，去掉冗余装饰，专注核心功能
  - 优化窗口尺寸，适合小屏幕使用
- **v0.2.1** - 新增自定义昵称功能、支持拼音搜索
- **v0.2.0** - 新增数据导入导出、增强标签管理
- **v0.1.0** - 基础文件管理功能

## 🐛 故障排除

### 应用无法启动

```bash
chmod +x FileManager.app/Contents/MacOS/file_manager
```

### 构建失败

```bash
cargo clean
./BuildApp/MacOS/build.sh
```

### 数据丢失

1. 检查 `file_manager_data.json` 文件
2. 从备份文件恢复
3. 重新导入数据

## 📄 许可证

MIT License - 详见 LICENSE 文件

## 🤝 贡献

欢迎提交 Issues 和 Pull Requests！

---

**提示**：建议定期使用备份功能保护数据，应用支持多设备间的数据同步。
