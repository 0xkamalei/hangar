pub mod ai;
pub mod config;
pub mod notifications;
pub mod proxy;
pub mod rules;
pub mod server;
pub mod storage;
pub mod subscription;
pub mod types;
pub mod version;

use config::{load_app_config, save_config};
use proxy::merge_configs;
use server::start_server;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};

// 全局状态，用于存储服务器是否已启动和服务器关闭通道
lazy_static::lazy_static! {
    static ref SERVER_RUNNING: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    static ref SERVER_SHUTDOWN: Arc<Mutex<Option<oneshot::Sender<()>>>> = Arc::new(Mutex::new(None));
}

// 获取配置文件路径
fn get_config_path(app_handle: &tauri::AppHandle) -> Result<String, String> {
    // 先尝试当前目录（开发模式）
    let dev_paths = vec![
        "subscriptions.json",
        "../subscriptions.json",
        "../../subscriptions.json",
    ];

    for path in &dev_paths {
        if std::path::Path::new(path).exists() {
            eprintln!("✓ 使用开发模式配置: {}", path);
            return Ok(path.to_string());
        }
    }

    // 生产模式：使用应用数据目录
    use tauri::Manager;
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取应用目录: {}", e))?;

    // 确保目录存在
    std::fs::create_dir_all(&app_dir).map_err(|e| format!("无法创建应用目录: {}", e))?;

    let config_file = app_dir.join("subscriptions.json");

    // 如果配置文件不存在，创建默认配置
    if !config_file.exists() {
        let default_config = r#"{
  "subscriptions": [],
  "server": {
    "port": 8080,
    "host": "127.0.0.1"
  },
  "output": {
    "path": "output_config.yaml"
  },
  "basic_config": {
    "path": "basic_test.yml"
  }
}"#;
        std::fs::write(&config_file, default_config)
            .map_err(|e| format!("无法创建默认配置: {}", e))?;
        eprintln!("✓ 创建默认配置文件: {}", config_file.display());
    }

    Ok(config_file.to_string_lossy().to_string())
}

#[tauri::command]
async fn start_proxy_server(app_handle: tauri::AppHandle) -> Result<String, String> {
    eprintln!("🔍 start_proxy_server 被调用");

    let mut running = SERVER_RUNNING.lock().await;

    if *running {
        eprintln!("⚠️  服务器已在运行中");
        return Ok("服务器已在运行中".to_string());
    }

    // 获取配置文件路径
    let config_path = get_config_path(&app_handle)?;
    eprintln!("✓ 使用配置文件: {}", config_path);

    let app_config = load_app_config(&config_path).map_err(|e| {
        let err = format!("❌ 无法加载配置文件: {}\n\n路径: {}", e, config_path);
        eprintln!("{}", err);
        err
    })?;

    eprintln!(
        "✓ 配置加载成功，订阅数量: {}",
        app_config.subscriptions.len()
    );

    // Check for existence of basic config only for UX warning
    let basic_config_paths = vec![
        app_config.basic_config.path.clone(),
        "_docs/basic.yml".to_string(),
        "../_docs/basic.yml".to_string(),
        "basic_test.yml".to_string(),
    ];

    let mut basic_exists = false;
    for path in &basic_config_paths {
        if std::path::Path::new(path).exists() {
            basic_exists = true;
            break;
        }
    }

    if !basic_exists {
        eprintln!("⚠️  Suggested basic config paths not found, merge might fail if resources are missing.");
    }

    eprintln!("✓ 开始合并配置...");

    // 合并配置
    let merged_config = merge_configs(&app_config.subscriptions)
        .await
        .map_err(|e| {
            let err = format!("❌ 合并配置失败: {}", e);
            eprintln!("{}", err);
            err
        })?;

    eprintln!("✓ 配置合并成功");

    // 保存配置
    save_config(&merged_config, &app_config.output.path).map_err(|e| {
        let err = format!("❌ 保存配置失败: {}", e);
        eprintln!("{}", err);
        err
    })?;

    eprintln!("✓ 配置已保存到: {}", app_config.output.path);

    let host = app_config.server.host.clone();
    let port = app_config.server.port;

    *running = true;

    // 克隆 host 用于返回消息
    let host_for_message = host.clone();

    eprintln!("✓ 即将启动服务器: {}:{}", host, port);

    // 创建关闭通道
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    // 保存关闭发送端
    let mut shutdown_sender = SERVER_SHUTDOWN.lock().await;
    *shutdown_sender = Some(shutdown_tx);
    drop(shutdown_sender); // 释放锁

    // 在后台启动服务器
    tokio::spawn(async move {
        eprintln!("🚀 服务器任务已启动");
        match start_server(merged_config, &host, port, shutdown_rx).await {
            Ok(_) => {
                eprintln!("✓ 服务器正常停止");
            }
            Err(e) => {
                eprintln!("❌ 服务器错误: {}", e);
                eprintln!("❌ 错误详情: {:?}", e);
            }
        }

        // 无论如何都要更新运行状态
        let mut running = SERVER_RUNNING.lock().await;
        *running = false;
        eprintln!("✓ 服务器状态已更新为停止");
    });

    eprintln!("✅ 服务器启动命令执行成功");

    Ok(format!("✅ 服务器已启动\n\n📍 订阅链接: http://{}:{}/config\n\n💡 在 Clash Verge 中添加此链接即可使用", host_for_message, port))
}

