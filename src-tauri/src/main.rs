use anyhow::Context;
use axum::response::IntoResponse;
use clap::{Parser, Subcommand};
use hangar_lib::{ai, proxy, server, storage, subscription, types, version};

#[derive(Parser)]
#[command(name = "hangar")]
#[command(about = "A CLI tool for managing Clash subscriptions with AI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Subscription management
    Sub {
        #[command(subcommand)]
        subcommand: SubCommands,
    },
    /// Merge cached subscriptions and local rules into current.yaml
    Merge,
    /// Start the configuration server
    #[command(alias = "start")]
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Auto-update interval in seconds (0 to disable)
        #[arg(short, long, default_value_t = 0)]
        interval: u64,
        /// Run server in daemon mode (background)
        #[arg(short, long, default_value_t = false)]
        daemon: bool,
    },
    /// Stop the background server
    Stop,
    /// Restart the background server
    Restart,
    /// Check server status
    Status,
    /// Show server logs
    Logs {
        /// Follow logs in real-time
        #[arg(short, long, default_value_t = false)]
        follow: bool,
    },
    /// AI-powered configuration modification
    Ai {
        /// The natural language prompt
        prompt: String,
    },
    /// Version history management
    History {
        #[command(subcommand)]
        subcommand: Option<HistoryCommands>,
    },
    /// Application configuration
    Config {
        /// Set LLM API Key
        #[arg(long)]
        api_key: Option<String>,
        /// Set LLM Base URL
        #[arg(long)]
        base_url: Option<String>,
        /// Set LLM Model
        #[arg(long)]
        model: Option<String>,
    },
    /// Open config in default editor
    Editor,
}

#[derive(Subcommand)]
enum SubCommands {
    /// Add a new subscription and download it
    Add {
        /// Subscription URL
        url: String,
        /// Optional name for the subscription
        #[arg(short, long)]
        name: Option<String>,
    },
    /// List all subscriptions
    List,
    /// Remove a subscription
    Remove {
        /// The ID or index of the subscription
        id: String,
    },
    /// Enable a subscription
    Enable {
        /// The ID or index of the subscription
        id: String,
    },
    /// Disable a subscription
    Disable {
        /// The ID or index of the subscription
        id: String,
    },
}

