use crate::types::Subscription;
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};

pub async fn download_subscription(sub: &Subscription) -> Result<std::path::PathBuf> {
    let client = reqwest::Client::new();
    let response = client
        .get(&sub.url)
        .header("User-Agent", "clash-verge/v2.4.5")
        .send()
        .await?;

    let content = response.text().await?;

    // Try to decode base64 if it looks like it
    let decoded_content =
        if let Ok(decoded_bytes) = general_purpose::STANDARD.decode(content.trim()) {
            String::from_utf8(decoded_bytes).unwrap_or(content)
        } else {
            content
        };

    crate::storage::save_proxies_cache(&sub.id, &decoded_content)
}

/// Count proxies in a subscription's cached YAML file
pub fn count_proxies(subscription_id: &str) -> Result<usize> {
    let cache_path = crate::storage::get_subscription_cache_path(subscription_id)?;

    if !cache_path.exists() {
        return Ok(0);
    }

    let content = std::fs::read_to_string(&cache_path)?;

    // Parse YAML to extract proxies
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(&content)?;

    if let Some(proxies) = yaml_value.get("proxies") {
        if let Some(proxy_array) = proxies.as_sequence() {
            return Ok(proxy_array.len());
        }
    }

    Ok(0)
}

// Deprecated: fetch_subscription is removed in favor of download_subscription + local parsing
// We keep a stub if needed or just remove it. Removing it as per plan.

pub fn extract_region(name: &str) -> Option<String> {
    let regions = vec![
        ("香港", "HK"),
        ("台湾", "TW"),
        ("日本", "JP"),
        ("新加坡", "SG"),
        ("美国", "US"),
        ("英国", "UK"),
        ("韩国", "KR"),
        ("德国", "DE"),
        ("加拿大", "CA"),
        ("印度", "IN"),
        ("马来西亚", "MY"),
        ("土耳其", "TR"),
        ("阿根廷", "AR"),
        ("俄罗斯", "RU"),
        ("越南", "VN"),
        ("乌克兰", "UA"),
        ("尼日利亚", "NG"),
        ("法国", "FR"),
        ("澳大利亚", "AU"),
        ("巴西", "BR"),
    ];

    for (cn, en) in regions {
        if name.contains(cn) || name.to_uppercase().contains(en) {
            return Some(en.to_string());
        }
    }
    None
}

/// 判断地区是否支持主流 AI 服务 (Gemini, OpenAI, Claude)
pub fn is_ai_supported_region(region: &str) -> bool {
    let supported = vec![
        "US", "UK", "JP", "SG", "TW", "KR", "DE", "CA", "IN", "FR", "AU", "BR"
    ];
    supported.contains(&region)
}
