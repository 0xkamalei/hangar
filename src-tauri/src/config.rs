use crate::types::{ClashConfig, Subscription};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputConfig {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BasicConfigPath {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub subscriptions: Vec<Subscription>,
    pub server: ServerConfig,
    pub output: OutputConfig,
    pub basic_config: BasicConfigPath,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "127.0.0.1".to_string(),
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            path: "output_config.yaml".to_string(),
        }
    }
}

impl Default for BasicConfigPath {
    fn default() -> Self {
        Self {
            path: "basic.yml".to_string(),
        }
    }
}

/// 加载应用配置
pub fn load_app_config(path: &str) -> Result<AppConfig> {
    let content = fs::read_to_string(path)?;
    let config: AppConfig = serde_json::from_str(&content)?;
    Ok(config)
}

/// 加载基础 Clash 配置
pub fn load_basic_config(path: &str) -> Result<ClashConfig> {
    let content = fs::read_to_string(path)?;
    let config: ClashConfig = serde_yaml::from_str(&content)?;
    Ok(config)
}

/// 保存合并后的配置
pub fn save_config(config: &ClashConfig, path: &str) -> Result<()> {
    let yaml = serde_yaml::to_string(config)?;
    fs::write(path, yaml)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "127.0.0.1");
    }

    #[test]
    fn test_output_config_default() {
        let config = OutputConfig::default();
        assert_eq!(config.path, "output_config.yaml");
    }

    #[test]
    fn test_basic_config_path_default() {
        let config = BasicConfigPath::default();
        assert_eq!(config.path, "basic.yml");
    }

    #[test]
    fn test_load_app_config_success() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.json");

        let test_config = r#"{
            "subscriptions": [
                {
                    "name": "Test Sub",
                    "url": "https://example.com/sub",
                    "enabled": true
                }
            ],
            "server": {
                "port": 9090,
                "host": "0.0.0.0"
            },
            "output": {
                "path": "test_output.yaml"
            },
            "basic_config": {
                "path": "test_basic.yml"
            }
        }"#;

        fs::write(&config_path, test_config).unwrap();

        let config = load_app_config(config_path.to_str().unwrap()).unwrap();
        assert_eq!(config.subscriptions.len(), 1);
        assert_eq!(config.subscriptions[0].name, "Test Sub");
        assert_eq!(config.server.port, 9090);
        assert_eq!(config.server.host, "0.0.0.0");
    }

    #[test]
    fn test_load_app_config_file_not_found() {
        let result = load_app_config("nonexistent.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_app_config_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid.json");
        fs::write(&config_path, "invalid json content").unwrap();

        let result = load_app_config(config_path.to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_save_and_load_config() {
        use std::collections::HashMap;

        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test_output.yaml");

        let config = ClashConfig {
            base_config: HashMap::new(),
            proxies: vec![],
            proxy_groups: vec![],
            rules: vec![],
            rule_providers: None,
        };

        // 保存配置
        let result = save_config(&config, output_path.to_str().unwrap());
        assert!(result.is_ok());

        // 验证文件存在
        assert!(output_path.exists());

        // 读取配置
        let loaded_config = load_basic_config(output_path.to_str().unwrap()).unwrap();
        assert_eq!(loaded_config.proxies.len(), 0);
        assert_eq!(loaded_config.proxy_groups.len(), 0);
    }
}
