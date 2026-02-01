use crate::types::{ProxyNode, Subscription};
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use serde_json::Value;
use std::collections::HashMap;

pub async fn fetch_subscription(sub: &Subscription) -> Result<Vec<ProxyNode>> {
    let response = reqwest::get(&sub.url).await?;
    let content = response.text().await?;
    
    // 尝试 base64 解码
    let decoded = if let Ok(decoded_bytes) = general_purpose::STANDARD.decode(&content) {
        String::from_utf8(decoded_bytes)?
    } else {
        content
    };
    
    // 解析 YAML
    let config: HashMap<String, Value> = serde_yaml::from_str(&decoded)?;
    
    let mut proxies = Vec::new();
    if let Some(Value::Array(proxy_list)) = config.get("proxies") {
        for proxy in proxy_list {
            if let Ok(mut node) = serde_json::from_value::<ProxyNode>(proxy.clone()) {
                // 在节点名称前加上机场名
                node.name = format!("[{}] {}", sub.name, node.name);
                node.airport = sub.name.clone();
                proxies.push(node);
            }
        }
    }
    
    Ok(proxies)
}

pub fn extract_region(name: &str) -> Option<String> {
    let regions = vec![
        ("香港", "HK"), ("台湾", "TW"), ("日本", "JP"), ("新加坡", "SG"),
        ("美国", "US"), ("英国", "UK"), ("韩国", "KR"), ("德国", "DE"),
        ("加拿大", "CA"), ("印度", "IN"), ("马来西亚", "MY"), ("土耳其", "TR"),
        ("阿根廷", "AR"), ("俄罗斯", "RU"), ("越南", "VN"), ("乌克兰", "UA"),
        ("尼日利亚", "NG"),
    ];
    
    for (cn, en) in regions {
        if name.contains(cn) || name.to_uppercase().contains(en) {
            return Some(en.to_string());
        }
    }
    None
}
