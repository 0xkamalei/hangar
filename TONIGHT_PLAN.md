# 今晚执行计划

## ✅ 已完成 - 关键Bug修复

### 1. 服务器启动崩溃 (FIXED)
**问题**: 点击"启动服务器"后应用直接退出  
**原因**: 
- `tokio::signal::ctrl_c()` 在 GUI 应用中不适用
- 配置文件路径在打包后找不到
- Tauri 2.0 API 使用不正确

**修复**:
```rust
// src-tauri/src/lib.rs
- 新增 get_config_path() 函数，智能处理开发/生产环境
- 自动创建应用数据目录
- 如果配置不存在，自动创建默认配置

// src-tauri/src/server.rs  
- 移除 tokio::signal::ctrl_c()
- 服务器直接运行在 tokio::spawn 中，可被 abort() 取消
```

### 2. 删除按钮无响应 (FIXED)
**问题**: 点击删除按钮完全没有反应  
**原因**: Tauri command 签名不匹配 - 前端传递参数但后端缺少 `app_handle`

**修复**:
```rust
// 所有 CRUD 命令都添加了 app_handle 参数
- get_subscriptions(app_handle: tauri::AppHandle)
- add_subscription(app_handle: tauri::AppHandle, ...)
- update_subscription(app_handle: tauri::AppHandle, ...)
- delete_subscription(app_handle: tauri::AppHandle, ...)
```

### 3. UI 消息可见性 (FIXED)
**问题**: 成功消息文字看不见，提示不会消失

