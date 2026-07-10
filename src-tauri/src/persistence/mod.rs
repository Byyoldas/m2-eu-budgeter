//! Persistence Layer — file I/O for .ercbudget project files.
//!
//! Projects are stored as UTF-8 JSON files with a `.ercbudget` extension.
//! All Decimal values are serialised as strings within the domain entities.
//!
//! Format versioning: the `format_version` field enables future migrations
//! without breaking existing files.

use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::domain::entities::Project;
use crate::error::AppError;

/// The current .ercbudget file format version.
pub const CURRENT_FORMAT_VERSION: &str = "1.0";

/// The top-level wrapper written to disk.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectFile {
    pub format_version: String,
    pub created_at: String,
    pub updated_at: String,
    pub project: Project,
}

/// Save a project to a file.
pub fn save_project(project: &Project, path: &Path) -> Result<(), AppError> {
    let now = chrono::Utc::now().to_rfc3339();

    // Read existing created_at if the file already exists.
    let created_at = if path.exists() {
        read_created_at(path).unwrap_or_else(|_| now.clone())
    } else {
        now.clone()
    };

    let file = ProjectFile {
        format_version: CURRENT_FORMAT_VERSION.to_string(),
        created_at,
        updated_at: now,
        project: project.clone(),
    };

    let json = serde_json::to_string_pretty(&file)
        .map_err(|e| AppError::Persistence(format!("Failed to serialise project: {e}")))?;

    std::fs::write(path, json.as_bytes())
        .map_err(|e| AppError::Persistence(format!("Failed to write file {}: {e}", path.display())))?;

    Ok(())
}

/// Load a project from a file.
pub fn load_project(path: &Path) -> Result<Project, AppError> {
    let json = std::fs::read_to_string(path)
        .map_err(|e| AppError::Persistence(format!("Failed to read file {}: {e}", path.display())))?;

    let file: ProjectFile = serde_json::from_str(&json)
        .map_err(|e| AppError::Persistence(format!(
            "Failed to parse project file (is this a valid .ercbudget file?): {e}"
        )))?;

    // Future: migrate format versions here if file.format_version != CURRENT_FORMAT_VERSION.

    Ok(file.project)
}

/// Auto-save to a temporary file (called after every mutation).
/// The temp path is a sibling of the project file with `.autosave` extension,
/// or falls back to the system temp directory.
pub fn auto_save(project: &Project, project_path: Option<&Path>) -> Result<(), AppError> {
    let auto_path = match project_path {
        Some(p) => p.with_extension("ercbudget.autosave"),
        None => {
            let mut temp = std::env::temp_dir();
            temp.push(format!("erc-budget-autosave-{}.ercbudget", project.id));
            temp
        }
    };
    save_project(project, &auto_path)
}

/// Read the created_at timestamp from an existing file without full deserialisation.
fn read_created_at(path: &Path) -> Result<String, AppError> {
    let json = std::fs::read_to_string(path)
        .map_err(|e| AppError::Persistence(e.to_string()))?;
    let value: serde_json::Value = serde_json::from_str(&json)
        .map_err(|e| AppError::Persistence(e.to_string()))?;
    let ts = value
        .get("created_at")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if ts.is_empty() {
        Err(AppError::Persistence("No created_at in file".to_string()))
    } else {
        Ok(ts)
    }
}
