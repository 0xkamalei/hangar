use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::storage::get_hangar_dir;

/// Built-in rule definitions from Loyalsoldier/clash-rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltinRule {
    pub name: String,
    pub description: String,
    pub url: String,
    pub behavior: String, // domain, ipcidr, classical
}

/// Rule source for online rule subscriptions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSource {
    pub id: String,
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub last_updated: Option<String>,
}

/// Default built-in rules from Loyalsoldier/clash-rules
pub fn get_default_builtin_rules() -> Vec<BuiltinRule> {
    vec![
        BuiltinRule {
            name: "reject".to_string(),
            description: "广告域名".to_string(),
            url: "https://cdn.jsdelivr.net/gh/Loyalsoldier/clash-rules@release/reject.txt".to_string(),
            behavior: "domain".to_string(),
        },
        BuiltinRule {
            name: "proxy".to_string(),
            description: "代理域名".to_string(),
            url: "https://cdn.jsdelivr.net/gh/Loyalsoldier/clash-rules@release/proxy.txt".to_string(),
            behavior: "domain".to_string(),
        },
        BuiltinRule {
            name: "direct".to_string(),
            description: "直连域名".to_string(),
            url: "https://cdn.jsdelivr.net/gh/Loyalsoldier/clash-rules@release/direct.txt".to_string(),
            behavior: "domain".to_string(),
        },
        BuiltinRule {
            name: "gfw".to_string(),
            description: "GFW 封锁域名".to_string(),
            url: "https://cdn.jsdelivr.net/gh/Loyalsoldier/clash-rules@release/gfw.txt".to_string(),
            behavior: "domain".to_string(),
        },
        BuiltinRule {
            name: "greatfire".to_string(),
            description: "GreatFire 域名".to_string(),
            url: "https://cdn.jsdelivr.net/gh/Loyalsoldier/clash-rules@release/greatfire.txt".to_string(),
            behavior: "domain".to_string(),
        },
        BuiltinRule {
            name: "tld-not-cn".to_string(),
            description: "非中国顶级域".to_string(),
            url: "https://cdn.jsdelivr.net/gh/Loyalsoldier/clash-rules@release/tld-not-cn.txt".to_string(),
            behavior: "domain".to_string(),
        },
        BuiltinRule {
            name: "telegramcidr".to_string(),
            description: "Telegram IP 段".to_string(),
            url: "https://cdn.jsdelivr.net/gh/Loyalsoldier/clash-rules@release/telegramcidr.txt".to_string(),
            behavior: "ipcidr".to_string(),
        },
        BuiltinRule {
            name: "cncidr".to_string(),
            description: "中国 IP 段".to_string(),
            url: "https://cdn.jsdelivr.net/gh/Loyalsoldier/clash-rules@release/cncidr.txt".to_string(),
            behavior: "ipcidr".to_string(),
        },
        BuiltinRule {
            name: "lancidr".to_string(),
            description: "局域网 IP 段".to_string(),
            url: "https://cdn.jsdelivr.net/gh/Loyalsoldier/clash-rules@release/lancidr.txt".to_string(),
            behavior: "ipcidr".to_string(),
        },
        BuiltinRule {
            name: "applications".to_string(),
            description: "需代理的程序".to_string(),
            url: "https://cdn.jsdelivr.net/gh/Loyalsoldier/clash-rules@release/applications.txt".to_string(),
            behavior: "classical".to_string(),
        },
    ]
}

/// Get the builtin rules cache directory
pub fn get_builtin_rules_cache_dir() -> Result<PathBuf> {
    let dir = get_hangar_dir()?.join("cache").join("rules").join("builtin");
    fs::create_dir_all(&dir).context("Failed to create builtin rules cache directory")?;
    Ok(dir)
}

/// Get the remote rules cache directory
pub fn get_remote_rules_cache_dir() -> Result<PathBuf> {
    let dir = get_hangar_dir()?.join("cache").join("rules").join("remote");
    fs::create_dir_all(&dir).context("Failed to create remote rules cache directory")?;
    Ok(dir)
}

/// Get the rule sources file path
fn get_rule_sources_path() -> Result<PathBuf> {
    Ok(get_hangar_dir()?.join("rule_sources.json"))
}

/// Load rule sources from disk
pub fn load_rule_sources() -> Result<Vec<RuleSource>> {
    let path = get_rule_sources_path()?;

    if !path.exists() {
        return Ok(vec![]);
    }

    let content = fs::read_to_string(&path)
        .context("Failed to read rule_sources.json")?;
    let sources: Vec<RuleSource> = serde_json::from_str(&content)
        .context("Failed to parse rule_sources.json")?;

    Ok(sources)
}

/// Save rule sources to disk
pub fn save_rule_sources(sources: &[RuleSource]) -> Result<()> {
    let path = get_rule_sources_path()?;
    let content = serde_json::to_string_pretty(sources)
        .context("Failed to serialize rule sources")?;
    fs::write(&path, content)
        .context("Failed to write rule_sources.json")?;
    Ok(())
}

