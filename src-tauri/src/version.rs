use crate::storage;
use crate::types::ConfigVersion;
use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;

/// Save current config as a new version
// filename can be used as the "type" indicator prefix
pub fn save_version(
    file_type: &str,
    description: &str,
    config_content: &str,
) -> Result<ConfigVersion> {
    let versions_dir = storage::get_versions_dir()?;
    let timestamp = Utc::now().timestamp();

    // Sanitize description for filename
    let safe_desc: String = description
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .take(30)
        .collect();

    // Format: v_{file_type}_{timestamp}_{description}.yaml
    // e.g. v_basic_123456789_clean.yaml
    let filename = format!("v_{}_{}_{}.yaml", file_type, timestamp, safe_desc);
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
                    // Parse filename: v_{opt_type}_{timestamp}_{description}.yaml
                    if filename.starts_with("v_") {
                        let parts: Vec<&str> = filename
                            .trim_end_matches(".yaml")
                            .trim_end_matches(".yml")
                            .split('_')
                            .collect();
                        // Minimum parts: v, type, timestamp, desc...
                        if parts.len() >= 4 {
                            // Try to find timestamp index. Usually index 2.
                            if let Ok(timestamp) = parts[2].parse::<i64>() {
                                let file_type = parts[1];
                                let description_part = parts[3..].join("_");
                                versions.push(ConfigVersion {
                                    id: filename.to_string(),
                                    timestamp,
                                    description: format!("[{}] {}", file_type, description_part),
                                    file_path: path.to_string_lossy().to_string(),
                                });
                            }
                        } else if parts.len() >= 3 {
                            // Legacy format support? v_timestamp_desc
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
    // ... sort
    versions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(versions)
}

/// Get version content by id
pub fn get_version_content(id: &str) -> Result<String> {
    let versions_dir = storage::get_versions_dir()?;
    let file_path = versions_dir.join(id);

    let content = fs::read_to_string(&file_path).context("Failed to read version file")?;

    Ok(content)
}

/// Rollback to a specific version
pub fn rollback_to_version(id: &str) -> Result<()> {
    let content = get_version_content(id)?;
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
    let versions_dir = storage::get_versions_dir()?;
    let file_path = versions_dir.join(id);

    fs::remove_file(&file_path).context("Failed to delete version file")?;

    Ok(())
}

/// Simple text diff between two strings
pub fn diff_configs(old: &str, new: &str) -> Vec<DiffLine> {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    let mut result = Vec::new();
    let max_len = old_lines.len().max(new_lines.len());

    for i in 0..max_len {
        let old_line = old_lines.get(i).copied();
        let new_line = new_lines.get(i).copied();

        match (old_line, new_line) {
            (Some(o), Some(n)) if o == n => {
                result.push(DiffLine {
                    line_type: "unchanged".to_string(),
                    content: o.to_string(),
                });
            }
            (Some(o), Some(n)) => {
                result.push(DiffLine {
                    line_type: "removed".to_string(),
                    content: o.to_string(),
                });
                result.push(DiffLine {
                    line_type: "added".to_string(),
                    content: n.to_string(),
                });
            }
            (Some(o), None) => {
                result.push(DiffLine {
                    line_type: "removed".to_string(),
                    content: o.to_string(),
                });
            }
            (None, Some(n)) => {
                result.push(DiffLine {
                    line_type: "added".to_string(),
                    content: n.to_string(),
                });
            }
            (None, None) => {}
        }
    }

    result
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiffLine {
    pub line_type: String,
    pub content: String,
}
