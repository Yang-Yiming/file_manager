# 文件管理器 - 快速访问工具

一个简洁高效的文件快速访问管理器，支持智能标签管理、数据导入导出等功能。

## ✨ 主要特性

- 🗂️ **快速文件访问** - 一键打开文件和文件夹
- 🏷️ **智能标签系统** - 多标签过滤、标签建议、批量操作
- 📤 **数据导入导出** - JSON格式数据备份和迁移
- 🎨 **主题支持** - 浅色/深色/系统主题
- 🔍 **强大搜索** - 文件名、描述、标签全文搜索
- 💾 **数据安全** - 本地存储，支持备份恢复

## 🚀 快速开始

### 直接下载使用

1. 下载预构建的 `FileManager.app`
2. 双击运行或拖拽到 Applications 文件夹
3. 开始添加和管理文件

### 从源码构建

```bash
# 克隆项目
git clone <repository-url>
cd file_manager

# 构建应用程序
./BuildApp/MacOS/build.sh

# 或快速更新
./BuildApp/MacOS/update.sh --quick
```

## 🎯 使用方法

### 基础操作

1. **添加文件**：点击"添加"按钮，选择文件或文件夹
2. **设置标签**：使用 `#标签名` 格式，如 `#工作 #重要`
3. **搜索过滤**：在搜索框输入关键词或标签
4. **快速打开**：点击文件名直接打开

### 标签管理

```
标签格式：#工作 #项目A #重要
多标签过滤：#工作 #重要（显示同时包含两个标签的文件）
点击标签：在文件列表中点击标签快速过滤
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
├── BuildApp/MacOS/         # 构建脚本
│   ├── build.sh           # 完整构建
│   └── update.sh          # 智能更新
├── FileManager.app/        # 构建输出
├── Cargo.toml             # 项目配置
└── README.md              # 本文档
```

## 🎨 使用技巧

### 标签最佳实践

```
工作场景：
#工作 #项目A #重要 #文档
#工作 #项目A #会议记录
#工作 #日常 #邮件

个人文件：
#个人 #照片 #旅行
#个人 #学习 #编程
#个人 #财务 #税务
```

### 高效搜索

- **文件名搜索**：直接输入文件名关键词
- **标签过滤**：使用 `#标签名` 精确过滤
- **组合搜索**：同时使用文件名和标签
- **多标签**：用空格分隔多个标签

### 数据迁移

1. **导出当前数据**到 JSON 文件
2. **在新设备上安装**应用程序
3. **导入 JSON 文件**恢复数据
4. **验证数据完整性**

## 🔄 版本历史

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