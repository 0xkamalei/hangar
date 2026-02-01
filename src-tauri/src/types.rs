use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    pub id: String,
    pub name: String,
    pub url: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyNode {
    pub name: String,
    #[serde(rename = "type")]
    pub proxy_type: String,
    pub server: String,
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip)]
    pub airport: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyGroup {
    pub name: String,
    #[serde(rename = "type")]
    pub group_type: String,
    pub proxies: Vec<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClashConfig {
    #[serde(flatten)]
    pub base_config: HashMap<String, serde_yaml::Value>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub proxies: Vec<ProxyNode>,
    #[serde(
        rename = "proxy-groups",
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    pub proxy_groups: Vec<ProxyGroup>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub rules: Vec<String>,
    #[serde(rename = "rule-providers", skip_serializing_if = "Option::is_none")]
    pub rule_providers: Option<HashMap<String, serde_yaml::Value>>,
}

// New data structures for Hangar

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            model: "gpt-4o".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "127.0.0.1".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HangarConfig {
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub rule_sources: Vec<String>,
}

impl Default for HangarConfig {
    fn default() -> Self {
        Self {
            llm: LlmConfig::default(),
            server: ServerConfig::default(),
            rule_sources: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionList {
    pub subscriptions: Vec<Subscription>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigVersion {
    pub id: String,
    pub timestamp: i64,
    pub description: String,
    pub file_path: String,
}
