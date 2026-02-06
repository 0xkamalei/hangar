use crate::subscription::extract_region;
use crate::types::{ClashConfig, ProxyGroup, ProxyNode, Subscription};
use anyhow::{Context, Result};
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
                name: format!("{}-地区", region),
                group_type: "fallback".to_string(), // 修改为 fallback
                proxies: proxy_names,
                extra: {
                    let mut map = HashMap::new();
                    map.insert("url".to_string(), serde_json::json!("http://www.gstatic.com/generate_204"));
                    map.insert("interval".to_string(), serde_json::json!(3600));
                    map
                },
            });
        }
    }

    groups
}

/// Parse a subscription's raw YAML (from cache) into ProxyNodes
pub fn parse_cached_subscription(sub: &Subscription) -> Result<Vec<ProxyNode>> {
    let cache_path = crate::storage::get_subscription_cache_path(&sub.id)?;
    if !cache_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(cache_path)?;
    // YAML parsing
    let config: serde_yaml::Value = serde_yaml::from_str(&content)?;

    let mut proxies = Vec::new();
    if let Some(serde_yaml::Value::Sequence(proxy_seq)) = config.get("proxies") {
        for proxy_val in proxy_seq {
            // Convert YAML value to JSON value to match ProxyNode serde
            let json_val = serde_json::to_value(proxy_val)?;
            if let Ok(mut node) = serde_json::from_value::<ProxyNode>(json_val) {
                // Add airport name prefix
                node.name = format!("[{}] {}", sub.name, node.name);
                node.airport = sub.name.clone();
                proxies.push(node);
            }
        }
    }
    Ok(proxies)
}

/// 合并配置
pub async fn merge_configs(
    subscriptions: &[Subscription],
    // basic_config is now loaded inside or passed in, plan said "Load basic.yml from src-tauri/resources/"
) -> Result<ClashConfig> {
    // 1. Load basic.yml
    let basic_path = crate::storage::get_basic_config_path()?;
    let basic_content = std::fs::read_to_string(&basic_path).context("Failed to read basic.yml")?;
    let basic_config: ClashConfig =
        serde_yaml::from_str(&basic_content).context("Failed to parse basic.yml")?;

    // 2. Load groups.yml (if exists)
    let groups_path = crate::storage::get_groups_config_path()?;
    let mut extra_groups = Vec::new();
    if groups_path.exists() {
        let groups_content = std::fs::read_to_string(&groups_path)?;
        // Assuming groups.yml structure matches what we expect (e.g., has a proxy-groups list)
        // For now, let's parse as Value and extract proxy-groups
        // groups.yml is a list of ProxyGroup
        let groups: Vec<ProxyGroup> = serde_yaml::from_str(&groups_content)?;
        extra_groups.extend(groups);
    }

    let mut all_proxies = basic_config.proxies.clone();
    let mut regions: HashSet<String> = HashSet::new();

    println!("🚀 Merging configuration from local cache...\n");

    // 3. Process Subscriptions (from cache)
    for sub in subscriptions {
        if !sub.enabled {
            continue;
        }

        match parse_cached_subscription(sub) {
            Ok(mut proxies) => {
                println!("   ✓ Loaded {} proxies from {}", proxies.len(), sub.name);
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
                println!("   ⚠️ Failed to load cache for {}: {}", sub.name, e);
            }
        }
    }

    println!("\n📊 Total proxies: {}", all_proxies.len());

    // 4. Create Groups
    // Ignore basic.yml proxy groups, start fresh
    let mut proxy_groups: Vec<ProxyGroup> = Vec::new();

    // Auto-generated region groups
    let region_groups = create_region_groups(&all_proxies);
    
    // 定义 AI 支持地区的优先级顺序
    let ai_priority = vec![
        "US", "SG", "JP", "TW", "KR", "DE", "UK", "CA", "IN", "FR", "AU", "BR"
    ];

    // 按照优先级顺序提取 AI 支持的分组名
    let mut ai_supported_region_group_names: Vec<String> = Vec::new();
    for &code in &ai_priority {
        let target_name = format!("{}-地区", code);
        if region_groups.iter().any(|g| g.name == target_name) {
            ai_supported_region_group_names.push(target_name);
        }
    }

    // 兜底：如果还有其他在 is_ai_supported_region 列表里但在 ai_priority 没排名的，也加进去
    for g in &region_groups {
        let region_code = g.name.split("-").next().unwrap_or("");
        if crate::subscription::is_ai_supported_region(region_code) 
           && !ai_supported_region_group_names.contains(&g.name) {
            ai_supported_region_group_names.push(g.name.clone());
        }
    }

    // Modify extra_groups (from groups.yml) to include auto-generated region groups in their proxies
    for group in &mut extra_groups {
        if group.name == "自动选择" || group.name == "AI-专用" {
            group.proxies.extend(ai_supported_region_group_names.clone());
        } else {
            // 其他分组可以包含所有地区
            let all_region_group_names: Vec<String> = region_groups.iter().map(|g| g.name.clone()).collect();
            group.proxies.extend(all_region_group_names);
        }
    }

    // Add modified extra_groups to final list
    proxy_groups.extend(extra_groups);

    // Add region_groups to final list
    proxy_groups.extend(region_groups);

    Ok(ClashConfig {
        base_config: basic_config.base_config,
        proxies: all_proxies,
        proxy_groups,
        rules: basic_config.rules,
        rule_providers: basic_config.rule_providers,
    })
}
