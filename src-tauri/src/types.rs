use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    pub name: String,
    pub url: String,
    pub enabled: bool,
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