#[derive(Subcommand)]
enum HistoryCommands {
    /// List all snapshots
    List,
    /// Rollback to a specific version (restores file based on version type)
    Rollback {
        /// Version ID
        id: String,
    },
    /// Show diff between two versions
    Diff {
        /// First version ID
        v1: String,
        /// Second version ID (defaults to current)
        v2: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Ensure basic config exists in user directory
    storage::ensure_basic_config_exists()?;
    storage::ensure_groups_config_exists()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Sub { subcommand } => {
            match subcommand {
                SubCommands::Add { url, name } => {
                    let name = name.unwrap_or_else(|| "Untitled".to_string());
                    let mut subs = storage::load_subscriptions().unwrap_or_default();
                    let id = uuid::Uuid::new_v4().to_string();
                    let current_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                    let mut new_sub = types::Subscription {
                        id: id.clone(),
                        name: name.clone(),
                        url,
                        enabled: true,
                        last_updated: Some(current_time),
                        node_count: None,
                    };

                    // Immediately download
                    println!("📥 Downloading subscription: {}...", name);
                    match subscription::download_subscription(&new_sub).await {
                        Ok(path) => {
                            println!("✅ Downloaded to {:?}", path);

                            // Parse and count proxies
                            match subscription::count_proxies(&id) {
                                Ok(count) => {
                                    new_sub.node_count = Some(count);
                                    println!("   Found {} nodes", count);
                                }
                                Err(e) => {
                                    println!("⚠️ Failed to count proxies: {}", e);
                                }
                            }
                        }
                        Err(e) => println!("⚠️ Failed to download: {}", e),
                    }

                    subs.push(new_sub);
                    storage::save_subscriptions(&subs)?;
                    println!("✅ Added subscription: {} ({})", name, id);
                }
                SubCommands::List => {
                    let subs = storage::load_subscriptions().unwrap_or_default();
                    if subs.is_empty() {
                        println!("No subscriptions found.");
                    } else {
                        println!(
                            "{:<36} {:<20} {:<10} {:<8} {:<20}",
                            "ID", "Name", "Nodes", "Enabled", "Last Updated"
                        );
                        for sub in subs {
                            println!(
                                "{:<36} {:<20} {:<10?} {:<8} {:<20}",
                                sub.id,
                                sub.name,
                                sub.node_count.unwrap_or(0),
                                if sub.enabled { "✓" } else { "✗" },
                                sub.last_updated.unwrap_or_else(|| "N/A".to_string())
                            );
                        }
                    }
                }
                SubCommands::Remove { id } => {
                    let mut subs = storage::load_subscriptions().unwrap_or_default();
                    // Logic to remove by ID or index...
                    let mut to_remove_id = None;
                    if let Ok(idx) = id.parse::<usize>() {
                        if idx < subs.len() {
                            to_remove_id = Some(subs[idx].id.clone());
                        }
                    } else {
                        to_remove_id = Some(id.clone());
                    }

                    if let Some(rid) = to_remove_id {
                        let len_before = subs.len();
                        subs.retain(|s| s.id != rid);
                        if subs.len() < len_before {
                            storage::save_subscriptions(&subs)?;
                            println!("✅ Removed subscription: {}", rid);

                            // Remove cache file
                            match storage::get_subscription_cache_path(&rid) {
                                Ok(path) => {
                                    if path.exists() {
                                        match std::fs::remove_file(&path) {
                                            Ok(_) => println!("🗑️ Removed cache file: {:?}", path),
                                            Err(e) => {
                                                println!("⚠️ Failed to remove cache file: {}", e)
                                            }
                                        }
                                    }
                                }
                                Err(e) => println!("⚠️ Failed to get cache path: {}", e),
                            }
                        } else {
                            println!("❌ Subscription not found.");
                        }
                    }
                }
                SubCommands::Enable { id } => {
                    let mut subs = storage::load_subscriptions().unwrap_or_default();
                    let mut found = false;

                    // Try to find subscription by name first (most user-friendly)
                    for sub in &mut subs {
                        if sub.name == id {
                            sub.enabled = true;
                            found = true;
                            println!("✅ Enabled subscription: {} ({})", sub.name, sub.id);
                            break;
                        }
                    }

                    // If not found by name, try by ID
                    if !found {
                        for sub in &mut subs {
                            if sub.id == id {
                                sub.enabled = true;
                                found = true;
                                println!("✅ Enabled subscription: {} ({})", sub.name, sub.id);
                                break;
                            }
                        }
                    }

                    // If still not found, try by index
                    if !found {
                        if let Ok(idx) = id.parse::<usize>() {
                            if idx < subs.len() {
                                subs[idx].enabled = true;
                                found = true;
                                println!(
                                    "✅ Enabled subscription: {} ({})",
                                    subs[idx].name, subs[idx].id
                                );
                            }
                        }
                    }

                    if found {
                        storage::save_subscriptions(&subs)?;

                        // Automatically run merge
                        println!("🔄 Triggering merge after enabling subscription...");
                        match proxy::merge_configs(&subs).await {
                            Ok(merged) => {
                                let output_path = storage::get_current_config_path()?;
                                hangar_lib::config::save_config(
                                    &merged,
                                    output_path.to_str().unwrap(),
                                )?;
                                println!("✅ Config regenerated.");
                            }
                            Err(e) => println!("❌ Auto-merge failed: {}", e),
                        }
                    } else {
                        println!("❌ Subscription not found: '{}'", id);
                        println!("   Try using the subscription name, ID, or index from 'hangar sub list'");
                    }
                }
                SubCommands::Disable { id } => {
                    let mut subs = storage::load_subscriptions().unwrap_or_default();
                    let mut found = false;

                    // Try to find subscription by name first (most user-friendly)
                    for sub in &mut subs {
                        if sub.name == id {
                            sub.enabled = false;
                            found = true;
                            println!("✅ Disabled subscription: {} ({})", sub.name, sub.id);
                            break;
                        }
                    }

                    // If not found by name, try by ID
                    if !found {
                        for sub in &mut subs {
                            if sub.id == id {
                                sub.enabled = false;
                                found = true;
                                println!("✅ Disabled subscription: {} ({})", sub.name, sub.id);
                                break;
                            }
                        }
                    }

                    // If still not found, try by index
                    if !found {
                        if let Ok(idx) = id.parse::<usize>() {
                            if idx < subs.len() {
                                subs[idx].enabled = false;
                                found = true;
                                println!(
                                    "✅ Disabled subscription: {} ({})",
                                    subs[idx].name, subs[idx].id
                                );
                            }
                        }
                    }

                    if found {
                        storage::save_subscriptions(&subs)?;

                        // Automatically run merge
                        println!("🔄 Triggering merge after disabling subscription...");
                        match proxy::merge_configs(&subs).await {
                            Ok(merged) => {
                                let output_path = storage::get_current_config_path()?;
                                hangar_lib::config::save_config(
                                    &merged,
                                    output_path.to_str().unwrap(),
                                )?;
                                println!("✅ Config regenerated.");
                            }
                            Err(e) => println!("❌ Auto-merge failed: {}", e),
                        }
                    } else {
                        println!("❌ Subscription not found: '{}'", id);
                        println!("   Try using the subscription name, ID, or index from 'hangar sub list'");
                    }
                }
            }
        }
        Commands::Merge => {
            println!("🔄 Merging configuration...");
            let subs = storage::load_subscriptions().unwrap_or_default();
            // Basic config is loaded inside merge_configs now (per update in proxy.rs)
            // But wait, proxy.rs signature was changed to NOT take basic_config?
            // Let's double check proxy.rs signature in our mind...
            // "pub async fn merge_configs(subscriptions: &[Subscription]) ..."
            // Ah, the previous step implementation didn't change the signature in `proxy.rs` fully in the provided output?
            // Wait, I see "basic_config: ClashConfig," in the replace_file_content for proxy.rs but commented out instructions?
            // Re-reading step 67 output:
            // "pub async fn merge_configs(subscriptions: &[Subscription], ) -> Result<ClashConfig> {"
            // It seems I removed `basic_config` argument.

            match proxy::merge_configs(&subs).await {
                Ok(merged) => {
                    let output_path = storage::get_current_config_path()?;
                    hangar_lib::config::save_config(&merged, output_path.to_str().unwrap())?;
                    println!("✅ Config generated and saved to {:?}", output_path);
                }
                Err(e) => println!("❌ Merge failed: {}", e),
            }
        }
        Commands::Serve {
            port,
            host,
            interval,
            daemon,
        } => {
            // Handle daemon mode
            if daemon {
                let pid_path = storage::get_server_pid_path()?;
                if pid_path.exists() {
                    let pid_str = std::fs::read_to_string(&pid_path)?;
                    if let Ok(pid) = pid_str.trim().parse::<i32>() {
                        // Check if process exists (cross-platform is hard, but simple check for unix)
                        #[cfg(unix)]
                        if unsafe { libc::kill(pid, 0) == 0 } {
                            println!("❌ Server is already running with PID: {}", pid);
                            return Ok(());
                        }
                    }
                }

                println!("🔄 Starting server in daemon mode...");

                // Get the current executable path
                let exe = std::env::current_exe()?;

                // Build arguments without the --daemon flag
                let mut args: Vec<String> = std::env::args().collect();
                args.retain(|arg| arg != "--daemon" && arg != "-d");

                // Get the log file path
                let log_path = storage::get_server_log_path()?;
                println!("📝 Logs will be written to: {:?}", log_path);

                // Spawn the process in the background
                let child = std::process::Command::new(&exe)
                    .args(&args[1..]) // Skip the executable name
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::from(
                        std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&log_path)?,
                    ))
                    .stderr(std::process::Stdio::from(
                        std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&log_path)?,
                    ))
                    .spawn()?;

