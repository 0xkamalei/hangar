# Node Count 功能改进

## 🐛 问题描述

在执行 `hangar sub add` 添加订阅后，虽然订阅被成功下载，但是 `node_count` 字段始终显示为 0。这是因为在添加订阅时，代码只是下载了订阅文件，但没有解析它来统计节点数量。

## ✅ 解决方案

### 1. 新增函数：`count_proxies`

在 `src-tauri/src/subscription.rs` 中添加了 `count_proxies` 函数：

```rust
/// Count proxies in a subscription's cached YAML file
pub fn count_proxies(subscription_id: &str) -> Result<usize> {
    let cache_path = crate::storage::get_subscription_cache_path(subscription_id)?;
    
    if !cache_path.exists() {
        return Ok(0);
    }

    let content = std::fs::read_to_string(&cache_path)?;
    
    // Parse YAML to extract proxies
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(&content)?;
    
    if let Some(proxies) = yaml_value.get("proxies") {
        if let Some(proxy_array) = proxies.as_sequence() {
            return Ok(proxy_array.len());
        }
    }
    
    Ok(0)
}
```

**功能说明：**
- 读取订阅的缓存 YAML 文件
- 解析 YAML 获取 `proxies` 数组
- 返回数组长度（即节点数量）

### 2. 更新 `add` 命令

在 `src-tauri/src/main.rs` 的 `SubCommands::Add` 处理中：

**修改前：**
```rust
let new_sub = types::Subscription {
    // ...
    node_count: None,  // 始终为 None
};

match subscription::download_subscription(&new_sub).await {
    Ok(path) => {
        println!("✅ Downloaded to {:?}", path);
        // 没有统计节点
    }
    // ...
}
```

**修改后：**
```rust
let mut new_sub = types::Subscription {  // 改为 mut
    // ...
    node_count: None,
};

match subscription::download_subscription(&new_sub).await {
    Ok(path) => {
        println!("✅ Downloaded to {:?}", path);
        
        // 解析并统计节点
        match subscription::count_proxies(&id) {
            Ok(count) => {
                new_sub.node_count = Some(count);  // 更新节点数
                println!("   Found {} nodes", count);
            }
            Err(e) => {
                println!("⚠️ Failed to count proxies: {}", e);
            }
        }
    }
    // ...
}
```

## 📝 工作流程

现在 `hangar sub add` 的完整流程：

1. 创建订阅对象（`node_count` 初始为 `None`）
2. 下载订阅内容到 `~/.hangar/cache/proxies/<id>.yaml`
3. **【新增】** 解析 YAML 文件，统计 `proxies` 数组长度
4. **【新增】** 更新订阅对象的 `node_count` 字段
5. 保存订阅列表到 `subscriptions.json`

## 🎯 效果对比

### 修改前
```bash
$ cargo run sub list
ID                                   Name                 Nodes      Enabled
65c32c53-17f7-4573-9077-b80e62ff6100 speedcat             0          ✓
```

### 修改后
```bash
$ cargo run sub add "https://example.com/sub" --name "测试订阅"
📥 Downloading subscription: 测试订阅...
✅ Downloaded to "/Users/xxx/.hangar/cache/proxies/<id>.yaml"
   Found 50 nodes
✅ Added subscription: 测试订阅 (<id>)

$ cargo run sub list
ID                                   Name                 Nodes      Enabled
65c32c53-17f7-4573-9077-b80e62ff6100 测试订阅             50         ✓
```

## 🔧 技术细节

### YAML 解析
使用 `serde_yaml` 库解析订阅文件：
- 首先将整个文件解析为 `serde_yaml::Value`
- 查找 `proxies` 字段
- 如果是数组类型，返回其长度

### 错误处理
- 如果缓存文件不存在，返回 0
- 如果 YAML 解析失败，返回错误但不中断添加流程
- 用户会看到警告信息，但订阅仍然会被添加

### 性能考虑
- 只解析 YAML 结构，不反序列化完整的代理对象
- 使用 `serde_yaml::Value` 直接访问，避免不必要的类型转换

## ✅ 验证

编译测试通过：
```bash
$ cargo check
    Checking hangar v0.1.0 (/Users/lei/dev/personal/hangar/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.41s
```

## 📦 相关文件

### 修改的文件
- `src-tauri/src/subscription.rs` - 新增 `count_proxies` 函数
- `src-tauri/src/main.rs` - 更新 `add` 命令逻辑

### 文档
- `NODE_COUNT_FIX.md` - 本文档

## 🚀 后续优化建议

1. **批量更新节点数**：添加命令来更新所有已存在订阅的节点数
   ```bash
   hangar sub refresh-counts
   ```

2. **merge 时更新**：在 `merge` 操作时也更新节点数，确保数据始终为最新

3. **显示更多信息**：在 `list` 命令中显示最后更新时间

4. **缓存过期机制**：如果缓存文件太旧，自动重新下载并更新节点数