#[tauri::command]
async fn stop_proxy_server() -> Result<String, String> {
    eprintln!("🔍 stop_proxy_server 被调用");

    let mut running = SERVER_RUNNING.lock().await;

    if !*running {
        eprintln!("⚠️  服务器未运行");
        return Ok("服务器未运行".to_string());
    }

    // 获取关闭发送端并发送关闭信号
    let mut shutdown_sender = SERVER_SHUTDOWN.lock().await;
    if let Some(tx) = shutdown_sender.take() {
        eprintln!("✓ 发送关闭信号");
        let _ = tx.send(()); // 忽略发送错误（接收端可能已关闭）
    }

    *running = false;

    eprintln!("✅ 服务器停止命令执行成功");

    Ok("✅ 服务器已停止".to_string())
}

#[tauri::command]
async fn get_server_status() -> Result<bool, String> {
    let running = SERVER_RUNNING.lock().await;
    Ok(*running)
}

#[tauri::command]
async fn refresh_subscription(id: String) -> Result<types::Subscription, String> {
    let mut subscriptions = storage::load_subscriptions()
        .map_err(|e| format!("Failed to load subscriptions: {}", e))?;

    let sub_index = subscriptions
        .iter()
        .position(|s| s.id == id)
        .ok_or_else(|| "Subscription not found".to_string())?;

    let sub = &subscriptions[sub_index];

    // Download fresh data
    match subscription::download_subscription(sub).await {
        Ok(_) => {}
        Err(e) => return Err(format!("Failed to download subscription: {}", e)),
    }

    // Parse to get count
    let proxies = proxy::parse_cached_subscription(sub)
        .map_err(|e| format!("Failed to parse subscription: {}", e))?;

    let node_count = proxies.len();
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Update metadata
    let mut updated_sub = subscriptions[sub_index].clone();
    updated_sub.node_count = Some(node_count);
    updated_sub.last_updated = Some(now);

    subscriptions[sub_index] = updated_sub.clone();

    // Save updated subscriptions list
    storage::save_subscriptions(&subscriptions)
        .map_err(|e| format!("Failed to save subscriptions: {}", e))?;

    Ok(updated_sub)
}

#[tauri::command]
fn get_subscriptions() -> Result<Vec<types::Subscription>, String> {
    storage::load_subscriptions().map_err(|e| format!("Failed to load subscriptions: {}", e))
}

#[tauri::command]
fn add_subscription(name: String, url: String) -> Result<String, String> {
    let mut subscriptions = storage::load_subscriptions()
        .map_err(|e| format!("Failed to load subscriptions: {}", e))?;

    let id = uuid::Uuid::new_v4().to_string();

    subscriptions.push(types::Subscription {
        id,
        name,
        url,
        enabled: true,
        last_updated: None,
        node_count: None,
    });

    storage::save_subscriptions(&subscriptions)
        .map_err(|e| format!("Failed to save subscriptions: {}", e))?;

    Ok("✅ 订阅添加成功".to_string())
}

#[tauri::command]
fn update_subscription(
    index: usize,
    name: String,
    url: String,
    enabled: bool,
) -> Result<String, String> {
    let mut subscriptions = storage::load_subscriptions()
        .map_err(|e| format!("Failed to load subscriptions: {}", e))?;

    if index >= subscriptions.len() {
        return Err("订阅索引超出范围".to_string());
    }

    let id = subscriptions[index].id.clone();
    subscriptions[index] = types::Subscription {
        id,
        name,
        url,
        enabled,
        last_updated: None,
        node_count: None,
    };

    storage::save_subscriptions(&subscriptions)
        .map_err(|e| format!("Failed to save subscriptions: {}", e))?;

    Ok("✅ 订阅更新成功".to_string())
}

