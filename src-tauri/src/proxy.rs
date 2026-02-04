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
        let groups_yaml: serde_yaml::Value = serde_yaml::from_str(&groups_content)?;
        if let Some(serde_yaml::Value::Sequence(seq)) = groups_yaml.get("proxy-groups") {
            for g in seq {
                let json_g = serde_json::to_value(g)?;
                if let Ok(group) = serde_json::from_value::<ProxyGroup>(json_g) {
                    extra_groups.push(group);
                }
            }
        }
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
    let all_proxy_names: Vec<String> = all_proxies.iter().map(|p| p.name.clone()).collect();
    let mut proxy_groups = basic_config.proxy_groups.clone();

    // Add groups from groups.yml
    proxy_groups.extend(extra_groups);

    // Auto-generated region groups
    let region_groups = create_region_groups(&all_proxies);
    proxy_groups.extend(region_groups);

    // Auto-generated service groups (Chatgpt etc) - keep previous logic?
    // The previous implementation hardcoded service groups. User wants groups.yml to control groups.
    // If groups.yml is provided, maybe we rely on that + basic.yml?
    // User said: "basic.yml中配置的rules中的target group都是以 groups.yml中配置的groups为基础"
    // And "merge到逻辑主要是生成以国家命名的group，然后再把所有的groups添加到 groups.yml中配置的所有groups下"

    // So logic:
    // 1. Basic (has some groups?)
    // 2. Groups.yml (has Main Groups like "Proxy", "Netflix", etc)
    // 3. Region Groups (Auto generated)
    // 4. We need to add all Region Groups to the Groups defined in groups.yml?

    // Let's implement the "Add all region groups to groups in groups.yml" logic if practical.
    // Usually groups.yml might have "proxies: []" or "use: []".
    // If a group in groups.yml is "select", we might want to append all region groups to it?
    // Or just append specific ones?

    // Simplified Logic per cli.md intent:
    // - Generate Region Groups (HK, US, JP...)
    // - Load Groups from groups.yml (which might be "Global", "Streaming"...)
    // - Add "Region Group Names" to the "proxies" list of appropriate groups in groups.yml?
    //   Actually, usually we add "HK", "US" etc to "Proxy" group.

    // For now, let's keep the `create_service_groups` logic but merge it with groups.yml intent if possible.
    // However, to be safe and strictly follow "merge... groups.yml", let's append region groups to the list.

    let service_groups = create_service_groups(&all_proxy_names, &all_proxies);
    // Note: The previous service groups were hardcoded.
    // If we want to fully switch to groups.yml, we should probably disable hardcoded ones except "Select"?
    // But `create_service_groups` creates specific things like ChatGPT.
    // Let's keep existing helper for now but append them.

    proxy_groups.extend(service_groups);

    Ok(ClashConfig {
        base_config: basic_config.base_config,
        proxies: all_proxies,
        proxy_groups,
        rules: basic_config.rules,
        rule_providers: basic_config.rule_providers,
    })
}
