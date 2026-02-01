use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use crate::storage;
use crate::types::ConfigVersion;

/// Save current config as a new version
pub fn save_version(description: &str, config_content: &str) -> Result<ConfigVersion> {
    let versions_dir = storage::get_versions_dir()?;
    let timestamp = Utc::now().timestamp();

    // Sanitize description for filename
    let safe_desc: String = description
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .take(30)
        .collect();

    let filename = format!("v_{}_{}.yaml", timestamp, safe_desc);
    let file_path = versions_dir.join(&filename);

    fs::write(&file_path, config_content)
        .context("Failed to save version file")?;

    let version = ConfigVersion {
        id: filename.clone(),
        timestamp,
        description: description.to_string(),
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
            if path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    // Parse filename: v_{timestamp}_{description}.yaml
                    if filename.starts_with("v_") {
                        let parts: Vec<&str> = filename.trim_end_matches(".yaml").trim_end_matches(".yml").splitn(3, '_').collect();
                        if parts.len() >= 2 {
                            if let Ok(timestamp) = parts[1].parse::<i64>() {
                                let description = parts.get(2).unwrap_or(&"").to_string();
                                versions.push(ConfigVersion {
                                    id: filename.to_string(),
                                    timestamp,
                                    description,
                                    file_path: path.to_string_lossy().to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort by timestamp descending (newest first)
    versions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(versions)
}

/// Get version content by id
pub fn get_version_content(id: &str) -> Result<String> {
    let versions_dir = storage::get_versions_dir()?;
    let file_path = versions_dir.join(id);

    let content = fs::read_to_string(&file_path)
        .context("Failed to read version file")?;

    Ok(content)
}

/// Rollback to a specific version
pub fn rollback_to_version(id: &str) -> Result<()> {
    let content = get_version_content(id)?;
    let current_path = storage::get_current_config_path()?;

    // Save current as a backup version first
    if current_path.exists() {
        let current_content = fs::read_to_string(&current_path)?;
        save_version("auto_backup_before_rollback", &current_content)?;
    }

    // Write the rollback content
    fs::write(&current_path, content)
        .context("Failed to write current config")?;

    Ok(())
}

/// Delete a version
pub fn delete_version(id: &str) -> Result<()> {
    let versions_dir = storage::get_versions_dir()?;
    let file_path = versions_dir.join(id);

    fs::remove_file(&file_path)
        .context("Failed to delete version file")?;

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
                result.push(DiffLine { line_type: "unchanged".to_string(), content: o.to_string() });
            }
            (Some(o), Some(n)) => {
                result.push(DiffLine { line_type: "removed".to_string(), content: o.to_string() });
                result.push(DiffLine { line_type: "added".to_string(), content: n.to_string() });
            }
            (Some(o), None) => {
                result.push(DiffLine { line_type: "removed".to_string(), content: o.to_string() });
            }
            (None, Some(n)) => {
                result.push(DiffLine { line_type: "added".to_string(), content: n.to_string() });
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