/// Add a new rule source
pub fn add_rule_source(name: String, url: String) -> Result<RuleSource> {
    let mut sources = load_rule_sources()?;

    // Check for duplicate URL
    if sources.iter().any(|s| s.url == url) {
        anyhow::bail!("Rule source with this URL already exists");
    }

    let source = RuleSource {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        url,
        enabled: true,
        last_updated: None,
    };

    sources.push(source.clone());
    save_rule_sources(&sources)?;

    Ok(source)
}

/// Remove a rule source by ID
pub fn remove_rule_source(id: &str) -> Result<()> {
    let mut sources = load_rule_sources()?;
    let initial_len = sources.len();
    sources.retain(|s| s.id != id);

    if sources.len() == initial_len {
        anyhow::bail!("Rule source not found");
    }

    save_rule_sources(&sources)?;
    Ok(())
}

/// Download and cache a rule file
async fn download_rule(url: &str, cache_path: &PathBuf) -> Result<()> {
    eprintln!("Downloading rule from: {}", url);

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "Hangar/1.0")
        .send()
        .await
        .context("Failed to fetch rule")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download rule: HTTP {}", response.status());
    }

    let content = response.text().await.context("Failed to read response")?;
    fs::write(cache_path, &content).context("Failed to write cache file")?;

    eprintln!("Rule cached to: {:?}", cache_path);
    Ok(())
}

/// Refresh all builtin rules
pub async fn refresh_builtin_rules() -> Result<()> {
    let rules = get_default_builtin_rules();
    let cache_dir = get_builtin_rules_cache_dir()?;

    for rule in rules {
        let cache_path = cache_dir.join(format!("{}.txt", rule.name));
        if let Err(e) = download_rule(&rule.url, &cache_path).await {
            eprintln!("Failed to download rule {}: {}", rule.name, e);
        }
    }

    Ok(())
}

/// Refresh all custom rule sources
pub async fn refresh_custom_rules() -> Result<()> {
    let mut sources = load_rule_sources()?;
    let cache_dir = get_remote_rules_cache_dir()?;
    let now = chrono::Utc::now().to_rfc3339();

    for source in &mut sources {
        if !source.enabled {
            continue;
        }

        let cache_path = cache_dir.join(format!("{}.txt", source.id));
        match download_rule(&source.url, &cache_path).await {
            Ok(_) => {
                source.last_updated = Some(now.clone());
            }
            Err(e) => {
                eprintln!("Failed to refresh rule source {}: {}", source.name, e);
            }
        }
    }

    save_rule_sources(&sources)?;
    Ok(())
}

/// Refresh all rules (builtin + custom)
pub async fn refresh_all_rules() -> Result<()> {
    refresh_builtin_rules().await?;
    refresh_custom_rules().await?;
    Ok(())
}

/// Generate rule-providers section for Clash config
pub fn generate_rule_providers() -> Result<HashMap<String, serde_yaml::Value>> {
    let mut providers = HashMap::new();
    let builtin_rules = get_default_builtin_rules();

    for rule in builtin_rules {
        let provider = serde_yaml::to_value(&RuleProviderConfig {
            provider_type: "http".to_string(),
            behavior: rule.behavior.clone(),
            url: rule.url.clone(),
            path: format!("./ruleset/{}.yaml", rule.name),
            interval: 86400,
        }).context("Failed to serialize rule provider")?;

        providers.insert(rule.name, provider);
    }

    // Add custom rule sources
    let sources = load_rule_sources().unwrap_or_default();
    for source in sources {
        if !source.enabled {
            continue;
        }

        // Try to guess behavior from URL
        let behavior = if source.url.contains("cidr") || source.url.contains("ip") {
            "ipcidr"
        } else {
            "domain"
        };

        let provider = serde_yaml::to_value(&RuleProviderConfig {
            provider_type: "http".to_string(),
            behavior: behavior.to_string(),
            url: source.url.clone(),
            path: format!("./ruleset/custom_{}.yaml", source.id),
            interval: 86400,
        }).context("Failed to serialize custom rule provider")?;

        // Use sanitized name as key
        let key = source.name.to_lowercase().replace(' ', "_");
        providers.insert(key, provider);
    }

    Ok(providers)
}

/// Helper struct for rule provider serialization
#[derive(Debug, Serialize, Deserialize)]
struct RuleProviderConfig {
    #[serde(rename = "type")]
    provider_type: String,
    behavior: String,
    url: String,
    path: String,
    interval: u64,
}

/// Generate default rules using rule-providers
pub fn generate_default_rules() -> Vec<String> {
    vec![
        "RULE-SET,reject,REJECT".to_string(),
        "RULE-SET,proxy,Proxy".to_string(),
        "RULE-SET,direct,DIRECT".to_string(),
        "RULE-SET,telegramcidr,Proxy".to_string(),
        "GEOIP,CN,DIRECT".to_string(),
        "MATCH,Proxy".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_default_builtin_rules() {
        let rules = get_default_builtin_rules();
        assert_eq!(rules.len(), 10);
        assert!(rules.iter().any(|r| r.name == "reject"));
        assert!(rules.iter().any(|r| r.name == "proxy"));
        assert!(rules.iter().any(|r| r.name == "direct"));
    }

    #[test]
    fn test_generate_rule_providers() {
        let providers = generate_rule_providers().unwrap();
        assert!(providers.contains_key("reject"));
        assert!(providers.contains_key("proxy"));
        assert!(providers.contains_key("direct"));
    }
}
