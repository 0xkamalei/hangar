use crate::types::ClashConfig;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use chrono::Local;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::{oneshot, RwLock};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<RwLock<ClashConfig>>,
}

#[derive(Deserialize)]
pub struct ConfigQuery {
    pub refresh: Option<bool>,
}

impl AppState {
    /// 从文件重新加载配置
    pub async fn reload_from_file(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let content = std::fs::read_to_string(path)?;
        let new_config: ClashConfig = serde_yaml::from_str(&content)?;

        let mut config = self.config.write().await;
        *config = new_config;

        Ok(())
    }

    /// 刷新配置（重新下载订阅并合并）
    pub async fn refresh(&self) -> anyhow::Result<()> {
        eprintln!("🔄 手动刷新订阅...");
        // 1. 加载订阅
        let mut subs = crate::storage::load_subscriptions().unwrap_or_default();
        let mut any_updated = false;

        for sub in &mut subs {
            if sub.enabled {
                match crate::subscription::download_subscription(sub).await {
                    Ok(_) => {
                        sub.last_updated =
                            Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                        any_updated = true;
                    }
                    Err(e) => {
                        eprintln!("⚠️ 无法更新订阅 {}: {}", sub.name, e)
                    }
                }
            }
        }

        if any_updated {
            crate::storage::save_subscriptions(&subs)?;
        }

        // 2. 合并配置
        let merged = crate::proxy::merge_configs(&subs).await?;

        // 3. 保存到 current.yaml
        let current_path = crate::storage::get_current_config_path()?;
        crate::config::save_config(&merged, current_path.to_str().unwrap())?;

        // 4. 更新内存中的状态
        let mut config = self.config.write().await;
        *config = merged;

        eprintln!("✅ 刷新完成");
        Ok(())
    }
}

/// 获取配置的处理器
async fn get_config(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<ConfigQuery>,
) -> Response {
    if let Some(true) = query.refresh {
        if let Err(e) = state.refresh().await {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to refresh config: {}", e),
            )
                .into_response();
        }
    }

    let config = state.config.read().await;

    match serde_yaml::to_string(&*config) {
        Ok(yaml) => (StatusCode::OK, yaml).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to serialize config: {}", e),
        )
            .into_response(),
    }
}

/// 健康检查
async fn health_check() -> &'static str {
    "OK"
}

/// 创建并启动 HTTP 服务器
pub async fn start_server(
    config: ClashConfig,
    host: &str,
    port: u16,
    shutdown_rx: oneshot::Receiver<()>,
) -> anyhow::Result<()> {
    eprintln!("🔍 start_server 函数被调用: {}:{}", host, port);

    let state = AppState {
        config: Arc::new(RwLock::new(config)),
    };

    eprintln!("✓ AppState 创建成功");

    let app = Router::new()
        .route("/config", get(get_config))
        .route("/health", get(health_check))
        .with_state(state);

    eprintln!("✓ Router 创建成功");

    let addr = format!("{}:{}", host, port);
    eprintln!("🔍 尝试绑定地址: {}", addr);

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => {
            eprintln!("✓ TcpListener 绑定成功");
            l
        }
        Err(e) => {
            eprintln!("❌ TcpListener 绑定失败: {}", e);
            return Err(anyhow::anyhow!("无法绑定地址 {}: {}", addr, e));
        }
    };

    eprintln!("\n🌐 正在启动 HTTP 服务器...");
    eprintln!("   地址: http://{}", addr);
    eprintln!("   订阅链接: http://{}/config", addr);
    eprintln!("\n✨ 服务器已启动，等待请求...\n");

    // 使用 with_graceful_shutdown 支持优雅关闭
    match axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            // 等待关闭信号
            let _ = shutdown_rx.await;
            eprintln!("🛑 收到关闭信号，开始优雅关闭...");
        })
        .await
    {
        Ok(_) => {
            eprintln!("✓ 服务器已优雅关闭");
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ 服务器错误: {}", e);
            Err(anyhow::anyhow!("服务器运行错误: {}", e))
        }
    }
}
