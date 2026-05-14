# File Watching Feature for Hangar Serve Command

## 概述 (Overview)

实现了 `serve` 命令的文件监控功能，能够自动检测 `current.yaml` 文件的变化并重新加载配置。同时添加了 `--daemon` 参数支持后台运行，并修复了文件变化时的重复日志问题。

The `serve` command now includes file watching functionality that automatically detects changes to `current.yaml` and reloads the configuration in real-time. Added `--daemon` flag for background execution and fixed duplicate log entries when files change.

## 主要改动 (Key Changes)

### 1. 依赖添加 (Dependencies Added)

在 `Cargo.toml` 中添加了 `notify` crate 用于文件系统监控：

```toml
notify = "7.0"
```

### 2. 服务器状态管理 (Server State Management)

在 `src/server.rs` 中：
- 为 `AppState` 添加了 `reload_from_file` 方法
- 该方法可以从磁盘读取 `current.yaml` 并更新内存中的配置

```rust
impl AppState {
    pub async fn reload_from_file(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let content = std::fs::read_to_string(path)?;
        let new_config: ClashConfig = serde_yaml::from_str(&content)?;
        
        let mut config = self.config.write().await;
        *config = new_config;
        
        Ok(())
    }
}
```

### 3. 文件监控实现 (File Watching Implementation)

在 `src/main.rs` 的 `serve` 命令中：
- 使用 `notify` crate 创建文件监控器
- 监控 `current.yaml` 文件的修改事件
- 当检测到文件变化时，自动调用 `reload_from_file` 重新加载配置

关键代码：
```rust
// Spawn file watcher task
tokio::task::spawn_blocking(move || {
    use notify::{Watcher, RecursiveMode, Event, EventKind};
    
    let (tx_notify, rx_notify) = std::sync::mpsc::channel::<Result<Event, notify::Error>>();
    
    let mut watcher = notify::recommended_watcher(tx_notify)
        .expect("Failed to create file watcher");
    
    watcher.watch(&watch_path, RecursiveMode::NonRecursive)
        .expect("Failed to watch current.yaml");
    
    println!("👀 Watching for changes to {:?}", watch_path);
    
    for res in rx_notify {
        match res {
            Ok(event) => {
                if matches!(event.kind, EventKind::Modify(_)) {
                    println!("📝 Detected change in current.yaml, reloading...");
                    
                    // Reload config
                    runtime.spawn(async move {
                        match state.reload_from_file(&path).await {
                            Ok(_) => println!("✅ Config reloaded successfully"),
                            Err(e) => eprintln!("❌ Failed to reload config: {}", e),
                        }
                    });
                }
            }
            Err(e) => eprintln!("⚠️ Watch error: {}", e),
        }
    }
});
```

### 4. 重复日志修复 (Duplicate Log Fix)

**问题**：文件系统监控器在文件修改时可能触发多个事件（如 `Modify(Data)` 和 `Modify(Metadata)`），导致重复的日志输出。

**解决方案**：实现了两层防护机制：

1. **精确事件过滤**：只响应 `EventKind::Modify(ModifyKind::Data(_))` 事件
2. **Debounce 机制**：使用时间戳记录上次重载时间，100ms 内的重复事件会被忽略

```rust
// Debounce mechanism to prevent duplicate events
let last_reload = Arc::new(Mutex::new(Instant::now()));
let debounce_duration = Duration::from_millis(100);

for res in rx_notify {
    match res {
        Ok(event) => {
            // Only respond to data modification events
            if matches!(event.kind, EventKind::Modify(notify::event::ModifyKind::Data(_))) {
                let mut last = last_reload.lock().unwrap();
                let now = Instant::now();
                
                // Only reload if enough time has passed
                if now.duration_since(*last) > debounce_duration {
                    *last = now;
                    drop(last);
                    
                    println!("📝 Detected change in current.yaml, reloading...");
                    // ... reload logic
                }
            }
        }
        Err(e) => eprintln!("⚠️ Watch error: {}", e),
    }
}
```

### 5. Daemon 模式实现 (Daemon Mode Implementation)

添加了 `--daemon` 参数，允许服务器在后台运行：

**实现原理**：
- 检测到 `--daemon` 标志时，重新启动自身作为子进程
- 子进程的标准输入/输出/错误重定向到日志文件
- 父进程保存子进程 PID 并退出
- 子进程继续运行服务器