#[tauri::command]
fn delete_subscription(index: usize) -> Result<String, String> {
    eprintln!("🔍 delete_subscription 被调用，index: {}", index);

    let mut subscriptions = storage::load_subscriptions()
        .map_err(|e| format!("Failed to load subscriptions: {}", e))?;

    eprintln!("✓ 当前订阅数量: {}", subscriptions.len());

    if index >= subscriptions.len() {
        let err = format!(
            "订阅索引超出范围: index={}, len={}",
            index,
            subscriptions.len()
        );
        eprintln!("❌ {}", err);
        return Err(err);
    }

    let removed = subscriptions.remove(index);
    eprintln!("✓ 已删除订阅: {}", removed.name);

    storage::save_subscriptions(&subscriptions)
        .map_err(|e| format!("Failed to save subscriptions: {}", e))?;

    eprintln!("✓ 配置已保存");

    Ok("✅ 订阅删除成功".to_string())
}

#[tauri::command]
fn batch_delete_subscriptions(indices: Vec<usize>) -> Result<String, String> {
    let mut subscriptions = storage::load_subscriptions()
        .map_err(|e| format!("Failed to load subscriptions: {}", e))?;

    let mut sorted_indices = indices.clone();
    sorted_indices.sort_by(|a, b| b.cmp(a)); // Sort descending to remove without affecting other indices

    for index in sorted_indices {
        if index < subscriptions.len() {
            subscriptions.remove(index);
        }
    }

    storage::save_subscriptions(&subscriptions)
        .map_err(|e| format!("Failed to save subscriptions: {}", e))?;

    Ok("✅ 批量删除成功".to_string())
}

#[tauri::command]
fn batch_toggle_subscriptions(indices: Vec<usize>, enabled: bool) -> Result<String, String> {
    let mut subscriptions = storage::load_subscriptions()
        .map_err(|e| format!("Failed to load subscriptions: {}", e))?;

    for &index in &indices {
        if index < subscriptions.len() {
            subscriptions[index].enabled = enabled;
        }
    }

    storage::save_subscriptions(&subscriptions)
        .map_err(|e| format!("Failed to save subscriptions: {}", e))?;

    Ok("✅ 批量状态更新成功".to_string())
}

#[tauri::command]
fn export_subscriptions(path: String) -> Result<String, String> {
    let subscriptions = storage::load_subscriptions().map_err(|e| e.to_string())?;

    let content = serde_json::to_string_pretty(&types::SubscriptionList { subscriptions })
        .map_err(|e| e.to_string())?;

    std::fs::write(path, content).map_err(|e| e.to_string())?;

    Ok("✅ 订阅已导出".to_string())
}

#[tauri::command]
fn import_subscriptions(path: String) -> Result<Vec<types::Subscription>, String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;

    let list: types::SubscriptionList =
        serde_json::from_str(&content).map_err(|e| e.to_string())?;

    let mut current_subs = storage::load_subscriptions().map_err(|e| e.to_string())?;

    // Add imported subscriptions (avoiding ID duplicates if any, though UUIDs should be unique)

    for mut sub in list.subscriptions {
        if !current_subs.iter().any(|s| s.id == sub.id) {
            // If ID matches but name/url different, we could generate new ID,
            // but for simplicity we just append if ID is not there.
            current_subs.push(sub);
        } else {
            // Regnerate ID to avoid collision
            sub.id = uuid::Uuid::new_v4().to_string();
            current_subs.push(sub);
        }
    }

    storage::save_subscriptions(&current_subs).map_err(|e| e.to_string())?;

    Ok(current_subs)
}

#[tauri::command]
fn get_notifications() -> Vec<types::Notification> {
    notifications::get_notifications()
}

#[tauri::command]
fn mark_notification_read(id: String) {
    notifications::mark_as_read(&id);
}

