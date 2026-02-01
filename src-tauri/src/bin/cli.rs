// 命令行工具 - 快速测试订阅合并
// 使用方法: cargo run --bin cli -- <subs_file> <output_file>

use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Subscription {
    name: String,
    url: String,
    enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProxyNode {
    name: String,
    #[serde(rename = "type")]
    proxy_type: String,
    server: String,
    port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    region: Option<String>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProxyGroup {
    name: String,
    #[serde(rename = "type")]
    group_type: String,
    proxies: Vec<String>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClashConfig {
    #[serde(flatten)]
    base_config: HashMap<String, serde_yaml::Value>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    proxies: Vec<ProxyNode>,
    #[serde(rename = "proxy-groups", skip_serializing_if = "Vec::is_empty", default)]
    proxy_groups: Vec<ProxyGroup>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    rules: Vec<String>,
}

async fn fetch_subscription(url: &str, name: &str) -> Result<Vec<ProxyNode>> {
    println!("📡 获取订阅: {}", name);
    let response = reqwest::get(url).await?;
    let content = response.text().await?;
    
    // 尝试 base64 解码
    let decoded = if let Ok(decoded_bytes) = general_purpose::STANDARD.decode(&content) {
        String::from_utf8(decoded_bytes)?
    } else {
        content
    };
    
    // 解析 YAML
    let config: HashMap<String, serde_json::Value> = serde_yaml::from_str(&decoded)?;
    
    let mut proxies = Vec::new();
    if let Some(serde_json::Value::Array(proxy_list)) = config.get("proxies") {
        for proxy in proxy_list {
            if let Ok(mut node) = serde_json::from_value::<ProxyNode>(proxy.clone()) {
                // 在节点名称前加上机场名
                node.name = format!("[{}] {}", name, node.name);
                
                // 提取地区
                node.region = extract_region(&node.name);
                
                proxies.push(node);
            }
        }
    }
    
    println!("  ✓ 获取到 {} 个节点", proxies.len());
    Ok(proxies)
}

fn extract_region(name: &str) -> Option<String> {
    let regions = vec![
        ("香港", "HK"), ("HK", "HK"),
        ("台湾", "TW"), ("TW", "TW"), ("台", "TW"),
        ("日本", "JP"), ("JP", "JP"),
        ("新加坡", "SG"), ("SG", "SG"), ("狮城", "SG"),
        ("美国", "US"), ("US", "US"),
        ("英国", "UK"), ("UK", "UK"),
        ("韩国", "KR"), ("KR", "KR"),
        ("德国", "DE"), ("DE", "DE"),
        ("加拿大", "CA"), ("CA", "CA"),
        ("印度", "IN"), ("IN", "IN"),
        ("马来西亚", "MY"), ("MY", "MY"),
        ("土耳其", "TR"), ("TR", "TR"),
        ("阿根廷", "AR"), ("AR", "AR"),
        ("俄罗斯", "RU"), ("RU", "RU"),
        ("越南", "VN"), ("VN", "VN"),
        ("乌克兰", "UA"), ("UA", "UA"),
        ("尼日利亚", "NG"), ("NG", "NG"),
    ];
    
    let upper_name = name.to_uppercase();
    for (pattern, code) in regions {
        if name.contains(pattern) || upper_name.contains(&pattern.to_uppercase()) {
            return Some(code.to_string());
        }
    }
    None
}

fn create_region_groups(proxies: &[ProxyNode]) -> Vec<ProxyGroup> {
    let mut region_map: HashMap<String, Vec<String>> = HashMap::new();
    
    for proxy in proxies {
        if let Some(region) = &proxy.region {
            region_map
                .entry(region.clone())
                .or_default()
                .push(proxy.name.clone());
        }
    }
    
    let mut groups = Vec::new();
    for (region, proxy_names) in region_map {
        if !proxy_names.is_empty() {
            groups.push(ProxyGroup {
                name: format!("{} 地区", region),
                group_type: "select".to_string(),
                proxies: proxy_names,
                extra: HashMap::new(),
            });
        }
    }
    
    groups
}

fn create_service_groups(all_proxy_names: &[String], proxies: &[ProxyNode]) -> Vec<ProxyGroup> {
    let mut groups = Vec::new();
    
    // 节点选择组
    groups.push(ProxyGroup {
        name: "节点选择".to_string(),
        group_type: "select".to_string(),
        proxies: all_proxy_names.to_vec(),
        extra: HashMap::new(),
    });
    
    // ChatGPT 组 (优选美国、英国、新加坡、台湾)
    let chatgpt_regions = ["US", "UK", "SG", "TW"];
    let chatgpt_proxies: Vec<String> = proxies
        .iter()
        .filter(|p| {
            p.region
                .as_ref()
                .map(|r| chatgpt_regions.contains(&r.as_str()))
                .unwrap_or(false)
        })
        .map(|p| p.name.clone())
        .collect();
    
    if !chatgpt_proxies.is_empty() {
        groups.push(ProxyGroup {
            name: "ChatGPT".to_string(),
            group_type: "select".to_string(),
            proxies: chatgpt_proxies,
            extra: HashMap::new(),
        });
    }
    
    // Gemini 组 (优选美国、英国、新加坡、香港、台湾)
    let gemini_regions = ["US", "UK", "SG", "HK", "TW"];
    let gemini_proxies: Vec<String> = proxies
        .iter()
        .filter(|p| {
            p.region
                .as_ref()
                .map(|r| gemini_regions.contains(&r.as_str()))
                .unwrap_or(false)
        })
        .map(|p| p.name.clone())
        .collect();
    
    if !gemini_proxies.is_empty() {
        groups.push(ProxyGroup {
            name: "Gemini".to_string(),
            group_type: "select".to_string(),
            proxies: gemini_proxies,
            extra: HashMap::new(),
        });
    }
    
    groups
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("使用方法: {} <subs_file> [output_file]", args[0]);
        eprintln!("\n示例:");
        eprintln!("  {} subs.txt", args[0]);
        eprintln!("  {} subs.txt clash.yml", args[0]);
        std::process::exit(1);
    }
    
    let subs_file = &args[1];
    let output_file = if args.len() > 2 {
        &args[2]
    } else {
        "clash.yml"
    };
    
    println!("🚀 代理订阅合并工具\n");
    println!("📄 读取订阅文件: {}", subs_file);
    
    // 读取订阅文件
    let content = fs::read_to_string(subs_file)?;
    let urls: Vec<&str> = content.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    
    println!("✓ 找到 {} 个订阅\n", urls.len());
    
    // 获取所有订阅
    let mut all_proxies = Vec::new();
    for (i, url) in urls.iter().enumerate() {
        let name = format!("机场{}", i + 1);
        match fetch_subscription(url, &name).await {
            Ok(proxies) => {
                all_proxies.extend(proxies);
            }
            Err(e) => {
                eprintln!("  ✗ 获取失败: {}", e);
            }
        }
    }
    
    println!("\n📊 统计信息:");
    println!("  总节点数: {}", all_proxies.len());
    
    // 统计地区分布
    let mut region_count: HashMap<String, usize> = HashMap::new();
    for proxy in &all_proxies {
        if let Some(region) = &proxy.region {
            *region_count.entry(region.clone()).or_insert(0) += 1;
        }
    }
    
    println!("\n🌍 地区分布:");
    let mut regions: Vec<_> = region_count.iter().collect();
    regions.sort_by(|a, b| b.1.cmp(a.1));
    for (region, count) in regions {
        println!("  {}: {} 个节点", region, count);
    }
    
    // 创建代理组
    let all_proxy_names: Vec<String> = all_proxies.iter().map(|p| p.name.clone()).collect();
    let mut proxy_groups = Vec::new();
    
    // 添加地区分组
    let region_groups = create_region_groups(&all_proxies);
    println!("\n🎯 创建了 {} 个地区分组", region_groups.len());
    
    // 添加服务专用组
    let service_groups = create_service_groups(&all_proxy_names, &all_proxies);
    println!("🎯 创建了 {} 个服务专用组", service_groups.len());
    
    proxy_groups.extend(region_groups);
    proxy_groups.extend(service_groups);
    
    // 创建配置
    let mut base_config = HashMap::new();
    base_config.insert("port".to_string(), serde_yaml::Value::Number(7890.into()));
    base_config.insert("socks-port".to_string(), serde_yaml::Value::Number(7891.into()));
    base_config.insert("allow-lan".to_string(), serde_yaml::Value::Bool(false));
    base_config.insert("mode".to_string(), serde_yaml::Value::String("Rule".to_string()));
    base_config.insert("log-level".to_string(), serde_yaml::Value::String("info".to_string()));
    
    let config = ClashConfig {
        base_config,
        proxies: all_proxies,
        proxy_groups,
        rules: vec![
            "DOMAIN-SUFFIX,google.com,节点选择".to_string(),
            "DOMAIN-KEYWORD,openai,ChatGPT".to_string(),
            "DOMAIN-KEYWORD,gemini,Gemini".to_string(),
            "MATCH,DIRECT".to_string(),
        ],
    };
    
    // 保存配置
    let yaml = serde_yaml::to_string(&config)?;
    fs::write(output_file, yaml)?;
    
    println!("\n✅ 配置已保存到: {}", output_file);
    println!("\n💡 使用建议:");
    println!("  1. 在 Clash 中导入 {}", output_file);
    println!("  2. 检查节点连通性");
    println!("  3. 根据需要调整分组和规则");
    
    Ok(())
}
