# 命令行工具使用指南

## 概述

命令行工具 `cli` 提供快速的订阅合并测试功能，无需启动 UI 即可生成 Clash 配置文件。

## 使用方法

### 基本用法

```bash
# 使用 subs.txt 生成 clash.yml
cargo run --bin cli -- subs.txt

# 指定输出文件
cargo run --bin cli -- subs.txt my_config.yml
```

### 输入文件格式

创建一个文本文件（如 `subs.txt`），每行一个订阅链接：

```
https://example1.com/subscribe?token=xxx
https://example2.com/subscribe?token=yyy
# 这是注释，会被忽略
https://example3.com/subscribe?token=zzz
```

### 输出示例

```
🚀 代理订阅合并工具

📄 读取订阅文件: subs.txt
✓ 找到 3 个订阅

📡 获取订阅: 机场1
  ✓ 获取到 77 个节点
📡 获取订阅: 机场2
  ✓ 获取到 45 个节点
📡 获取订阅: 机场3
  ✓ 获取到 32 个节点

📊 统计信息:
  总节点数: 154

🌍 地区分布:
  HK: 45 个节点
  US: 28 个节点
  JP: 22 个节点
  TW: 20 个节点
  SG: 18 个节点
  UK: 10 个节点
  ...

🎯 创建了 10 个地区分组
🎯 创建了 3 个服务专用组

✅ 配置已保存到: clash.yml
```

## 功能特性

### 1. 自动地区识别

工具会自动识别节点名称中的地区信息：

**支持的地区（中英文）：**
- 香港 / HK
- 台湾 / TW / 台
- 日本 / JP
- 新加坡 / SG / 狮城
- 美国 / US
- 英国 / UK
- 韩国 / KR
- 德国 / DE
- 加拿大 / CA
- 印度 / IN
- 马来西亚 / MY
- 土耳其 / TR
- 阿根廷 / AR
- 俄罗斯 / RU
- 越南 / VN
- 乌克兰 / UA
- 尼日利亚 / NG

### 2. 智能分组

**地区分组：**
- 根据识别的地区自动创建分组
- 示例：HK 地区、TW 地区、US 地区等

**服务专用组：**
1. **节点选择** - 包含所有节点
2. **ChatGPT** - 优选美国、英国、新加坡、台湾节点
3. **Gemini** - 优选美国、英国、新加坡、香港、台湾节点

### 3. 节点命名

所有节点会自动添加机场前缀：

```
原始名称: 香港-01
处理后: [机场1] 香港-01
```

这样可以轻松识别节点来源。

## 配置文件结构

生成的 `clash.yml` 包含：

```yaml
port: 7890
socks-port: 7891
allow-lan: false
mode: Rule
log-level: info

proxies:
  - name: '[机场1] 香港-01'
    type: vmess
    server: xxx.com
    port: 443
    # ... 其他配置

proxy-groups:
  - name: HK 地区
    type: select
    proxies:
      - '[机场1] 香港-01'
      - '[机场1] 香港-02'
      # ...
  
  - name: 节点选择
    type: select
    proxies:
      # 所有节点
  
  - name: ChatGPT
    type: select
    proxies:
      # US、UK、SG、TW 节点
  
  - name: Gemini
    type: select
    proxies:
      # US、UK、SG、HK、TW 节点

rules:
  - DOMAIN-SUFFIX,google.com,节点选择
  - DOMAIN-KEYWORD,openai,ChatGPT
  - DOMAIN-KEYWORD,gemini,Gemini
  - MATCH,DIRECT
```

## 测试工作流程

### 快速迭代测试

```bash
# 1. 修改代码
vim src-tauri/src/bin/cli.rs

# 2. 快速测试
cargo run --bin cli -- subs.txt test.yml

# 3. 查看结果
cat test.yml | grep "name:" | head -20

# 4. 导入 Clash 测试连接
```

### 对比UI版本

```bash
# 命令行生成
cargo run --bin cli -- subs.txt cli_output.yml

# UI 生成
# 启动应用 -> 启动服务器
# 访问 http://127.0.0.1:8080/config > ui_output.yml

# 对比
diff cli_output.yml ui_output.yml
```

## 常见问题

### Q: 订阅获取失败？
**A:** 检查以下几点：
1. 订阅链接是否有效
2. 网络连接是否正常
3. 订阅服务器是否可访问

### Q: 地区识别不准确？
**A:** 在 `cli.rs` 的 `extract_region` 函数中添加更多模式：

```rust
let regions = vec![
    ("香港", "HK"), ("HK", "HK"),
    ("港", "HK"), // 添加新模式
    // ...
];
```

### Q: 需要自定义分组？
**A:** 修改 `create_service_groups` 函数：

```rust
// 添加 Netflix 组
let netflix_regions = ["US", "SG", "JP"];
let netflix_proxies: Vec<String> = proxies
    .iter()
    .filter(|p| {
        p.region
            .as_ref()
            .map(|r| netflix_regions.contains(&r.as_str()))
            .unwrap_or(false)
    })
    .map(|p| p.name.clone())
    .collect();

if !netflix_proxies.is_empty() {
    groups.push(ProxyGroup {
        name: "Netflix".to_string(),
        group_type: "select".to_string(),
        proxies: netflix_proxies,
        extra: HashMap::new(),
    });
}
```

### Q: 如何添加更多规则？
**A:** 在 `main` 函数中修改 `rules` 数组：

```rust
let config = ClashConfig {
    // ...
    rules: vec![
        "DOMAIN-SUFFIX,google.com,节点选择".to_string(),
        "DOMAIN-KEYWORD,openai,ChatGPT".to_string(),
        "DOMAIN-KEYWORD,gemini,Gemini".to_string(),
        "DOMAIN-KEYWORD,netflix,Netflix".to_string(), // 新增
        "DOMAIN-SUFFIX,youtube.com,节点选择".to_string(), // 新增
        "MATCH,DIRECT".to_string(),
    ],
};
```

## 性能优势

相比 UI 版本，命令行工具的优势：

1. **快速测试** - 无需启动完整应用
2. **批量处理** - 可以编写脚本批量生成配置
3. **CI/CD 集成** - 可集成到自动化流程
4. **调试友好** - 直接看到详细输出

## 编译为独立可执行文件

```bash
# 编译发布版本
cargo build --bin cli --manifest-path src-tauri/Cargo.toml --release

# 可执行文件位置
src-tauri/target/release/cli

# 使用
./src-tauri/target/release/cli subs.txt clash.yml
```

## License

MIT
