# Hangar - Clash 订阅管理工具

一个强大的 CLI 工具，用于管理 Clash 订阅配置，支持 AI 辅助配置、文件监控和后台运行。

## 功能特性

- 🔄 **订阅管理**: 添加、列出、删除 Clash 订阅
- 🤖 **AI 辅助**: 使用自然语言修改配置
- 🌐 **配置服务器**: 内置 HTTP 服务器，提供订阅 URL
- 👀 **文件监控**: 自动检测配置文件变化并重新加载
- 🔧 **后台运行**: Daemon 模式支持，适合生产环境
- 📝 **版本管理**: 配置历史记录和回滚功能

## 安装

### 方式一：从源码构建并安装

```bash
# 1. 克隆仓库
git clone <your-repo-url>
cd hangar

# 2. 运行安装脚本
./install.sh
```

安装脚本会：
- 自动构建 release 版本
- 将二进制文件复制到 `/usr/local/bin`
- 使 `hangar` 命令全局可用

### 方式二：手动构建

```bash
# 构建
cd src-tauri
cargo build --release

# 二进制文件位于
# src-tauri/target/release/hangar
```

## 快速开始

### 1. 添加订阅

```bash
hangar sub add <订阅URL> --name "我的订阅"
```

### 2. 合并配置

```bash
hangar merge
```

这会将所有订阅合并到 `~/.hangar/current.yaml`

### 3. 启动服务器

```bash
# 前台模式（用于测试）
hangar serve --port 8080

# 后台模式（推荐用于生产）
hangar serve --daemon --port 8080 --interval 300
```

### 4. 在 Clash 中使用

在 Clash 客户端中，添加订阅 URL：
```
http://127.0.0.1:8080/config
```

## 服务器管理

使用 `hangar-server.sh` 脚本管理后台服务：

```bash
# 启动服务
./hangar-server.sh start

# 查看状态
./hangar-server.sh status

# 查看日志
./hangar-server.sh logs

# 实时跟踪日志
./hangar-server.sh logs -f

# 停止服务
./hangar-server.sh stop

# 重启服务
./hangar-server.sh restart
```

### 自定义参数

```bash
# 启动时指定端口和自动更新间隔
./hangar-server.sh start --port 9090 --interval 600
```

## 命令参考

### 订阅管理 (`sub`)

```bash
# 添加订阅
hangar sub add <URL> --name <名称>

# 列出所有订阅
hangar sub list

# 删除订阅
hangar sub remove <ID>
```

### 配置合并 (`merge`)

```bash
# 合并所有订阅和规则到 current.yaml
hangar merge
```

### 服务器 (`serve`)

```bash
# 启动服务器
hangar serve [选项]

选项:
  -p, --port <PORT>          监听端口 [默认: 8080]
      --host <HOST>          绑定地址 [默认: 127.0.0.1]
  -i, --interval <SECONDS>   自动更新间隔（秒），0 表示禁用 [默认: 0]
  -d, --daemon               以 daemon 模式运行
  -h, --help                 显示帮助信息
```

### AI 辅助 (`ai`)

```bash
# 使用自然语言修改配置
hangar ai "添加一个新的代理组"
hangar ai "将所有美国节点添加到美国组"
```

### 历史管理 (`history`)

```bash
# 查看所有版本
hangar history list

# 回滚到指定版本
hangar history rollback <ID>

# 查看差异
hangar history diff <ID1> <ID2>
```

### 编辑配置 (`editor`)

```bash
# 用默认编辑器打开 current.yaml
hangar editor
```

### 配置 (`config`)

```bash
# 设置 LLM API Key
hangar config --api-key <KEY>

# 设置 LLM Base URL
hangar config --base-url <URL>

# 设置 LLM Model
hangar config --model <MODEL>
```

## 文件说明

### 配置文件位置

所有配置文件存储在 `~/.hangar/` 目录下：

```
~/.hangar/
├── config.yaml          # Hangar 主配置
├── subscriptions.json   # 订阅列表
├── basic.yaml          # 基础配置模板
├── groups.yaml         # 代理组配置
├── current.yaml        # 当前生成的配置（供 Clash 使用）
├── cache/              # 缓存目录
│   └── proxies/        # 下载的订阅文件
├── versions/           # 配置版本历史
├── server.log          # 服务器日志（daemon 模式）
└── server.pid          # 服务器 PID（daemon 模式）
```

### 项目文件

```
hangar/
├── install.sh           # 安装脚本
├── hangar-server.sh    # 服务器管理脚本
├── src-tauri/          # Rust 源代码
│   ├── src/
│   │   ├── main.rs     # 入口文件
│   │   ├── server.rs   # HTTP 服务器
│   │   ├── proxy.rs    # 配置合并逻辑
│   │   └── ...
│   └── resources/      # 资源文件
│       ├── basic.yml   # 基础配置示例
│       └── groups.yml  # 代理组示例
└── docs/               # 文档
```

## 高级功能

### 自动更新订阅

使用 `--interval` 参数可以让服务器自动更新订阅：

```bash
# 每 300 秒（5 分钟）自动更新一次
hangar serve --daemon --interval 300
```

### 文件监控

服务器会自动监控 `current.yaml` 的变化：
- 手动修改配置文件时，服务器会自动重新加载
- 自动更新触发的配置变更也会自动生效
- 无需重启服务器

### 生产环境部署

推荐配置：

```bash
# 1. 安装 hangar
./install.sh

# 2. 配置 LLM（如果使用 AI 功能）
hangar config --api-key "your-api-key"
hangar config --base-url "https://api.openai.com/v1"

# 3. 添加订阅
hangar sub add "https://example.com/clash" --name "主订阅"

# 4. 生成初始配置
hangar merge

# 5. 启动 daemon
./hangar-server.sh start --port 8080 --interval 3600

# 6. 设置开机自启动（可选）
# 创建 systemd service 或使用 cron @reboot
```

## 故障排查

### 服务器无法启动

```bash
# 检查端口是否被占用
lsof -i :8080

# 查看日志
./hangar-server.sh logs
```

### 配置未生效

```bash
# 重新合并配置
hangar merge

# 手动重启服务器
./hangar-server.sh restart
```

### 查看详细日志

```bash
# 实时查看日志
./hangar-server.sh logs -f

# 查看完整日志文件
cat ~/.hangar/server.log
```

## 卸载

```bash
# 删除二进制文件
sudo rm /usr/local/bin/hangar

# 删除配置文件（可选）
rm -rf ~/.hangar
```

## 开发

### 构建

```bash
cd src-tauri
cargo build --release
```

### 测试

```bash
# 运行测试
cargo test

# 运行特定测试
cargo test test_name
```

### 调试

```bash
# 使用 debug 模式运行
cd src-tauri
cargo run -- serve --port 8080
```

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

[MIT License](LICENSE)

## 相关文档

- [CLI 命令详解](cli.md)
- [文件监控功能说明](FILE_WATCH_FEATURE.md)
- [Daemon 模式更新说明](DAEMON_UPDATE_SUMMARY.md)
