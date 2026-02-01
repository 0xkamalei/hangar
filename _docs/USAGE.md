# 使用指南

## 第一步：安装依赖

确保已安装 Rust：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## 第二步：配置订阅

编辑 `subscriptions.json` 文件：

```json
{
  "subscriptions": [
    {
      "name": "魔戒",
      "url": "https://planb.mojcn.com/api/v1/client/subscribe?token=你的token",
      "enabled": true
    },
    {
      "name": "速猫",
      "url": "https://scdy03.scsub.com/apiv2/你的token?clash=1",
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
    "path": "basic.yml"
  }
}
```

### 配置说明

- `name`: 机场名称，会自动添加到每个节点名称前，例如：`[魔戒] 香港-01`
- `url`: 你的订阅链接（从机场获取）
- `enabled`: 是否启用此订阅（true/false）

## 第三步：准备基础配置

确保 `basic.yml` 文件存在。这个文件包含：
- DNS 配置
- 端口设置
- 自定义规则
- 其他 Clash 配置

程序会保留这些配置，并添加：
- 所有订阅的代理节点
- 自动生成的代理组
- Loyalsoldier 规则集

## 第四步：运行程序

```bash
# 开发模式运行
cargo run

# 或者编译后运行
cargo build --release
./target/release/proxy-sub-manager
```

## 第五步：使用订阅

程序启动后会显示：

```
🚀 代理订阅管理器启动中...

📡 正在获取订阅: 魔戒
   ✓ 获取到 45 个节点
📡 正在获取订阅: 速猫
   ✓ 获取到 82 个节点

📊 共获取 127 个代理节点
🌍 地区分组: ["HK", "TW", "JP", "SG", "US", "UK"]
🎯 创建了 6 个服务专用组

⚙️  正在合并配置...
✓ 配置已保存到: output_config.yaml

🌐 正在启动 HTTP 服务器...
   地址: http://127.0.0.1:8080
   订阅链接: http://127.0.0.1:8080/config
```

## 第六步：在 Clash Verge 中使用

1. 打开 Clash Verge
2. 点击「订阅」
3. 添加订阅：`http://127.0.0.1:8080/config`
4. 更新订阅
5. 选择代理组和节点

## 代理组说明

程序会自动创建以下代理组：

### 节点选择
包含所有节点，可手动选择任意节点。

### ChatGPT
优选以下地区的节点：
- 🇺🇸 美国
- 🇬🇧 英国
- 🇸🇬 新加坡
- 🇹🇼 台湾

### Gemini
优选以下地区的节点：
- 🇺🇸 美国
- 🇬🇧 英国
- 🇸🇬 新加坡
- 🇭🇰 香港
- 🇹🇼 台湾

### Google
包含所有节点。

### Netflix
包含所有节点，可根据需要选择特定地区。

### Telegram
包含所有节点。

## 规则说明

程序使用 [Loyalsoldier/clash-rules](https://github.com/Loyalsoldier/clash-rules) 规则集：

### 直连规则
- 中国大陆域名和 IP
- 局域网地址
- Apple 中国服务
- 私有网络

### 代理规则
- GFW 列表
- 国际服务
- Google 服务
- Telegram

### 拒绝规则
- 广告域名
- 追踪域名

## 高级配置

### 修改服务器端口

编辑 `subscriptions.json`：

```json
{
  "server": {
    "port": 9090,
    "host": "127.0.0.1"
  }
}
```

### 添加更多服务组

编辑 `src/proxy.rs` 中的 `create_service_groups` 函数：

```rust
ProxyGroup {
    name: "YouTube".to_string(),
    group_type: "select".to_string(),
    proxies: all_proxy_names.clone(),
    extra: HashMap::new(),
},
```

### 自定义规则

在 `basic.yml` 中添加自定义规则：

```yaml
rules:
  - DOMAIN-SUFFIX,example.com,DIRECT
  - DOMAIN-KEYWORD,google,节点选择
```

## 故障排除

### 订阅获取失败

1. 检查订阅链接是否正确
2. 确认网络连接正常
3. 查看机场是否正常运行

### 配置文件错误

1. 确保 `basic.yml` 是有效的 YAML 格式
2. 检查 `subscriptions.json` 是否是有效的 JSON

### 服务器启动失败

1. 检查端口是否被占用
2. 尝试更换端口号

## 更新订阅

程序运行时会自动从订阅源获取最新节点。如需更新：

1. 停止程序（Ctrl+C）
2. 重新运行程序
3. 在 Clash Verge 中更新订阅

## 注意事项

1. 保持程序运行，Clash Verge 才能访问订阅链接
2. 订阅链接仅在本机可用（127.0.0.1）
3. 规则集会自动从 CDN 下载，首次使用需要网络连接
4. 建议定期更新订阅以获取最新节点
