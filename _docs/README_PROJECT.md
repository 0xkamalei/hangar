# 代理订阅管理器

基于 Rust 的代理订阅合并管理工具，支持多订阅源合并、智能分组和规则集成。

## 功能特性

- ✅ 多订阅源管理和合并
- ✅ 自动按地区分组代理节点
- ✅ 节点名称自动添加机场标识
- ✅ 为常用服务创建专用组（ChatGPT、Gemini、Google 等）
- ✅ 集成 Loyalsoldier/clash-rules 规则集
- ✅ 合并自定义 basic.yml 规则
- ✅ 内置 HTTP 服务器提供订阅链接
- ✅ 支持 Clash Verge 和其他 Clash Premium 客户端

## 快速开始

### 1. 配置订阅

编辑 `subscriptions.json` 文件，添加你的订阅地址：

```json
{
  "subscriptions": [
    {
      "name": "机场A",
      "url": "你的订阅链接",
      "enabled": true
    }
  ]
}
```

### 2. 准备基础配置

确保 `basic.yml` 文件存在，包含你的基础 Clash 配置（DNS、端口等）。

### 3. 运行程序

```bash
cargo run --release
```

### 4. 使用订阅

程序启动后会在 `http://127.0.0.1:8080/config` 提供合并后的订阅链接。

在 Clash Verge 中添加此订阅链接即可使用。

## 配置说明

### 订阅配置 (subscriptions.json)

- `name`: 机场名称，会添加到节点名称前
- `url`: 订阅链接
- `enabled`: 是否启用此订阅

### 基础配置 (basic.yml)

包含 Clash 的基础配置，如：
- 端口设置
- DNS 配置
- 自定义规则
- 其他 Clash 配置项

程序会保留这些配置，并添加：
- 合并后的代理节点
- 自动生成的代理组
- Loyalsoldier 规则集

## 代理组说明

程序会自动创建以下代理组：

- **节点选择**: 包含所有节点
- **ChatGPT**: 优选美国、英国、新加坡、台湾节点
- **Gemini**: 优选美国、英国、新加坡、香港、台湾节点
- **Google**: 包含所有节点
- **Netflix**: 包含所有节点
- **Telegram**: 包含所有节点

## 规则集

使用 [Loyalsoldier/clash-rules](https://github.com/Loyalsoldier/clash-rules) 规则集：

- 直连域名列表 (direct)
- 代理域名列表 (proxy)
- 广告域名列表 (reject)
- Apple/iCloud 域名
- Google 域名
- GFW 列表
- 中国大陆 IP (cncidr)
- Telegram IP (telegramcidr)
- 等等...

## 技术栈

- Rust
- Tokio (异步运行时)
- Axum (HTTP 服务器)
- Serde (序列化/反序列化)
- Reqwest (HTTP 客户端)

## 开发

```bash
# 构建
cargo build

# 运行
cargo run

# 发布构建
cargo build --release
```

## 注意事项

1. 确保订阅链接有效且可访问
2. 基础配置文件 `basic.yml` 必须是有效的 YAML 格式
3. 服务器默认监听 `127.0.0.1:8080`，可在代码中修改
4. 规则集会自动从 CDN 下载，需要网络连接

## 许可证

MIT
