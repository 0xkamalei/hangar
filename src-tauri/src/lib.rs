pub mod config;
pub mod proxy;
pub mod server;
pub mod subscription;
pub mod types;

use config::{load_app_config, load_basic_config, save_config};
use proxy::merge_configs;
use server::start_server;
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};

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
    std::fs::create_dir_all(&app_dir)
        .map_err(|e| format!("无法创建应用目录: {}", e))?;
    
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
    
    let app_config = load_app_config(&config_path)
        .map_err(|e| {
            let err = format!("❌ 无法加载配置文件: {}\n\n路径: {}", e, config_path);
            eprintln!("{}", err);
            err
        })?;
    
    eprintln!("✓ 配置加载成功，订阅数量: {}", app_config.subscriptions.len());
    
    // 尝试加载基础配置
    let basic_config_paths = vec![
        app_config.basic_config.path.clone(),
        "_docs/basic.yml".to_string(),
        "../_docs/basic.yml".to_string(),
        "basic_test.yml".to_string(),
    ];
    
    let mut basic_config = None;
    for path in &basic_config_paths {
        eprintln!("🔍 尝试加载基础配置: {}", path);
        match load_basic_config(path) {
            Ok(config) => {
                eprintln!("✓ 基础配置加载成功: {}", path);
                basic_config = Some(config);
                break;
            }
            Err(e) => {
                eprintln!("⚠️  加载失败: {}", e);
                continue;
            }
        }
    }
    
    let basic_config = basic_config.ok_or_else(|| {
        let err = "❌ 无法加载基础配置文件\n\n请确保 basic.yml 或 basic_test.yml 存在".to_string();
        eprintln!("{}", err);
        err
    })?;
    
    eprintln!("✓ 开始合并配置...");
    
    // 合并配置
    let merged_config = merge_configs(&app_config.subscriptions, basic_config)
        .await
        .map_err(|e| {
            let err = format!("❌ 合并配置失败: {}", e);
            eprintln!("{}", err);
            err
        })?;
    
    eprintln!("✓ 配置合并成功");
    
    // 保存配置
    save_config(&merged_config, &app_config.output.path)
        .map_err(|e| {
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
fn get_subscriptions(app_handle: tauri::AppHandle) -> Result<Vec<types::Subscription>, String> {
    let config_path = get_config_path(&app_handle)?;
    
    match load_app_config(&config_path) {
        Ok(config) => Ok(config.subscriptions),
        Err(_) => Ok(vec![]) // 如果文件不存在，返回空列表
    }
}

#[tauri::command]
fn add_subscription(app_handle: tauri::AppHandle, name: String, url: String) -> Result<String, String> {
    let config_path = get_config_path(&app_handle)?;
    
    let mut config = load_app_config(&config_path)
        .map_err(|e| format!("加载配置失败: {}", e))?;
    
    config.subscriptions.push(types::Subscription {
        name,
        url,
        enabled: true,
    });
    
    // 保存配置
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    std::fs::write(&config_path, json)
        .map_err(|e| format!("保存配置失败: {}", e))?;
    
    Ok("✅ 订阅添加成功".to_string())
}

#[tauri::command]
fn update_subscription(app_handle: tauri::AppHandle, index: usize, name: String, url: String, enabled: bool) -> Result<String, String> {
    let config_path = get_config_path(&app_handle)?;
    
    let mut config = load_app_config(&config_path)
        .map_err(|e| format!("加载配置失败: {}", e))?;
    
    if index >= config.subscriptions.len() {
        return Err("订阅索引超出范围".to_string());
    }
    
    config.subscriptions[index] = types::Subscription {
        name,
        url,
        enabled,
    };
    
    // 保存配置
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    std::fs::write(&config_path, json)
        .map_err(|e| format!("保存配置失败: {}", e))?;
    
    Ok("✅ 订阅更新成功".to_string())
}

#[tauri::command]
fn delete_subscription(app_handle: tauri::AppHandle, index: usize) -> Result<String, String> {
    eprintln!("🔍 delete_subscription 被调用，index: {}", index);
    
    let config_path = get_config_path(&app_handle)?;
    eprintln!("✓ 配置路径: {}", config_path);
    
    let mut config = load_app_config(&config_path)
        .map_err(|e| format!("加载配置失败: {}", e))?;
    
    eprintln!("✓ 当前订阅数量: {}", config.subscriptions.len());
    
    if index >= config.subscriptions.len() {
        let err = format!("订阅索引超出范围: index={}, len={}", index, config.subscriptions.len());
        eprintln!("❌ {}", err);
        return Err(err);
    }
    
    let removed = config.subscriptions.remove(index);
    eprintln!("✓ 已删除订阅: {}", removed.name);
    
    // 保存配置
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    std::fs::write(&config_path, json)
        .map_err(|e| format!("保存配置失败: {}", e))?;
    
    eprintln!("✓ 配置已保存");
    
    Ok("✅ 订阅删除成功".to_string())
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
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
            add_subscription,
            update_subscription,
            delete_subscription
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