                println!("✅ Server started in background with PID: {}", child.id());
                println!("   Address: http://{}:{}/config", host, port);
                println!("   Log file: {:?}", log_path);
                println!("\nTo stop the server, run: hangar stop");

                // Save PID to file for easy management
                std::fs::write(&pid_path, child.id().to_string())?;
                println!("   PID file: {:?}", pid_path);

                return Ok(());
            }

            println!("🚀 Starting server at http://{}:{}/config", host, port);

            // Initial load
            let subs = storage::load_subscriptions().unwrap_or_default();
            let merged = proxy::merge_configs(&subs).await?;

            // Save initial config to current.yaml
            let current_config_path = storage::get_current_config_path()?;
            hangar_lib::config::save_config(&merged, current_config_path.to_str().unwrap())?;

            let (_tx, rx) = tokio::sync::oneshot::channel::<()>();

            // Create shared state for server
            let state = server::AppState {
                config: std::sync::Arc::new(tokio::sync::RwLock::new(merged)),
            };

            // Clone state for file watcher
            let watcher_state = state.clone();
            let watch_path = current_config_path.clone();

            // Spawn file watcher task
            tokio::task::spawn_blocking(move || {
                use notify::{Event, EventKind, RecursiveMode, Watcher};
                use std::sync::{Arc, Mutex};
                use std::time::{Duration, Instant};

                let (tx_notify, rx_notify) =
                    std::sync::mpsc::channel::<Result<Event, notify::Error>>();

                let mut watcher =
                    notify::recommended_watcher(tx_notify).expect("Failed to create file watcher");

                watcher
                    .watch(&watch_path, RecursiveMode::NonRecursive)
                    .expect("Failed to watch current.yaml");

                println!("👀 Watching for changes to {:?}", watch_path);

                let runtime = tokio::runtime::Handle::current();

                // Debounce mechanism to prevent duplicate events
                let last_reload = Arc::new(Mutex::new(Instant::now()));
                let debounce_duration = Duration::from_millis(100);

                for res in rx_notify {
                    match res {
                        Ok(event) => {
                            // Check if it's a modify event with data changes
                            if matches!(
                                event.kind,
                                EventKind::Modify(notify::event::ModifyKind::Data(_))
                            ) {
                                // Debounce: only reload if enough time has passed since last reload
                                let mut last = last_reload.lock().unwrap();
                                let now = Instant::now();

                                if now.duration_since(*last) > debounce_duration {
                                    *last = now;
                                    drop(last); // Release lock before async operation

                                    println!("📝 Detected change in current.yaml, reloading...");

                                    let state = watcher_state.clone();
                                    let path = watch_path.clone();

                                    runtime.spawn(async move {
                                        match state.reload_from_file(&path).await {
                                            Ok(_) => println!("✅ Config reloaded successfully"),
                                            Err(e) => {
                                                eprintln!("❌ Failed to reload config: {}", e)
                                            }
                                        }
                                    });
                                }
                            }
                        }
                        Err(e) => eprintln!("⚠️ Watch error: {}", e),
                    }
                }
            });

            // If interval > 0, spawn update task
            if interval > 0 {
                let duration = std::time::Duration::from_secs(interval);
                let _update_state = state.clone();

                tokio::spawn(async move {
                    loop {
                        tokio::time::sleep(duration).await;
                        println!("⏰ Auto-updating subscriptions...");
                        // Reload subs in case they changed
                        let mut subs = storage::load_subscriptions().unwrap_or_default();
                        let mut any_updated = false;

                        for sub in &mut subs {
                            if sub.enabled {
                                match subscription::download_subscription(sub).await {
                                    Ok(_) => {
                                        sub.last_updated = Some(
                                            chrono::Local::now()
                                                .format("%Y-%m-%d %H:%M:%S")
                                                .to_string(),
                                        );
                                        any_updated = true;
                                    }
                                    Err(e) => {
                                        println!(
                                            "⚠️ Failed to update subscription {}: {}",
                                            sub.name, e
                                        )
                                    }
                                }
                            }
                        }

                        if any_updated {
                            if let Err(e) = storage::save_subscriptions(&subs) {
                                println!("❌ Failed to save updated subscriptions: {}", e);
                            }
                        }

                        // Re-merge
                        match proxy::merge_configs(&subs).await {
                            Ok(new_config) => {
                                // Save to current.yaml
                                let _ = storage::get_current_config_path().map(|p| {
                                    hangar_lib::config::save_config(
                                        &new_config,
                                        p.to_str().unwrap(),
                                    )
                                });
                                println!("✅ Auto-merge complete.");
                                // The file watcher will automatically reload the config
                            }
                            Err(e) => println!("❌ Auto-merge failed: {}", e),
                        }
                    }
                });
            }

            // Start server with the initial state
            let addr = format!("{}:{}", host, port);
            let listener = tokio::net::TcpListener::bind(&addr).await?;

            println!("\n🌐 Server started successfully");
            println!("   Address: http://{}", addr);
            println!("   Config URL: http://{}/config", addr);
            println!("\n✨ Waiting for requests...\n");

            let app =
                axum::Router::new()
                    .route(
                        "/config",
                        axum::routing::get(
                            |axum::extract::State(state): axum::extract::State<
                                server::AppState,
                            >| async move {
                                let config = state.config.read().await;
                                match serde_yaml::to_string(&*config) {
                                    Ok(yaml) => (axum::http::StatusCode::OK, yaml).into_response(),
                                    Err(e) => (
                                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                        format!("Failed to serialize config: {}", e),
                                    )
                                        .into_response(),
                                }
                            },
                        ),
                    )
                    .route("/health", axum::routing::get(|| async { "OK" }))
                    .with_state(state);

            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    let _ = rx.await;
                    eprintln!("🛑 Shutting down server...");
                })
                .await?;
        }
        Commands::Stop => {
            let pid_path = storage::get_server_pid_path()?;
            if !pid_path.exists() {
                println!("❌ Server is not running (no PID file found)");
                return Ok(());
            }

            let pid_str = std::fs::read_to_string(&pid_path)?;
            let pid = pid_str
                .trim()
                .parse::<i32>()
                .context("Failed to parse PID")?;

            println!("🛑 Stopping server (PID: {})...", pid);

            #[cfg(unix)]
            {
                use std::time::{Duration, Instant};
                let res = unsafe { libc::kill(pid, 15) }; // SIGTERM
                if res == 0 {
                    // Wait for it to stop
                    let start = Instant::now();
                    while start.elapsed() < Duration::from_secs(5) {
                        if unsafe { libc::kill(pid, 0) } != 0 {
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(200)).await;
                    }

                    if unsafe { libc::kill(pid, 0) } == 0 {
                        println!("⚠️ Server didn't stop gracefully, force killing...");
                        unsafe { libc::kill(pid, 9) }; // SIGKILL
                    }

                    let _ = std::fs::remove_file(&pid_path);
                    println!("✅ Server stopped");
                } else {
                    println!("❌ Failed to stop server: process not found or no permission");
                    let _ = std::fs::remove_file(&pid_path);
                }
            }

            #[cfg(not(unix))]
            {
                println!("❌ Stop command is currently only supported on Unix-like systems.");
            }
        }
        Commands::Restart => {
            // Stop
            let pid_path = storage::get_server_pid_path()?;
            if pid_path.exists() {
                let pid_str = std::fs::read_to_string(&pid_path)?;
                if let Ok(pid) = pid_str.trim().parse::<i32>() {
                    println!("🛑 Stopping server (PID: {})...", pid);
                    #[cfg(unix)]
                    {
                        unsafe { libc::kill(pid, 15) };
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                }
                let _ = std::fs::remove_file(&pid_path);
            }

            // Start - we need to know the default port/host or previous ones?
            // For simplicity, we just run 'serve --daemon' with defaults or we could try to persist last used args.
            // But usually defaults are fine or user can just run start again.
            // Let's just trigger the same logic as Serve but with default args.
            println!("🚀 Restarting server...");
            let exe = std::env::current_exe()?;
            std::process::Command::new(&exe)
                .arg("serve")
                .arg("--daemon")
                .spawn()?;
            println!("✅ Restart initiated.");
        }
        Commands::Status => {
            let pid_path = storage::get_server_pid_path()?;
            let log_path = storage::get_server_log_path()?;

            if !pid_path.exists() {
                println!("❌ Server is not running");
                return Ok(());
            }

            let pid_str = std::fs::read_to_string(&pid_path)?;
            if let Ok(pid) = pid_str.trim().parse::<i32>() {
                #[cfg(unix)]
                if unsafe { libc::kill(pid, 0) } == 0 {
                    println!("✅ Server is running (PID: {})", pid);
                    println!("   Log file: {:?}", log_path);
                    return Ok(());
                }
                println!("❌ Server is not running (stale PID file)");
                let _ = std::fs::remove_file(&pid_path);
            } else {
                println!("❌ Invalid PID file content");
            }
        }
        Commands::Logs { follow } => {
            let log_path = storage::get_server_log_path()?;
            if !log_path.exists() {
                println!("❌ Log file not found: {:?}", log_path);
                return Ok(());
            }

            if follow {
                let mut child = std::process::Command::new("tail")
                    .arg("-f")
                    .arg(&log_path)
                    .spawn()?;
                child.wait()?;
            } else {
                let output = std::process::Command::new("tail")
                    .arg("-n")
                    .arg("50")
                    .arg(&log_path)
                    .output()?;
                println!("{}", String::from_utf8_lossy(&output.stdout));
            }
        }
        Commands::Ai { prompt } => {
            println!("🤖 Processing AI request: \"{}\"", prompt);
            let result = ai::generate_config_patch(&prompt).await?;
            println!("✨ AI suggested changes for [{}]:", result.target);
            println!("{}", serde_json::to_string_pretty(&result.operations)?);

            let target_file = match result.target.as_str() {
                "basic" => storage::get_basic_config_path()?,
                "groups" => storage::get_groups_config_path()?,
                _ => storage::get_current_config_path()?,
            };

            if target_file.exists() {
                let content = std::fs::read_to_string(&target_file)?;
                let patched = ai::apply_patch_to_config(&content, &result.operations)?;
                version::save_version(&result.target, "ai_update", &content)?;
                std::fs::write(&target_file, patched)?;
                println!(
                    "✅ Applied changes to {:?} and created backup.",
                    target_file
                );

                // If we modified basic or groups, we should re-merge
                if result.target == "basic" || result.target == "groups" {
                    println!("🔄 Triggering merge after base config change...");
                    let subs = storage::load_subscriptions().unwrap_or_default();
                    let merged = proxy::merge_configs(&subs).await?;
                    let output_path = storage::get_current_config_path()?;
                    hangar_lib::config::save_config(&merged, output_path.to_str().unwrap())?;
                    println!("✅ Config regenerated.");
                }
            } else {
                println!("⚠️ Target file not found: {:?}", target_file);
            }
        }
        Commands::History { subcommand } => match subcommand.unwrap_or(HistoryCommands::List) {
            HistoryCommands::List => {
                let versions = version::list_versions()?;
                println!("{:<40} {:<20} {}", "ID", "Time", "Description");
                for v in versions {
                    println!("{:<40} {:<20} {}", v.id, v.timestamp, v.description);
                }
            }
            HistoryCommands::Rollback { id } => {
                // Resolve alias first (though rollback_to_version does it too, explicit feedback is nice)
                match version::resolve_version_id(&id) {
                    Ok(resolved) => match version::rollback_to_version(&resolved) {
                        Ok(_) => println!("✅ Rolled back to {} ({})", id, resolved),
                        Err(e) => println!("❌ Rollback failed: {}", e),
                    },
                    Err(e) => println!("❌ Invalid version ID or alias: {}", e),
                }
            }
            HistoryCommands::Diff { v1, v2 } => {
                let resolved_v1 = match version::resolve_version_id(&v1) {
                    Ok(v) => v,
                    Err(e) => {
                        println!("❌ Invalid version v1: {}", e);
                        return Ok(());
                    }
                };

                // Get content for v1
                let content1 = version::get_version_content(&resolved_v1)?;

                // Determine content for v2
                let (resolved_v2, content2) = if let Some(id) = v2 {
                    let r_v2 = match version::resolve_version_id(&id) {
                        Ok(v) => v,
                        Err(e) => {
                            println!("❌ Invalid version v2: {}", e);
                            return Ok(());
                        }
                    };
                    let c = version::get_version_content(&r_v2)?;
                    (r_v2, c)
                } else {
                    // Default to current config if v2 is not provided
                    let path = storage::get_current_config_path()?;
                    if path.exists() {
                        ("current.yaml".to_string(), std::fs::read_to_string(&path)?)
                    } else {
                        ("current (empty)".to_string(), "".to_string())
                    }
                };

                println!("📊 Diffing {} vs {}...", resolved_v1, resolved_v2);

                let diff = version::diff_configs(&content1, &content2);
                if diff.is_empty() {
                    println!("No changes found.");
                } else {
                    for line in diff {
                        match line.line_type.as_str() {
                            "added" => println!("+ {}", line.content),
                            "removed" => println!("- {}", line.content),
                            _ => println!("  {}", line.content),
                        }
                    }
                }
            }
        },
        Commands::Editor => {
            let path = storage::get_current_config_path()?;
            println!("📝 Opening default editor for {:?}", path);
            std::process::Command::new("open")
                .arg(&path)
                .spawn()
                .context("Failed to open editor")?
                .wait()
                .context("Failed to wait for editor")?;
            println!("✅ Edit closed.");
        }
        Commands::Config {
            api_key,
            base_url,
            model,
        } => {
            let mut config = storage::load_hangar_config().unwrap_or_default();

            if api_key.is_none() && base_url.is_none() && model.is_none() {
                println!("{}", serde_json::to_string_pretty(&config)?);
                return Ok(());
            }

            if let Some(k) = api_key {
                config.llm.api_key = k;
            }
            if let Some(u) = base_url {
                config.llm.base_url = u;
            }
            if let Some(m) = model {
                config.llm.model = m;
            }
            storage::save_hangar_config(&config)?;
            println!("✅ Configuration updated.");
        }
    }

    Ok(())
}
