use anyhow::{Context, Result};
use std::path::PathBuf;
use std::fs;
use crate::types::{HangarConfig, SubscriptionList, Subscription};

/// Get the Hangar data directory path (~/.hangar/)
pub fn get_hangar_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Failed to get home directory")?;
    let hangar_dir = home.join(".hangar");

    // Ensure directory exists
    fs::create_dir_all(&hangar_dir)
        .context("Failed to create .hangar directory")?;

    Ok(hangar_dir)
}

/// Get the path to config.json
pub fn get_config_path() -> Result<PathBuf> {
    Ok(get_hangar_dir()?.join("config.json"))
}

/// Get the path to subscriptions.json
pub fn get_subscriptions_path() -> Result<PathBuf> {
    Ok(get_hangar_dir()?.join("subscriptions.json"))
}

/// Get the path to current.yaml
pub fn get_current_config_path() -> Result<PathBuf> {
    Ok(get_hangar_dir()?.join("current.yaml"))
}

/// Get the versions directory path
pub fn get_versions_dir() -> Result<PathBuf> {
    let dir = get_hangar_dir()?.join("versions");
    fs::create_dir_all(&dir)
        .context("Failed to create versions directory")?;
    Ok(dir)
}

/// Get the cache directory path
pub fn get_cache_dir() -> Result<PathBuf> {
    let dir = get_hangar_dir()?.join("cache");
    fs::create_dir_all(&dir)
        .context("Failed to create cache directory")?;
    Ok(dir)
}

/// Get the proxies cache directory
pub fn get_proxies_cache_dir() -> Result<PathBuf> {
    let dir = get_cache_dir()?.join("proxies");
    fs::create_dir_all(&dir)
        .context("Failed to create proxies cache directory")?;
    Ok(dir)
}

/// Get the rules cache directory
pub fn get_rules_cache_dir() -> Result<PathBuf> {
    let dir = get_cache_dir()?.join("rules");
    fs::create_dir_all(&dir)
        .context("Failed to create rules cache directory")?;
    Ok(dir)
}

/// Load Hangar configuration
pub fn load_hangar_config() -> Result<HangarConfig> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        // Create default config
        let default_config = HangarConfig::default();
        save_hangar_config(&default_config)?;
        return Ok(default_config);
    }

    let content = fs::read_to_string(&config_path)
        .context("Failed to read config.json")?;
    let config: HangarConfig = serde_json::from_str(&content)
        .context("Failed to parse config.json")?;

    Ok(config)
}

/// Save Hangar configuration
pub fn save_hangar_config(config: &HangarConfig) -> Result<()> {
    let config_path = get_config_path()?;
    let content = serde_json::to_string_pretty(config)
        .context("Failed to serialize config")?;
    fs::write(&config_path, content)
        .context("Failed to write config.json")?;
    Ok(())
}

/// Load subscriptions list
pub fn load_subscriptions() -> Result<Vec<Subscription>> {
    let subs_path = get_subscriptions_path()?;

    if !subs_path.exists() {
        // Create default subscriptions file
        let default_list = SubscriptionList {
            subscriptions: vec![],
        };
        save_subscriptions(&default_list.subscriptions)?;
        return Ok(default_list.subscriptions);
    }

    let content = fs::read_to_string(&subs_path)
        .context("Failed to read subscriptions.json")?;
    let list: SubscriptionList = serde_json::from_str(&content)
        .context("Failed to parse subscriptions.json")?;

    Ok(list.subscriptions)
}

/// Save subscriptions list
pub fn save_subscriptions(subscriptions: &[Subscription]) -> Result<()> {
    let subs_path = get_subscriptions_path()?;
    let list = SubscriptionList {
        subscriptions: subscriptions.to_vec(),
    };
    let content = serde_json::to_string_pretty(&list)
        .context("Failed to serialize subscriptions")?;
    fs::write(&subs_path, content)
        .context("Failed to write subscriptions.json")?;
    Ok(())
}