**修复** (`src/App.tsx`):
- 深绿色文字 (#065f46) + 浅绿色背景 (#d1fae5)
- 深红色文字 (#991b1b) + 浅红色背景 (#fee2e2)
- 3秒自动隐藏消息
- 移除 placeholder 文字

---

## 🎯 今晚优化任务

### 第一阶段: 必须完成 (2小时)

#### 1. 测试修复并验证 (30分钟) ⭐⭐⭐
```bash
bun run dev
```

**测试清单**:
- [ ] 添加订阅 - 确认可以添加
- [ ] 编辑订阅 - 确认可以修改
- [ ] **删除订阅 - 确认按钮有响应** ⭐
- [ ] 启用/禁用订阅 - 确认状态改变
- [ ] **启动服务器 - 确认应用不崩溃** ⭐
- [ ] 访问 http://127.0.0.1:8080/config - 确认能看到配置
- [ ] 停止服务器 - 确认正常停止

**如果发现问题**: 立即反馈，优先修复

#### 2. basic.yml 文件处理 (30分钟) ⭐⭐⭐
**问题**: 打包后找不到 basic_test.yml

**解决方案**:
```rust
// 修改 get_config_path() 同时处理 basic.yml
fn get_basic_config_path(app_handle: &tauri::AppHandle) -> Result<String, String> {
    // 1. 检查应用数据目录
    // 2. 如果不存在，从资源复制默认 basic.yml
    // 3. 返回路径
}
```

**文件位置**:
- `src-tauri/src/lib.rs` - 新增函数
- `src-tauri/resources/basic.yml` - 内嵌默认配置

#### 3. 网络请求超时 (30分钟) ⭐⭐
**位置**: `src-tauri/src/subscription.rs`

```rust
// 添加超时
use tokio::time::timeout;
use std::time::Duration;

pub async fn fetch_subscription(url: &str) -> anyhow::Result<String> {
    let future = reqwest::get(url).send();
    let response = timeout(Duration::from_secs(30), future)
        .await
        .map_err(|_| anyhow!("请求超时(30秒)"))?  
        .map_err(|e| anyhow!("网络错误: {}", e))?;
    
    let body = response.text().await?;
    Ok(body)
}
```

#### 4. 订阅测试功能 (30分钟) ⭐⭐⭐
**位置**: 
- `src-tauri/src/lib.rs` - 新增 Tauri command
- `src/App.tsx` - 添加"测试"按钮

```rust
#[tauri::command]
async fn test_subscription(url: String) -> Result<String, String> {
    // 1. 获取订阅内容
    // 2. 解析 YAML
    // 3. 返回节点数量和预览信息
    // 4. 不保存到配置
}
```

**UI 改进**:
- 添加表单中增加"测试"按钮
- 显示测试结果（节点数、区域分布）
- 测试通过后才允许保存

---

### 第二阶段: 推荐完成 (1小时)

#### 5. 显示订阅详细信息 (30分钟) ⭐⭐
**位置**: `src/App.tsx`

扩展 Subscription 接口:
```typescript
interface Subscription {
  name: string;
  url: string;
  enabled: boolean;
  // 新增字段
  lastUpdate?: string;     // 最后更新时间
  nodeCount?: number;      // 节点数量
  status?: 'success' | 'error' | 'pending'; // 状态
}
```

**UI 改进**:
```tsx
<div className="subscription-card">
  <div>
    <h4>{sub.name}</h4>
    {sub.nodeCount && <span>{sub.nodeCount} 个节点</span>}
    {sub.lastUpdate && <span>更新于 {sub.lastUpdate}</span>}
  </div>
</div>
```

#### 6. 服务器状态增强 (20分钟) ⭐⭐
**位置**: `src/App.tsx`

```tsx
interface ServerInfo {
  running: boolean;
  activeSubscriptions: number;
  totalNodes: number;
  uptime?: number;  // 运行时长(秒)
}

// 显示
<div>
  <p>活跃订阅: {serverInfo.activeSubscriptions}</p>
  <p>总节点数: {serverInfo.totalNodes}</p>
  <p>运行时长: {formatUptime(serverInfo.uptime)}</p>
</div>
```

#### 7. 一键复制订阅链接 (10分钟) ⭐
**位置**: `src/App.tsx`

```tsx
function copySubscriptionUrl() {
  const url = `http://127.0.0.1:8080/config`;
  navigator.clipboard.writeText(url);
  showStatus("✅ 订阅链接已复制");
}

// UI
<button onClick={copySubscriptionUrl}>
  📋 复制订阅链接
</button>
```

---

### 第三阶段: 可选完成 (1小时)

#### 8. 加载状态和进度 (20分钟)
- 启动服务器时显示 spinner
- 获取订阅时显示进度
- 使用 CSS 动画

#### 9. 配置备份 (20分钟)
- 保存前自动备份
- 保留最近 3 个备份
- 提供恢复功能

#### 10. 批量操作 (20分钟)
- 全选/全不选
- 批量启用/禁用
- 批量删除

---

## 📝 执行流程

### 准备工作 (5分钟)
```bash
# 1. 清理环境
pkill -f tauri || true
pkill -f vite || true

# 2. 确认编译通过
./test-critical-fixes.sh

# 3. 启动开发服务器
bun run dev
```

### 迭代开发
1. **选择一个任务** - 从第一阶段开始
2. **编写代码** - 小步实现
3. **立即测试** - 在开发模式测试
4. **运行检查** - `bun run tsc --noEmit` + `cargo check`
5. **提交代码** - 单个功能单独提交
6. **继续下一个**

### 每小时检查点
- [ ] 第1小时结束: 完成第一阶段任务 1-2
- [ ] 第2小时结束: 完成第一阶段任务 3-4
- [ ] 第3小时结束: 完成第二阶段任务 5-7
- [ ] 第4小时结束: 根据时间完成第三阶段

---

## ✅ 验收标准

### 核心功能
- [ ] 开发模式正常运行
- [ ] 打包后正常运行
- [ ] 启动服务器不崩溃
- [ ] 删除订阅正常工作
- [ ] 配置文件正确保存和读取

### 用户体验
- [ ] 有明确的加载提示
- [ ] 错误消息清晰易懂
- [ ] 消息自动消失
- [ ] 可以测试订阅链接

### 代码质量
- [ ] TypeScript 类型检查通过
- [ ] Rust 编译无错误
- [ ] 所有测试通过
- [ ] 无严重 Clippy 警告

---

## 🚨 如果遇到问题

1. **编译错误** - 立即停止，先修复编译问题
2. **运行时崩溃** - 添加 `eprintln!()` 日志定位
3. **功能不工作** - 检查 Tauri command 签名
4. **时间不够** - 只完成第一阶段的必须任务

---

## 📊 优先级说明

- ⭐⭐⭐ 高优先级 - 必须完成
- ⭐⭐ 中优先级 - 推荐完成  
- ⭐ 低优先级 - 时间充裕时完成

---

## 🎯 开始吧！

现在运行：
```bash
bun run dev
```

验证修复是否生效，然后我们开始优化！