**关键代码**：
```rust
if daemon {
    let exe = std::env::current_exe()?;
    let mut args: Vec<String> = std::env::args().collect();
    args.retain(|arg| arg != "--daemon" && arg != "-d");
    
    let log_path = storage::get_hangar_dir()?.join("server.log");
    
    let child = std::process::Command::new(&exe)
        .args(&args[1..])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::from(/*log file*/))
        .stderr(std::process::Stdio::from(/*log file*/))
        .spawn()?;
    
    // Save PID
    let pid_path = storage::get_hangar_dir()?.join("server.pid");
    std::fs::write(&pid_path, child.id().to_string())?;
    
    return Ok(());
}

## 工作流程 (Workflow)

1. **服务器启动 (Server Start)**
   - 加载初始配置并保存到 `current.yaml`
   - 创建共享状态 `AppState`
   - 启动文件监控任务

2. **文件监控 (File Watching)**
   - 监控器持续监听 `current.yaml` 的变化
   - 检测到修改事件时触发重新加载

3. **自动重载 (Auto Reload)**
   - 从磁盘读取更新后的 `current.yaml`
   - 解析 YAML 内容为 `ClashConfig` 结构
   - 更新共享状态中的配置
   - 后续的 HTTP 请求将返回新的配置

4. **配置更新来源 (Config Update Sources)**
   - 手动编辑 `current.yaml` 文件
   - `--interval` 参数触发的自动订阅更新
   - AI 命令修改配置
   - 其他任何修改 `current.yaml` 的操作

## 使用方法 (Usage)

### 启动服务器 (Start Server)

```bash
# 仅启动服务器（前台）
cargo run -- serve --port 8080

# 启动服务器并启用自动订阅更新（每300秒）
cargo run -- serve --port 8080 --interval 300

# 以 daemon 模式启动（后台运行）
cargo run -- serve --daemon --port 8080 --interval 300

# 使用管理脚本
./hangar-server.sh start
./hangar-server.sh status
./hangar-server.sh logs -f
./hangar-server.sh stop
```

### Daemon 模式 (Daemon Mode)

使用 `--daemon` 或 `-d` 参数可以让服务器在后台运行：

```bash
hangar serve --daemon --port 8080
```

Daemon 模式特性：
- 进程在后台运行，不占用终端
- 日志输出到 `~/.hangar/server.log`
- PID 保存到 `~/.hangar/server.pid`
- 可以通过 PID 文件管理进程

停止 daemon：
```bash
# 方式1：使用管理脚本
./hangar-server.sh stop

# 方式2：直接使用 kill
kill $(cat ~/.hangar/server.pid)
```

### 测试文件监控 (Test File Watching)

1. 启动服务器：
   ```bash
   cargo run -- serve --port 8080
   ```

2. 在另一个终端中修改配置：
   ```bash
   # 打开编辑器
   open ~/.hangar/current.yaml
   
   # 或使用 vim/nano 等编辑器
   vim ~/.hangar/current.yaml
   ```

3. 保存文件后，在服务器终端中会看到：
   ```
   📝 Detected change in current.yaml, reloading...
   ✅ Config reloaded successfully
   ```

### 使用测试脚本 (Using Test Script)

```bash
./test-file-watch.sh
```

## 技术细节 (Technical Details)

### 线程模型 (Threading Model)

- 文件监控运行在 `spawn_blocking` 任务中（因为 `notify` 是同步的）
- 配置重载异步执行，使用 `tokio::spawn`
- 使用 `Arc<RwLock<ClashConfig>>` 实现线程安全的配置共享

### 性能考虑 (Performance Considerations)

- 读锁允许多个并发的 HTTP 请求
- 写锁仅在重载配置时使用，时间很短
- 文件监控使用操作系统的 inotify/FSEvents，性能开销小

### 错误处理 (Error Handling)

- 文件监控错误会打印警告，但不会终止服务器
- 配置重载失败时保留旧配置，确保服务持续可用
- 所有错误都会记录到标准错误输出

## 优点 (Benefits)

1. **实时更新** - 无需重启服务器即可应用配置更改
2. **自动化** - 与自动订阅更新功能完美配合
3. **灵活性** - 支持手动编辑和程序化修改
4. **可靠性** - 重载失败不影响现有服务
5. **性能** - 使用高效的文件系统事件通知机制

## 兼容性 (Compatibility)

- ✅ macOS (FSEvents)
- ✅ Linux (inotify)
- ✅ Windows (ReadDirectoryChangesW)

所有主流操作系统都得到 `notify` crate 的原生支持。

## 未来改进 (Future Improvements)

- [ ] 支持监控多个配置文件（`basic.yaml`, `groups.yaml`）
- [ ] 添加配置验证，防止加载无效配置
- [ ] 实现配置热重载的WebSocket推送通知
- [ ] 添加重载历史记录和回滚功能
