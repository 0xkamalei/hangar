# 订阅管理：Enable/Disable 命令使用指南

## 概述

新增的 `enable` 和 `disable` 命令允许你临时禁用或重新启用已添加的订阅，而无需删除它们。这在以下场景非常有用：

- 临时测试特定的订阅源
- 节省订阅更新时的网络流量
- 在多个订阅之间切换

## 命令使用

### 查看订阅列表

```bash
hangar sub list
```

输出示例：
```
ID                                   Name                 Nodes      Enabled
a1b2c3d4-5678-90ab-cdef-1234567890ab speedcat             50         ✓
f9e8d7c6-5432-10ba-fedc-0987654321ba naiyun               30         ✗
```

### 禁用订阅

**推荐方式（使用 name）：**
```bash
hangar sub disable speedcat
```

也可以使用 ID：
```bash
hangar sub disable a1b2c3d4-5678-90ab-cdef-1234567890ab
```

或使用索引（从0开始）：
```bash
hangar sub disable 0
```

输出：
```
✅ Disabled subscription: speedcat (a1b2c3d4-5678-90ab-cdef-1234567890ab)
```

### 启用订阅

**推荐方式（使用 name）：**
```bash
hangar sub enable speedcat
```

也可以使用 ID：
```bash
hangar sub enable a1b2c3d4-5678-90ab-cdef-1234567890ab
```

或使用索引：
```bash
hangar sub enable 0
```

输出：
```
✅ Enabled subscription: speedcat (a1b2c3d4-5678-90ab-cdef-1234567890ab)
```

## 行为说明

### Merge 操作
运行 `hangar merge` 时，**只有启用的订阅**会被合并到 `current.yaml` 中。禁用的订阅会被跳过。

### Serve 自动更新
当使用 `hangar serve --interval <seconds>` 启动服务器时，自动更新仅会下载和合并**已启用的订阅**。禁用的订阅不会被更新。

### 订阅缓存
禁用订阅不会删除其缓存文件（位于 `~/.hangar/cache/proxies/` 目录）。重新启用订阅后，如果需要最新数据，可以重新下载：

```bash
hangar sub enable <id>
hangar merge  # 触发合并以使用最新配置
```

## 工作流示例

### 场景1：测试新订阅

```bash
# 1. 禁用所有现有订阅（使用 name）
hangar sub disable speedcat
hangar sub disable naiyun

# 2. 添加新订阅（默认启用）
hangar sub add https://new-subscription-url --name "测试订阅"

# 3. 合并并测试
hangar merge
hangar serve --port 8080

# 4. 如果测试通过，重新启用之前的订阅
hangar sub enable speedcat
hangar sub enable naiyun
hangar merge
```

### 场景2：临时切换订阅源

```bash
# 当前使用 speedcat，想切换到 naiyun
hangar sub disable speedcat
hangar sub enable naiyun
hangar merge  # 重新生成配置
```

## 注意事项

1. **推荐使用 name**：订阅名称短小易记，推荐作为首选方式。命令会按以下顺序查找：
   - 首先匹配订阅名称（name）
   - 其次匹配完整 UUID
   - 最后尝试解析为索引
2. **name 的唯一性**：建议为每个订阅设置不同的名称，避免混淆
3. **状态持久化**：启用/禁用状态会保存在 `~/.hangar/subscriptions.json` 中
4. **不影响删除**：`hangar sub remove` 命令仍然会完全删除订阅及其配置

## 命令参考

| 命令 | 参数 | 说明 |
|------|------|------|
| `hangar sub list` | - | 列出所有订阅及其状态 |
| `hangar sub enable` | `<name\|id\|index>` | 启用指定订阅（推荐使用 name） |
| `hangar sub disable` | `<name\|id\|index>` | 禁用指定订阅（推荐使用 name） |
| `hangar sub add` | `<url> [--name <name>]` | 添加新订阅（默认启用） |
| `hangar sub remove` | `<name\|id\|index>` | 删除订阅 |