#[tauri::command]
fn clear_notifications() {
    notifications::clear_notifications();
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// AI Commands
#[tauri::command]
async fn generate_ai_patch(prompt: String) -> Result<ai::AiPatchResult, String> {
    ai::generate_config_patch(&prompt)
        .await
        .map_err(|e| format!("AI generation failed: {}", e))
}

#[tauri::command]
async fn test_llm_connection(
    base_url: String,
    api_key: String,
    model: String,
) -> Result<String, String> {
    ai::test_llm_connection(&base_url, &api_key, &model)
        .await
        .map_err(|e| format!("{}", e))
}

#[tauri::command]
async fn apply_ai_patch(operations: Vec<serde_json::Value>) -> Result<String, String> {
    let current_path = storage::get_current_config_path().map_err(|e| e.to_string())?;

    if !current_path.exists() {
        return Err("当前配置文件不存在".to_string());
    }

    let current_content = std::fs::read_to_string(&current_path).map_err(|e| e.to_string())?;

    let patched_content =
        ai::apply_patch_to_config(&current_content, &operations).map_err(|e| e.to_string())?;

    version::save_version("current", "auto_before_ai_patch", &current_content)
        .map_err(|e| e.to_string())?;

    std::fs::write(&current_path, patched_content).map_err(|e| e.to_string())?;

    Ok("✅ AI 修改已应用并创建了备份快照".to_string())
}

// Config Commands
#[tauri::command]
fn get_hangar_config() -> Result<types::HangarConfig, String> {
    storage::load_hangar_config().map_err(|e| format!("Failed to load config: {}", e))
}

#[tauri::command]
fn save_hangar_config(config: types::HangarConfig) -> Result<String, String> {
    storage::save_hangar_config(&config).map_err(|e| format!("Failed to save config: {}", e))?;
    Ok("配置已保存".to_string())
}

// Version Commands
#[tauri::command]
fn list_versions() -> Result<Vec<types::ConfigVersion>, String> {
    version::list_versions().map_err(|e| format!("Failed to list versions: {}", e))
}

#[tauri::command]
fn get_version_content(id: String) -> Result<String, String> {
    version::get_version_content(&id).map_err(|e| format!("Failed to get version: {}", e))
}

#[tauri::command]
fn create_manual_snapshot(description: String) -> Result<types::ConfigVersion, String> {
    let current_path = storage::get_current_config_path().map_err(|e| e.to_string())?;

    let current_content = std::fs::read_to_string(&current_path).map_err(|e| e.to_string())?;

    version::save_version("manual", &description, &current_content).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_versions_diff(id1: String, id2: Option<String>) -> Result<Vec<version::DiffLine>, String> {
    let content1 = version::get_version_content(&id1).map_err(|e| e.to_string())?;

    let content2 = if let Some(id) = id2 {
        version::get_version_content(&id).map_err(|e| e.to_string())?
    } else {
        let current_path = storage::get_current_config_path().map_err(|e| e.to_string())?;
        std::fs::read_to_string(&current_path).map_err(|e| e.to_string())?
    };

    Ok(version::diff_configs(&content1, &content2))
}

#[tauri::command]
fn rollback_version(id: String) -> Result<String, String> {
    version::rollback_to_version(&id).map_err(|e| format!("Failed to rollback: {}", e))?;
    Ok("已回退到指定版本".to_string())
}

#[tauri::command]
fn delete_version(id: String) -> Result<String, String> {
    version::delete_version(&id).map_err(|e| format!("Failed to delete version: {}", e))?;
    Ok("版本已删除".to_string())
}

// Rules Commands
#[tauri::command]
fn open_config_in_editor(app_handle: tauri::AppHandle) -> Result<(), String> {
    let path = storage::get_current_config_path().map_err(|e| e.to_string())?;

    use tauri_plugin_opener::OpenerExt;
    app_handle
        .opener()
        .open_path(path.to_string_lossy().to_string(), None::<String>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn open_data_directory(app_handle: tauri::AppHandle) -> Result<(), String> {
    let path = storage::get_hangar_dir().map_err(|e| e.to_string())?;

    use tauri_plugin_opener::OpenerExt;
    app_handle
        .opener()
        .open_path(path.to_string_lossy().to_string(), None::<String>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_builtin_rules() -> Vec<rules::BuiltinRule> {
    rules::get_default_builtin_rules()
}

#[tauri::command]
fn get_rule_sources() -> Result<Vec<rules::RuleSource>, String> {
    rules::load_rule_sources().map_err(|e| format!("Failed to load rule sources: {}", e))
}

#[tauri::command]
fn add_rule_source(name: String, url: String) -> Result<rules::RuleSource, String> {
    rules::add_rule_source(name, url).map_err(|e| format!("Failed to add rule source: {}", e))
}

#[tauri::command]
fn remove_rule_source(id: String) -> Result<String, String> {
    rules::remove_rule_source(&id).map_err(|e| format!("Failed to remove rule source: {}", e))?;
    Ok("规则源已删除".to_string())
}

#[tauri::command]
async fn refresh_rules() -> Result<String, String> {
    rules::refresh_all_rules()
        .await
        .map_err(|e| format!("Failed to refresh rules: {}", e))?;
    Ok("规则刷新完成".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            start_proxy_server,
            stop_proxy_server,
            get_server_status,
            get_subscriptions,
            refresh_subscription,
            add_subscription,
            update_subscription,
            delete_subscription,
            batch_delete_subscriptions,
            batch_toggle_subscriptions,
            export_subscriptions,
            import_subscriptions,
            get_notifications,
            mark_notification_read,
            clear_notifications,
            // AI commands
            generate_ai_patch,
            apply_ai_patch,
            test_llm_connection,
            // Config commands
            get_hangar_config,
            save_hangar_config,
            // Version commands
            list_versions,
            get_version_content,
            create_manual_snapshot,
            get_versions_diff,
            rollback_version,
            delete_version,
            // Rules commands
            get_builtin_rules,
            get_rule_sources,
            add_rule_source,
            remove_rule_source,
            refresh_rules,
            open_config_in_editor,
            open_data_directory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
