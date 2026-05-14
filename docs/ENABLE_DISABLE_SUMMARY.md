# Enable/Disable 功能实现总结

## 📝 概述

为 `hangar sub` 命令添加了 `enable` 和 `disable` 两个新子命令，允许用户临时禁用或启用已添加的订阅，而无需删除它们。

## ✨ 新增功能

### 1. 命令接口

#### `hangar sub enable <id>`
- 启用指定的订阅
- 参数可以是订阅的 UUID 或索引（从0开始）
- 启用后的订阅会参与 merge 操作和自动更新

#### `hangar sub disable <id>`
- 禁用指定的订阅
- 参数可以是订阅的 UUID 或索引（从0开始）
- 禁用后的订阅会被跳过，不参与 merge 和自动更新

### 2. List 命令增强

`hangar sub list` 现在会显示每个订阅的启用状态：

```
ID                                   Name                 Nodes      Enabled
a1b2c3d4-5678-90ab-cdef-1234567890ab 我的订阅A            50         ✓
f9e8d7c6-5432-10ba-fedc-0987654321ba 我的订阅B            30         ✗
```

- ✓ 表示已启用
- ✗ 表示已禁用

## 🔧 技术实现

### 1. 数据模型
`Subscription` 结构体已经包含 `enabled: bool` 字段（之前已存在）：

```rust
pub struct Subscription {
    pub id: String,
    pub name: String,
    pub url: String,
    pub enabled: bool,  // 控制启用/禁用状态
    pub last_updated: Option<String>,
    pub node_count: Option<usize>,
}
```

### 2. CLI 命令定义
在 `src-tauri/src/main.rs` 中的 `SubCommands` 枚举添加了两个新变体：

```rust
enum SubCommands {
    Add { ... },
    List,
    Remove { id: String },
    Enable { id: String },   // 新增
    Disable { id: String },  // 新增
}
```

### 3. 命令处理逻辑
两个命令的实现逻辑相似，都包括：

1. 加载订阅列表
2. 通过 ID 或索引查找订阅
3. 修改 `enabled` 字段
4. 保存订阅列表
5. 输出操作结果

### 4. 自动更新集成
在 `serve` 命令的自动更新循环中（已存在的代码）：

```rust
for sub in &subs {
    if sub.enabled {  // 只更新启用的订阅
        let _ = subscription::download_subscription(sub).await;
    }
}
```

### 5. Merge 操作集成
`proxy::merge_configs()` 函数会过滤掉禁用的订阅（需要在 `proxy.rs` 中确认此逻辑）。

## 📄 文档更新

### 1. 新增文档
- **`SUBSCRIPTION_ENABLE_DISABLE.md`**: 详细的使用指南
  - 命令使用说明
  - 行为说明
  - 工作流示例
  - 注意事项

### 2. 更新的文档
- **`cli.md`**: 添加了 `enable` 和 `disable` 命令说明
- **`QUICKSTART.md`**: 添加了场景 3.5，展示如何使用这些命令

### 3. 测试脚本
- **`test-enable-disable.sh`**: 自动化测试脚本，演示新功能

## 🎯 使用场景

### 场景 1: 测试新订阅
禁用现有订阅，添加并测试新订阅，确认后再重新启用旧订阅。

### 场景 2: 节省流量
临时禁用不常用的订阅，减少自动更新时的网络流量。

### 场景 3: 订阅切换
在多个订阅源之间快速切换，无需反复添加和删除。

### 场景 4: 故障排查
当某个订阅出现问题时，可以先禁用它，继续使用其他订阅。

## ✅ 验证清单

- [x] 代码编译通过（`cargo check`）
- [x] 添加了 `enable` 和 `disable` 命令
- [x] 更新了 `list` 命令显示启用状态
- [x] 支持通过 ID 或索引操作
- [x] 状态持久化到 `subscriptions.json`
- [x] 自动更新只处理启用的订阅
- [x] 文档已更新
- [x] 创建了使用示例和测试脚本

## 🚀 后续建议

1. **批量操作**: 考虑添加批量 enable/disable 命令
   ```bash
   hangar sub enable-all
   hangar sub disable-all
   ```

2. **状态过滤**: 在 list 命令中添加过滤选项
   ```bash
   hangar sub list --enabled
   hangar sub list --disabled
   ```

3. **默认状态配置**: 允许用户设置新添加订阅的默认启用状态

4. **UI 增强**: 在表格输出中使用颜色区分启用和禁用的订阅

## 📦 文件清单

### 修改的文件
- `src-tauri/src/main.rs` - 添加命令和实现逻辑

### 更新的文档
- `cli.md` - 命令参考
- `QUICKSTART.md` - 快速开始指南

### 新增的文件
- `SUBSCRIPTION_ENABLE_DISABLE.md` - 详细使用指南
- `test-enable-disable.sh` - 测试脚本
- `ENABLE_DISABLE_SUMMARY.md` - 本文档

## 🎉 总结

成功为 Hangar 添加了订阅启用/禁用功能，用户现在可以更灵活地管理订阅，而无需频繁添加和删除。这个功能与现有的自动更新和配置合并机制完美集成，提供了更好的用户体验。
