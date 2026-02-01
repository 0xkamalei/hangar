use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use crate::storage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiPatchResult {
    pub description: String,
    pub operations: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    pub description: String,
    pub operations: Vec<PatchOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchOperation {
    pub op: String,
    pub path: String,
    pub value: Option<serde_json::Value>,
}

/// Build the system prompt for AI
fn build_system_prompt(proxy_groups: &[String], proxies_by_region: &std::collections::HashMap<String, Vec<String>>) -> String {
    let groups_str = proxy_groups.join(", ");
    let mut regions_str = String::new();
    for (region, nodes) in proxies_by_region {
        regions_str.push_str(&format!("\n  {}: {} 个节点", region, nodes.len()));
    }

    format!(r#"你是一个 Clash 配置专家。用户会描述他们对代理规则的需求，你需要生成 JSON Patch 格式的配置修改。

可用的 proxy-groups:
{}

可用的节点（按地区分组）:{}

规则格式参考:
- DOMAIN-SUFFIX,google.com,代理组名
- DOMAIN-KEYWORD,google,代理组名
- IP-CIDR,8.8.8.8/32,代理组名
- GEOIP,US,代理组名

输出格式要求（必须是有效的 JSON）:
{{
  "description": "简短描述这次修改做了什么",
  "operations": [
    {{"op": "add", "path": "/rules/0", "value": "DOMAIN-SUFFIX,google.com,Taiwan"}},
    {{"op": "replace", "path": "/proxy-groups/2/proxies/0", "value": "台湾节点1"}}
  ]
}}

注意：
1. 只输出 JSON，不要有其他文字
2. path 使用 JSON Pointer 格式
3. 添加规则时使用 /rules/0 表示添加到规则列表开头（优先级最高）"#, groups_str, regions_str)
}

/// Call LLM API to generate config patch
pub async fn generate_config_patch(prompt: &str) -> Result<AiPatchResult> {
    let config = storage::load_hangar_config()?;

    if config.llm.api_key.is_empty() {
        anyhow::bail!("请先在设置中配置 LLM API Key");
    }

    // TODO: Load actual proxy groups and nodes from current config
    let proxy_groups = vec!["节点选择".to_string(), "HK 地区".to_string(), "TW 地区".to_string(), "US 地区".to_string()];
    let mut proxies_by_region = std::collections::HashMap::new();
    proxies_by_region.insert("HK".to_string(), vec!["香港节点1".to_string(), "香港节点2".to_string()]);
    proxies_by_region.insert("TW".to_string(), vec!["台湾节点1".to_string(), "台湾节点2".to_string()]);
    proxies_by_region.insert("US".to_string(), vec!["美国节点1".to_string(), "美国节点2".to_string()]);

    let system_prompt = build_system_prompt(&proxy_groups, &proxies_by_region);

    let client = reqwest::Client::new();
    let request_body = serde_json::json!({
        "model": config.llm.model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.7,
        "max_tokens": 2000
    });

    let response = client
        .post(format!("{}/chat/completions", config.llm.base_url))
        .header("Authorization", format!("Bearer {}", config.llm.api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .context("Failed to call LLM API")?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("LLM API error: {}", error_text);
    }

    let response_json: serde_json::Value = response.json().await
        .context("Failed to parse LLM response")?;

    let content = response_json["choices"][0]["message"]["content"]
        .as_str()
        .context("Invalid response format")?;

    // Parse the AI response
    let ai_response: AiPatchResult = serde_json::from_str(content)
        .context("Failed to parse AI response as JSON")?;

    Ok(ai_response)
}

/// Apply JSON Patch to current config
pub fn apply_patch_to_config(current_config: &str, operations: &[serde_json::Value]) -> Result<String> {
    // Parse current config as JSON (convert from YAML first)
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(current_config)
        .context("Failed to parse current config as YAML")?;

    let mut json_value: serde_json::Value = serde_json::to_value(&yaml_value)
        .context("Failed to convert YAML to JSON")?;

    // Apply each operation
    for op in operations {
        let patch = json_patch::Patch(vec![serde_json::from_value(op.clone())
            .context("Invalid patch operation")?]);
        json_patch::patch(&mut json_value, &patch)
            .context("Failed to apply patch")?;
    }

    // Convert back to YAML
    let result = serde_yaml::to_string(&json_value)
        .context("Failed to convert back to YAML")?;

    Ok(result)
}

/// Test LLM connection
pub async fn test_llm_connection(base_url: &str, api_key: &str, model: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let request_body = serde_json::json!({
        "model": model,
        "messages": [
            {"role": "user", "content": "Say 'OK' if you can read this."}
        ],
        "max_tokens": 10
    });

    let response = client
        .post(format!("{}/chat/completions", base_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .context("Failed to connect to LLM API")?;

    if response.status().is_success() {
        Ok("连接成功！".to_string())
    } else {
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("连接失败: {}", error_text)
    }
}
