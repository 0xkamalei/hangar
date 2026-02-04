use crate::types::{ProxyNode, Subscription};
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use serde_json::Value;
use std::collections::HashMap;

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
    ];

    for (cn, en) in regions {
        if name.contains(cn) || name.to_uppercase().contains(en) {
            return Some(en.to_string());
        }
    }
    None
}
