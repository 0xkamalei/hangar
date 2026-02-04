use crate::{storage, types};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiPatchResult {
    pub description: String,
    pub target: String, // "basic", "groups", "current"
    pub operations: Vec<serde_json::Value>,
}

/// Build the system prompt for AI
fn build_system_prompt(
    proxy_groups: &[String],
    proxies_by_region: &HashMap<String, Vec<String>>,
) -> String {
    let groups_str = proxy_groups.join(", ");
    let mut regions_str = String::new();

    let mut sorted_regions: Vec<_> = proxies_by_region.keys().collect();
    sorted_regions.sort();

    for region in sorted_regions {
        if let Some(nodes) = proxies_by_region.get(region) {
            regions_str.push_str(&format!("\n  {}: {} 个节点", region, nodes.len()));
        }
    }

    format!(
        r#"你是一个 Clash 配置专家。用户会描述他们对代理规则的需求，你需要判断应该修改哪个配置文件，并生成 JSON Patch 格式的修改。

**配置文件说明**:
- `basic.yml`: 基础配置，包含通用代理组、规则。修改此文件会影响未来的所有配置生成。
- `groups.yml`: 额外的代理组定义。
- `current.yaml`: 当前正在运行的最终配置（临时修改）。

**可用的 proxy-groups**:
{}

**可用的节点（按地区分组）**:{}

**输出格式要求（必须是有效的 JSON）**:
{{
  "description": "简短描述这次修改做了什么",
  "target": "basic", // 或 "groups", "current"
  "operations": [
    {{"op": "add", "path": "/rules/0", "value": "DOMAIN-SUFFIX,google.com,Taiwan"}}
  ]
}}

**注意**:
1. 如果是修改规则(rules)，通常建议修改 `basic.yml`。
2. 如果是添加新的代理组，通常修改 `groups.yml`。
3. 如果是临时调整某个组的节点选择，或者用户明确要求立即生效不影响基础配置，修改 `current.yaml`。
4. path 使用 JSON Pointer 格式。
"#,
        groups_str, regions_str
    )
}

/// Call LLM API to generate config patch
pub async fn generate_config_patch(prompt: &str) -> Result<AiPatchResult> {
    let config = storage::load_hangar_config()?;

    if config.llm.api_key.is_empty() {
        anyhow::bail!("请先在设置中配置 LLM API Key");
    }

    // Load actual proxy groups and nodes from current config to give contest
    let current_config_path = storage::get_current_config_path()?;
    let mut proxy_groups = Vec::new();
    let mut proxies_by_region: HashMap<String, Vec<String>> = HashMap::new();

    if current_config_path.exists() {
        let content =
            fs::read_to_string(&current_config_path).context("Failed to read current.yaml")?;
        // permissive parsing
        if let Ok(clash_config) = serde_yaml::from_str::<types::ClashConfig>(&content) {
            proxy_groups = clash_config
                .proxy_groups
                .iter()
                .map(|g| g.name.clone())
                .collect();
            for proxy in clash_config.proxies {
                let region = proxy.region.unwrap_or_else(|| "Unknown".to_string());
                proxies_by_region
                    .entry(region)
                    .or_default()
                    .push(proxy.name);
            }
        }
    }

    if proxy_groups.is_empty() {
        proxy_groups = vec!["节点选择".to_string()];
    }

    let system_prompt = build_system_prompt(&proxy_groups, &proxies_by_region);

    let client = reqwest::Client::new();
    let request_body = serde_json::json!({
        "model": config.llm.model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.2, // Lower temp for more deterministic code gen
        "max_tokens": 2000
    });

    // ... (rest of the request logic)
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

    let response_json: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse LLM response")?;

    let content = response_json["choices"][0]["message"]["content"]
        .as_str()
        .context("Invalid response format")?;

    // Clean markdown code blocks if any
    let cleaned_content = content
        .replace("```json", "")
        .replace("```", "")
        .trim()
        .to_string();

    // Parse the AI response
    let ai_response: AiPatchResult =
        serde_json::from_str(&cleaned_content).context("Failed to parse AI response as JSON")?;

    Ok(ai_response)
}

/// Apply JSON Patch to any YAML config content
pub fn apply_patch_to_config(
    config_content: &str,
    operations: &[serde_json::Value],
) -> Result<String> {
    // Parse current config as JSON (convert from YAML first)
    let yaml_value: serde_yaml::Value =
        serde_yaml::from_str(config_content).context("Failed to parse current config as YAML")?;

    let mut json_value: serde_json::Value =
        serde_json::to_value(&yaml_value).context("Failed to convert YAML to JSON")?;

    // Apply each operation
    for op in operations {
        let patch = json_patch::Patch(vec![
            serde_json::from_value(op.clone()).context("Invalid patch operation")?
        ]);
        json_patch::patch(&mut json_value, &patch).context("Failed to apply patch")?;
    }

    // Convert back to YAML
    let result = serde_yaml::to_string(&json_value).context("Failed to convert back to YAML")?;

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
