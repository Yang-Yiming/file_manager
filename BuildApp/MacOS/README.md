# macOS 构建脚本说明

## 📋 脚本概览

| 脚本 | 用途 | 速度 | 推荐场景 |
|------|------|------|----------|
| `build.sh` | 完整构建 | ⚡⚡ | 首次构建、正式发布 |
| `update.sh` | 智能更新 | ⚡⚡⚡ | 日常开发、版本管理 |

## 🚀 快速使用

### 日常开发
```bash
# 快速重新编译
./update.sh --quick

# 或简写
./update.sh -q
```

### 版本发布
```bash
# 更新版本号并构建
./update.sh --version 0.3.0

# 自动安装到 Applications
./update.sh -v 0.3.0 -i
```

### 首次构建
```bash
# 完整构建（推荐）
./build.sh
```

## ⚙️ update.sh 选项

```bash
-q, --quick         # 快速模式（仅重新编译）
-v, --version VER   # 更新版本号
-b, --no-backup     # 跳过备份
-i, --install       # 自动安装到 Applications
-h, --help          # 显示帮助
```

## 📝 使用示例

```bash
# 开发时快速迭代
./update.sh -q

# 版本更新 + 安装
./update.sh -v 1.0.0 -i

# 跳过备份的快速更新
./update.sh -q -b

# 查看所有选项
./update.sh --help
```

## 🔄 典型工作流

1. **修改代码** → `./update.sh -q` → **测试**
2. **功能完成** → `./update.sh -v 0.x.0` → **发布**
3. **首次部署** → `./build.sh` → **分发**

## 📦 输出说明

- 构建成功后生成 `FileManager.app`
- 自动备份旧版本（除非使用 `-b`）
- 支持直接运行或安装到 Applications

## 💡 提示

- 开发期间使用 `update.sh -q` 最快
- 发布前使用 `update.sh -v X.Y.Z`
- 脚本可从任意目录运行
- 自动处理版本号和依赖关系