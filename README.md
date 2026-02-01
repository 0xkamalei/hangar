# 代理订阅管理器 (Proxy Subscription Manager)

一个基于 Tauri 2.0 构建的桌面应用，用于聚合多个代理订阅源并提供统一的 HTTP 服务器。

## 功能特性

- ✅ 支持多订阅源聚合
- ✅ 自动解析 Clash 配置格式
- ✅ 智能地区分组（香港、台湾、日本、新加坡等）
- ✅ 服务专用组（ChatGPT、Gemini、Google、Netflix、Telegram）
- ✅ 内置 HTTP 服务器，提供实时订阅链接
- ✅ 友好的图形界面
- ✅ 启动/停止服务器控制
- ✅ 实时状态显示

## 技术栈

**前端:**
- React 19
- TypeScript 5.8
- Vite 7

**后端:**
- Rust 2021
- Tauri 2.0
- Tokio (异步运行时)
- Axum (HTTP 服务器)

## ✅ 项目状态

- ✅ 所有功能已实现并测试通过
- ✅ 开发环境配置正确
- ✅ 可以立即使用

详细验证报告：[VERIFICATION.md](VERIFICATION.md)

## 快速开始

### 开发环境要求

- Node.js 18+ 或 Bun 1.0+
- Rust 1.70+

### 命令行工具（推荐用于快速测试）

无需启动 UI，直接生成 Clash 配置：

```bash
# 使用 subs.txt 生成配置
cargo run --bin cli -- subs.txt clash.yml

# 查看详细使用文档
cat CLI.md
```

**优势：**
- ⚡ 快速测试订阅合并逻辑
- 🔍 查看详细节点统计
- 🎯 验证地区识别和分组
- 📝 无需 UI 即可生成配置

详细文档：[CLI.md](CLI.md)

### 安装依赖

```bash
bun install
```

### 配置订阅

编辑 `subscriptions.json` 文件，添加你的订阅链接：

```json
{
  "subscriptions": [
    {
      "name": "机场名称",
      "url": "https://example.com/sub",
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
    "path": "_docs/basic.yml"
  }
}
```

### 运行开发模式

```bash
# 启动完整应用（推荐）
bun run dev

# 或使用其他方式
bun run tauri:dev        # 同上
./scripts/test-dev.sh    # 使用测试脚本

# 仅启动前端（用于前端开发）
bun run dev:frontend
```

### 构建生产版本

```bash
# 完整构建流程（推荐）
bun run build:all

# 或分步构建
bun run tauri build    # 构建应用
bun run build:dmg      # 创建 DMG 安装包
```

构建产物位于：
- macOS 应用: `src-tauri/target/release/bundle/macos/proxy-sub-manager.app`
- DMG 安装包: `src-tauri/target/release/bundle/dmg/proxy-sub-manager_0.1.0_custom_arm64.dmg`

更多打包选项请查看：[打包脚本文档](scripts/README.md)

## 使用说明

1. 启动应用
2. 点击"启动服务器"按钮
3. 应用会自动拉取所有启用的订阅并合并配置
4. 在 Clash Verge 或其他 Clash 客户端中添加订阅链接：
   ```
   http://127.0.0.1:8080/config
   ```
5. 更新订阅即可使用合并后的代理节点

## 项目结构

```
proxy-sub-manager/
├── src/                    # 前端 React/TypeScript 代码
│   ├── App.tsx            # 主组件
│   └── main.tsx           # 入口文件
├── src-tauri/             # Rust 后端代码
│   ├── src/
│   │   ├── lib.rs         # Tauri 命令
│   │   ├── config.rs      # 配置加载
│   │   ├── proxy.rs       # 代理逻辑
│   │   ├── subscription.rs # 订阅获取
│   │   └── server.rs      # HTTP 服务器
│   └── Cargo.toml         # Rust 依赖
├── _docs/                 # 配置文件模板
├── subscriptions.json     # 订阅配置
└── test_e2e.sh           # E2E 测试脚本
```

## 测试

### 运行单元测试

```bash
# Rust 单元测试
cargo test --manifest-path src-tauri/Cargo.toml

# TypeScript 类型检查
npx tsc --noEmit
```

### 运行 E2E 测试

```bash
./test_e2e.sh
```

### 代码检查

```bash
# Rust 代码检查
cargo clippy --manifest-path src-tauri/Cargo.toml

# Rust 代码格式化
cargo fmt --manifest-path src-tauri/Cargo.toml
```

## 开发指南

详细的开发指南请参考 `AGENTS.md` 文件。

## 推荐 IDE 配置

- [VS Code](https://code.visualstudio.com/)
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## License

MIT
