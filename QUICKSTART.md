# 快速开始指南

## 开发命令速查

### 启动开发环境

```bash
# 🚀 启动完整应用（前端 + 后端）
bun run dev

# 这会自动：
# 1. 启动 Vite 前端开发服务器 (http://localhost:1420)
# 2. 启动 Tauri 应用窗口
# 3. 监听文件变化，自动热重载

# 原理：
# - tauri dev 会先运行 beforeDevCommand (启动 Vite)
# - 然后启动 Tauri 应用加载 localhost:1420
```

### 其他开发命令

```bash
# 📦 仅启动前端（用于前端开发/调试）
bun run dev:frontend

# 🧪 使用测试脚本启动（包含检查和清理）
./scripts/test-dev.sh

# 🔨 TypeScript 类型检查
bun run tsc --noEmit

# 🦀 Rust 测试
cargo test --manifest-path src-tauri/Cargo.toml

# 🎨 Rust 代码检查
cargo clippy --manifest-path src-tauri/Cargo.toml
```

### 快速测试订阅（命令行）

```bash
# ⚡ 最快的测试方式 - 无需 UI
cargo run --bin cli -- subs.txt clash.yml

# 查看生成的配置
cat clash.yml | grep "name:" | head -20
```

### 构建生产版本

```bash
# 🏗️ 完整构建（应用 + DMG）
bun run build:all

# 或分步构建
bun run tauri:build    # 构建应用
bun run build:dmg      # 创建 DMG 安装包

# 快速构建（包含测试）
bun run build:quick
```

## 常用工作流程

### 工作流 1：前端开发

```bash
# 1. 启动完整应用
bun run dev

# 2. 修改 src/App.tsx

# 3. 保存后自动热重载，在应用中查看效果

# 4. 按 Ctrl+C 停止
```

### 工作流 2：后端开发

```bash
# 1. 修改 Rust 代码
vim src-tauri/src/proxy.rs

# 2. 快速测试（命令行）
cargo run --bin cli -- subs.txt test.yml

# 3. 查看结果
cat test.yml | head -50

# 4. 或启动完整应用测试
bun run dev
```

### 工作流 3：测试订阅分组逻辑

```bash
# 1. 准备订阅文件
echo "https://your-subscription-url" > test_subs.txt

# 2. 运行命令行工具
cargo run --bin cli -- test_subs.txt output.yml

# 3. 查看统计和分组
# 输出会显示：
#   - 总节点数
#   - 地区分布
#   - 创建的分组数量

# 4. 检查生成的配置
cat output.yml
```

### 工作流 4：打包发布

```bash
# 1. 运行所有测试
cargo test --manifest-path src-tauri/Cargo.toml
bun run tsc --noEmit

# 2. 完整构建
bun run build:all

# 3. 测试 DMG
open src-tauri/target/release/bundle/dmg/*.dmg

# 4. 测试安装和运行
```

## 目录结构说明

```
proxy-sub-manager/
├── src/                      # 🎨 前端代码（React + TypeScript）
│   ├── App.tsx              # 主界面（订阅管理）
│   ├── main.tsx             # 入口文件
│   └── App.css              # 样式
│
├── src-tauri/               # 🦀 后端代码（Rust + Tauri）
│   ├── src/
│   │   ├── bin/
│   │   │   └── cli.rs       # ⚡ 命令行工具
│   │   ├── lib.rs           # Tauri 命令定义
│   │   ├── config.rs        # 配置文件加载
│   │   ├── proxy.rs         # 代理分组逻辑
│   │   ├── subscription.rs  # 订阅获取
│   │   └── server.rs        # HTTP 服务器
│   ├── Cargo.toml           # Rust 依赖
│   └── icons/               # 应用图标
│
├── scripts/                 # 🛠️ 工具脚本
│   ├── simple-dmg.sh        # DMG 打包
│   ├── test-dev.sh          # 开发测试
│   └── create-simple-icon.sh # 图标生成
│
├── subs.txt                 # 📝 订阅列表文件
├── subscriptions.json       # 📋 应用配置
├── basic_test.yml           # ⚙️ 基础 Clash 配置
│
└── 文档/
    ├── README.md            # 主文档
    ├── CLI.md               # 命令行工具文档
    ├── QUICKSTART.md        # 本文档
    └── AGENTS.md            # 开发指南
```

## 配置文件说明

### subs.txt - 订阅列表（命令行工具用）

```text
# 这是注释
https://example1.com/subscribe?token=xxx
https://example2.com/subscribe?token=yyy

# 可以添加多个订阅
https://example3.com/subscribe?token=zzz
```

### subscriptions.json - 应用配置（UI 用）

```json
{
  "subscriptions": [
    {
      "name": "机场A",
      "url": "https://example.com/subscribe?token=xxx",
      "enabled": true
    }
  ],
  "server": {
    "port": 8080,
    "host": "127.0.0.1"
  },
  "output": {
    "path": "output_config.yaml"
  },
  "basic_config": {
    "path": "basic_test.yml"
  }
}
```

## 常见问题

### Q: `bun run dev` 没有打开 Tauri 窗口？

**A:** 现在已经修复！`bun run dev` 会同时启动前端和 Tauri 应用。

如果还有问题：
```bash
# 使用完整命令
bun run tauri:dev

# 或使用测试脚本
./scripts/test-dev.sh
```

### Q: 修改代码后没有自动重载？

**A:** 
- **前端修改**：会自动热重载
- **Rust 修改**：需要重新编译，Tauri 会自动检测并重启

### Q: 命令行工具和 UI 有什么区别？

**A:** 
| 特性 | 命令行工具 | UI 应用 |
|------|-----------|---------|
| 速度 | ⚡ 快速 | 稍慢 |
| 用途 | 测试/批处理 | 日常使用 |
| 配置 | subs.txt | subscriptions.json |
| 输出 | clash.yml | HTTP 服务器 |
| 优势 | 快速迭代 | 用户友好 |

**使用建议：**
- 开发测试 → 用命令行工具
- 日常使用 → 用 UI 应用

### Q: 如何调试 Rust 代码？

**A:** 
```bash
# 1. 添加打印语句
println!("Debug: {:?}", variable);

# 2. 运行并查看输出
bun run dev

# 3. 或使用命令行工具查看详细输出
cargo run --bin cli -- subs.txt test.yml
```

### Q: 如何查看生成的订阅配置？

**A:** 
```bash
# 方式1：命令行工具直接生成
cargo run --bin cli -- subs.txt output.yml
cat output.yml

# 方式2：UI 应用生成
bun run dev
# 点击"启动服务器"
curl http://127.0.0.1:8080/config > output.yml
cat output.yml

# 方式3：查看本地文件
cat output_config.yaml
```

## 性能提示

```bash
# 🚀 编译优化
cargo build --bin cli --release   # 发布模式，速度更快

# 📦 减小包体积
cargo build --bin cli --release --strip
strip src-tauri/target/release/cli

# 🧹 清理缓存
cargo clean --manifest-path src-tauri/Cargo.toml
rm -rf node_modules && bun install
```

## 推荐开发工具

- **VS Code** + Rust Analyzer
- **iTerm2** 或其他终端
- **Clash Verge** 用于测试订阅

## 下一步

1. ✅ 运行 `bun run dev` 启动应用
2. 📝 在 UI 中添加你的订阅
3. 🚀 点击"启动服务器"
4. 🌐 在 Clash 中添加 http://127.0.0.1:8080/config
5. 🎉 开始使用！

---

💡 **提示**: 保持这个文档在手边，开发时随时参考！
