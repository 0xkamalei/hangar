# 调试指南 - Debug Guide

## 修复内容

### 1. 删除按钮修复
- ✅ 添加了详细的日志输出
- ✅ 前端和后端都有完整的调试信息
- ✅ 确保参数正确传递

### 2. 服务器崩溃修复
- ✅ 使用 `oneshot` 通道实现优雅关闭
- ✅ 替换了 `task.abort()` 方法（会导致 panic）
- ✅ 使用 `axum::serve().with_graceful_shutdown()`
- ✅ 添加了详细的错误日志

## 如何测试

### 方法 1: 使用测试脚本（推荐）

```bash
./test-debug.sh
```

这会启动应用并显示所有调试信息。

### 方法 2: 手动启动

```bash
bun run tauri dev
```

## 测试步骤

### 测试删除功能

1. 启动应用
2. 打开浏览器开发者工具（在应用窗口右键 -> Inspect Element）
3. 切换到 Console 标签页
4. 点击任意订阅的「删除」按钮
5. 确认删除对话框

**期望结果：**

前端控制台输出：
```
🔍 准备删除订阅，index: 0
✓ 用户确认删除，调用 delete_subscription
✓ 删除成功: ✅ 订阅删除成功
```

终端输出：
```
🔍 delete_subscription 被调用，index: 0
✓ 配置路径: /path/to/subscriptions.json
✓ 当前订阅数量: 2
✓ 已删除订阅: 订阅名称
✓ 配置已保存
```

### 测试服务器启动

1. 启动应用
2. 打开浏览器开发者工具
3. 点击「启动服务器」按钮

**期望结果：**

前端控制台输出：
```
🔍 开始启动服务器...
✓ 调用 start_proxy_server 命令
✓ 服务器启动成功: ✅ 服务器已启动...
✓ startServer 函数执行完毕
```

终端输出（详细步骤）：
```
🔍 start_proxy_server 被调用
✓ 使用配置文件: /path/to/subscriptions.json
✓ 配置加载成功，订阅数量: 1
🔍 尝试加载基础配置: basic_test.yml
✓ 基础配置加载成功: basic_test.yml
✓ 开始合并配置...
✓ 配置合并成功
✓ 配置已保存到: output_config.yaml
✓ 即将启动服务器: 127.0.0.1:8080
🚀 服务器任务已启动
✅ 服务器启动命令执行成功
🔍 start_server 函数被调用: 127.0.0.1:8080
✓ AppState 创建成功
✓ Router 创建成功
🔍 尝试绑定地址: 127.0.0.1:8080
✓ TcpListener 绑定成功

🌐 正在启动 HTTP 服务器...
   地址: http://127.0.0.1:8080
   订阅链接: http://127.0.0.1:8080/config

✨ 服务器已启动，等待请求...
```

4. 测试访问：在浏览器访问 `http://127.0.0.1:8080/config`
5. 应该能看到 YAML 配置内容

### 测试服务器停止

1. 服务器运行时，点击「停止服务器」按钮

**期望结果：**

终端输出：
```
🔍 stop_proxy_server 被调用
✓ 发送关闭信号
✅ 服务器停止命令执行成功
🛑 收到关闭信号，开始优雅关闭...
✓ 服务器已优雅关闭
✓ 服务器状态已更新为停止
```

## 如果还有问题

### 情况 1: 删除按钮仍然无效

**检查：**
1. 前端控制台有任何输出吗？
2. 终端有任何输出吗？
3. 是否显示了确认对话框？

**如果没有任何输出：**
- 可能是按钮点击事件没有绑定
- 检查浏览器控制台是否有 JavaScript 错误

**如果有前端输出但没有后端输出：**
- Tauri 命令可能没有正确注册
- 检查 `lib.rs` 中的 `invoke_handler` 是否包含 `delete_subscription`

### 情况 2: 服务器仍然崩溃

**检查崩溃发生在哪个阶段：**

1. **加载配置时崩溃**
   - 检查 `subscriptions.json` 格式是否正确
   - 检查 `basic_test.yml` 是否存在

2. **合并配置时崩溃**
   - 检查订阅 URL 是否有效
   - 检查网络连接

3. **启动服务器时崩溃**
   - 端口 8080 是否被占用？运行 `lsof -i :8080`
   - 尝试更改端口（修改 `subscriptions.json` 中的 port）

4. **服务器运行期间崩溃**
   - 查看终端的完整错误信息
   - 检查是否有 panic 信息

## 关键改进说明

### 优雅关闭机制

**旧方法（会导致崩溃）：**
```rust
handle.abort();  // 暴力终止任务，可能导致 panic
```

**新方法（优雅关闭）：**
```rust
// 使用 oneshot 通道
let (shutdown_tx, shutdown_rx) = oneshot::channel();

// 启动服务器时传入接收端
axum::serve(listener, app)
    .with_graceful_shutdown(async move {
        let _ = shutdown_rx.await;  // 等待关闭信号
    })
    .await

// 停止时发送信号
shutdown_tx.send(());  // 触发优雅关闭
```

这样可以确保：
- ✅ 当前请求处理完成
- ✅ 连接正确关闭
- ✅ 资源正确清理
- ✅ 不会产生 panic

## 编译检查

```bash
# TypeScript 检查
bun run tsc --noEmit

# Rust 检查
cargo check --manifest-path src-tauri/Cargo.toml

# 运行测试
cargo test --manifest-path src-tauri/Cargo.toml
```

所有检查都应该通过（只有一个无害的 warning）。

## 下一步

如果测试通过，我们可以继续：
1. 移除调试日志（保留关键日志）
2. 添加网络超时处理
3. 添加订阅测试功能
4. 改进错误提示
