use crate::subscription::{extract_region, fetch_subscription};
use crate::types::{ClashConfig, ProxyGroup, ProxyNode, Subscription};
use anyhow::Result;
use std::collections::{HashMap, HashSet};

/// 创建地区分组
pub fn create_region_groups(proxies: &[ProxyNode]) -> Vec<ProxyGroup> {
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

/// 创建服务专用组
pub fn create_service_groups(all_proxy_names: &[String], proxies: &[ProxyNode]) -> Vec<ProxyGroup> {
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
    
    // Google 组
    groups.push(ProxyGroup {
        name: "Google".to_string(),
        group_type: "select".to_string(),
        proxies: all_proxy_names.to_vec(),
        extra: HashMap::new(),
    });
    
    // Netflix 组
    groups.push(ProxyGroup {
        name: "Netflix".to_string(),
        group_type: "select".to_string(),
        proxies: all_proxy_names.to_vec(),
        extra: HashMap::new(),
    });
    
    // Telegram 组
    groups.push(ProxyGroup {
        name: "Telegram".to_string(),
        group_type: "select".to_string(),
        proxies: all_proxy_names.to_vec(),
        extra: HashMap::new(),
    });
    
    groups
}

/// 合并配置
pub async fn merge_configs(
    subscriptions: &[Subscription],
    basic_config: ClashConfig,
) -> Result<ClashConfig> {
    let mut all_proxies = basic_config.proxies.clone();
    let mut regions: HashSet<String> = HashSet::new();
    
    println!("🚀 代理订阅管理器启动中...\n");
    
    // 获取所有订阅的代理节点
    for sub in subscriptions {
        if !sub.enabled {
            continue;
        }
        
        println!("📡 正在获取订阅: {}", sub.name);
        match fetch_subscription(sub).await {
            Ok(mut proxies) => {
                println!("   ✓ 获取到 {} 个节点", proxies.len());
                
                // 提取地区信息
                for proxy in &mut proxies {
                    if let Some(region) = extract_region(&proxy.name) {
                        proxy.region = Some(region.clone());
                        regions.insert(region);
                    }
                }
                
                all_proxies.extend(proxies);
            }
            Err(e) => {
                println!("   ✗ 获取失败: {}", e);
            }
        }
    }
    
    println!("\n📊 共获取 {} 个代理节点", all_proxies.len());
    println!("🌍 地区分组: {:?}", regions);
    
    // 创建代理组
    let all_proxy_names: Vec<String> = all_proxies.iter().map(|p| p.name.clone()).collect();
    
    let mut proxy_groups = basic_config.proxy_groups.clone();
    
    // 添加地区分组
    let region_groups = create_region_groups(&all_proxies);
    println!("🎯 创建了 {} 个地区分组", region_groups.len());
    
    // 添加服务专用组
    let service_groups = create_service_groups(&all_proxy_names, &all_proxies);
    println!("🎯 创建了 {} 个服务专用组", service_groups.len());
    
    proxy_groups.extend(region_groups);
    proxy_groups.extend(service_groups);
    
    Ok(ClashConfig {
        base_config: basic_config.base_config,
        proxies: all_proxies,
        proxy_groups,
        rules: basic_config.rules,
        rule_providers: basic_config.rule_providers,
    })
}
