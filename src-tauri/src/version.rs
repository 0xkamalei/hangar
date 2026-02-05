use crate::storage;
use crate::types::ConfigVersion;
use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;

/// Helper to determine the next version number
fn get_next_version_number() -> Result<usize> {
    let versions = list_versions()?;
    let max_ver = versions
        .iter()
        .filter_map(|v| {
            // Try to extract number from id "v{N}_..."
            let parts: Vec<&str> = v.id.split('_').collect();
            if parts.len() > 1 && parts[0].starts_with('v') {
                parts[0][1..].parse::<usize>().ok()
            } else {
                None
            }
        })
        .max()
        .unwrap_or(0);

    Ok(max_ver + 1)
}

/// Helper to resolve a version ID or alias (v1, v0) to a specific file or content
pub fn resolve_version_id(alias: &str) -> Result<String> {
    if alias == "v0" {
        return Ok("v0".to_string());
    }

    // If it looks like a short alias "v1", "v2" etc.
    if alias.starts_with('v') && alias[1..].chars().all(|c| c.is_digit(10)) {
        let target_ver = alias[1..].parse::<usize>().unwrap_or(0);
        let versions = list_versions()?;
        for v in versions {
            let parts: Vec<&str> = v.id.split('_').collect();
            if parts.len() > 1 && parts[0].starts_with('v') {
                if let Ok(ver) = parts[0][1..].parse::<usize>() {
                    if ver == target_ver {
                        return Ok(v.id);
                    }
                }
            }
        }
        return Err(anyhow::anyhow!("Version alias {} not found", alias));
    }

    // Otherwise assume it's a full ID
    Ok(alias.to_string())
}

/// Save current config as a new version
// filename can be used as the "type" indicator prefix
pub fn save_version(
    file_type: &str,
    description: &str,
    config_content: &str,
) -> Result<ConfigVersion> {
    let versions_dir = storage::get_versions_dir()?;
    let timestamp = Utc::now().timestamp();
    let version_num = get_next_version_number()?;

    // Sanitize description for filename
    let safe_desc: String = description
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .take(30)
        .collect();

    // Format: v{N}_{file_type}_{timestamp}_{description}.yaml
    // e.g. v1_basic_123456789_clean.yaml
    let filename = format!(
        "v{}_{}_{}_{}.yaml",
        version_num, file_type, timestamp, safe_desc
    );
    let file_path = versions_dir.join(&filename);

    fs::write(&file_path, config_content).context("Failed to save version file")?;

    let version = ConfigVersion {
        id: filename.clone(),
        timestamp,
        description: format!("[{}] {}", file_type, description),
        file_path: file_path.to_string_lossy().to_string(),
    };

    Ok(version)
}

/// List all saved versions
pub fn list_versions() -> Result<Vec<ConfigVersion>> {
    let versions_dir = storage::get_versions_dir()?;
    let mut versions = Vec::new();

    if let Ok(entries) = fs::read_dir(&versions_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .extension()
                .map_or(false, |ext| ext == "yaml" || ext == "yml")
            {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    // Parse filename: v{N}_{type}_{timestamp}_{desc}.yaml
                    if filename.starts_with("v") {
                        let parts: Vec<&str> = filename
                            .trim_end_matches(".yaml")
                            .trim_end_matches(".yml")
                            .split('_')
                            .collect();

                        // Try new format: v{N}, type, timestamp, desc...
                        if parts.len() >= 4 && parts[0][1..].chars().all(|c| c.is_digit(10)) {
                            if let Ok(timestamp) = parts[2].parse::<i64>() {
                                let ver_num = parts[0]; // v1
                                let file_type = parts[1];
                                let description_part = parts[3..].join("_");
                                versions.push(ConfigVersion {
                                    id: filename.to_string(),
                                    timestamp,
                                    description: format!(
                                        "{} [{}] {}",
                                        ver_num, file_type, description_part
                                    ),
                                    file_path: path.to_string_lossy().to_string(),
                                });
                                continue;
                            }
                        }

                        // Fallback Legacy format support: v_{type}_{timestamp}_{desc} OR v_{timestamp}_{desc}
                        // We treat legacy "v_" as just ID without version number
                        if parts[0] == "v" {
                            // v_basic_123...
                            if parts.len() >= 4 {
                                if let Ok(timestamp) = parts[2].parse::<i64>() {
                                    let file_type = parts[1];
                                    let description_part = parts[3..].join("_");
                                    versions.push(ConfigVersion {
                                        id: filename.to_string(),
                                        timestamp,
                                        description: format!(
                                            "[{}] {}",
                                            file_type, description_part
                                        ),
                                        file_path: path.to_string_lossy().to_string(),
                                    });
                                }
                            } else if parts.len() >= 3 {
                                // v_timestamp_desc
                                if let Ok(timestamp) = parts[1].parse::<i64>() {
                                    let description = parts[2..].join("_");
                                    versions.push(ConfigVersion {
                                        id: filename.to_string(),
                                        timestamp,
                                        description: format!("[legacy] {}", description),
                                        file_path: path.to_string_lossy().to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    // Sort by timestamp desc
    versions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(versions)
}

/// Get version content by id
pub fn get_version_content(id: &str) -> Result<String> {
    if id == "v0" {
        return Ok("".to_string());
    }

    let versions_dir = storage::get_versions_dir()?;
    let file_path = versions_dir.join(id);

    let content = fs::read_to_string(&file_path).context("Failed to read version file")?;

    Ok(content)
}

/// Rollback to a specific version
pub fn rollback_to_version(id: &str) -> Result<()> {
    // If id is v0, maybe clear config? For now assume valid file ID or alias resolved outside
    // But wait, this receives raw ID or alias?
    // Let's protect against raw alias if not resolved, but caller usually resolves or passes ID.
    // If we pass "v1" here, it won't work unless we resolve it.
    // Let's resolve it inside just in case.
    let resolved_id = resolve_version_id(id)?;

    if resolved_id == "v0" {
        return Err(anyhow::anyhow!("Cannot rollback to v0 (empty state) yet"));
    }

    let content = get_version_content(&resolved_id)?;
    let current_path = storage::get_current_config_path()?;

    // Save current as a backup version first
    if current_path.exists() {
        let current_content = fs::read_to_string(&current_path)?;
        save_version("backup", "auto_backup_before_rollback", &current_content)?;
    }

    // Write the rollback content
    fs::write(&current_path, content).context("Failed to write current config")?;

    Ok(())
}

/// Delete a version
pub fn delete_version(id: &str) -> Result<()> {
    let resolved_id = resolve_version_id(id)?;
    if resolved_id == "v0" {
        return Ok(()); // Nothing to delete
    }

    let versions_dir = storage::get_versions_dir()?;
    let file_path = versions_dir.join(resolved_id);

    fs::remove_file(&file_path).context("Failed to delete version file")?;

    Ok(())
}

/// Simple text diff between two strings
pub fn diff_configs(old: &str, new: &str) -> Vec<DiffLine> {
    let diff = similar::TextDiff::from_lines(old, new);
    let mut result = Vec::new();

    for op in diff.ops() {
        for change in diff.iter_changes(op) {
            let line_type = match change.tag() {
                similar::ChangeTag::Delete => "removed",
                similar::ChangeTag::Insert => "added",
                similar::ChangeTag::Equal => "unchanged",
            };
            result.push(DiffLine {
                line_type: line_type.to_string(),
                content: change.value().trim_end_matches('\n').to_string(),
            });
        }
    }

    result
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiffLine {
    pub line_type: String, // "added", "removed", "unchanged"
    pub content: String,
}
