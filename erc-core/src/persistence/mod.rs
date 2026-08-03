//! Persistence Layer — file I/O for .ercbudget project files.
//!
//! Projects are stored as UTF-8 JSON files with a `.ercbudget` extension.
//! All Decimal values are serialised as strings within the domain entities.
//!
//! Format versioning: the `format_version` field enables future migrations
//! without breaking existing files.
//!
//! `ProjectFile` also carries an optional `execution_data` block (format
//! v1.1), added in Milestone 1 Step 7 so the ERC Execution Application can
//! enrich a `.ercbudget` file without the Budget Application ever needing to
//! understand execution-specific types. It is opaque `serde_json::Value`
//! here — `erc-execution` deserialises it into its own typed
//! `ExecutionData` struct — which keeps `erc-core` free of any dependency on
//! execution-specific types. The Budget Application only ever writes
//! `execution_data: None`, and `skip_serializing_if` means the field is
//! simply absent from files it produces, so v1.0 files stay v1.0.

use crate::domain::entities::Project;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// The current .ercbudget file format version written by this crate's
/// `save_project`. Readers must also accept "1.1" (see `load_project`).
pub const CURRENT_FORMAT_VERSION: &str = "1.0";

/// The top-level wrapper written to disk.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectFile {
    pub format_version: String,
    pub created_at: String,
    pub updated_at: String,
    pub project: Project,
    /// Opaque to `erc-core` — typed as `Option<ExecutionData>` by
    /// `erc-execution`. `None`/absent for every file the Budget Application
    /// produces.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_data: Option<serde_json::Value>,
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
        execution_data: None,
    };

    let json = serde_json::to_string_pretty(&file)
        .map_err(|e| AppError::Persistence(format!("Failed to serialise project: {e}")))?;

    std::fs::write(path, json.as_bytes()).map_err(|e| {
        AppError::Persistence(format!("Failed to write file {}: {e}", path.display()))
    })?;

    Ok(())
}

/// Load a project from a file.
pub fn load_project(path: &Path) -> Result<Project, AppError> {
    let json = std::fs::read_to_string(path).map_err(|e| {
        AppError::Persistence(format!("Failed to read file {}: {e}", path.display()))
    })?;

    let file: ProjectFile = serde_json::from_str(&json).map_err(|e| {
        AppError::Persistence(format!(
            "Failed to parse project file (is this a valid .ercbudget file?): {e}"
        ))
    })?;

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
    let json = std::fs::read_to_string(path).map_err(|e| AppError::Persistence(e.to_string()))?;
    let value: serde_json::Value =
        serde_json::from_str(&json).map_err(|e| AppError::Persistence(e.to_string()))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{Project, ProjectConfig};
    use rust_decimal_macros::dec;

    fn make_project() -> Project {
        let config = ProjectConfig {
            project_title: "Persistence Test".to_string(),
            pi_name: "PI".to_string(),
            call_reference: "ERC-2025-CoG".to_string(),
            duration_years: 1,
            work_package_count: 1,
            work_package_names: vec![None],
            work_package_start_months: vec![1],
            work_package_end_months: vec![12],
            default_inflation_rate_pct: dec!(0),
            try_eur_rate: dec!(50),
            indirect_cost_rate_pct: dec!(25),
            rate_version_id: "from_2025_05_13".to_string(),
            call_opening_date: None,
        };
        Project::new(config)
    }

    fn temp_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "erc-core-persistence-test-{name}-{}.ercbudget",
            std::process::id()
        ))
    }

    #[test]
    fn test_v1_0_file_without_execution_data_loads_with_none() {
        // A real v1.0 file has no `execution_data` key at all. Confirms the
        // Budget App (which only knows format 1.0) can still open any file
        // it — or an older version of itself — has ever produced.
        let json = r#"{
            "format_version": "1.0",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z",
            "project": {
                "id": "11111111-1111-4111-8111-111111111111",
                "config": {
                    "project_title": "Legacy",
                    "pi_name": "PI",
                    "call_reference": "ERC-2025-CoG",
                    "duration_years": 1,
                    "work_package_count": 1,
                    "work_package_names": [null],
                    "work_package_start_months": [1],
                    "work_package_end_months": [12],
                    "default_inflation_rate_pct": "0",
                    "try_eur_rate": "50",
                    "indirect_cost_rate_pct": "25",
                    "rate_version_id": "from_2025_05_13",
                    "call_opening_date": null
                },
                "personnel_roles": [],
                "equipment_items": [],
                "trips": [],
                "other_cost_items": [],
                "subcontracting": { "amount_eur": "0", "work_package_id": 1 },
                "cfs_warning_dismissed": false
            }
        }"#;

        let file: ProjectFile = serde_json::from_str(json).expect("v1.0 file must parse");
        assert_eq!(file.format_version, "1.0");
        assert!(file.execution_data.is_none());
        assert_eq!(file.project.config.project_title, "Legacy");
    }

    #[test]
    fn test_v1_1_file_with_execution_data_loads_and_preserves_it() {
        let json = r#"{
            "format_version": "1.1",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-03-01T00:00:00Z",
            "project": {
                "id": "11111111-1111-4111-8111-111111111111",
                "config": {
                    "project_title": "With Execution Data",
                    "pi_name": "PI",
                    "call_reference": "ERC-2025-CoG",
                    "duration_years": 1,
                    "work_package_count": 1,
                    "work_package_names": [null],
                    "work_package_start_months": [1],
                    "work_package_end_months": [12],
                    "default_inflation_rate_pct": "0",
                    "try_eur_rate": "50",
                    "indirect_cost_rate_pct": "25",
                    "rate_version_id": "from_2025_05_13",
                    "call_opening_date": null
                },
                "personnel_roles": [],
                "equipment_items": [],
                "trips": [],
                "other_cost_items": [],
                "subcontracting": { "amount_eur": "0", "work_package_id": 1 },
                "cfs_warning_dismissed": false
            },
            "execution_data": { "schema_version": "1.0", "reporting_periods": [] }
        }"#;

        let file: ProjectFile = serde_json::from_str(json).expect("v1.1 file must parse");
        assert_eq!(file.format_version, "1.1");
        let exec = file.execution_data.expect("execution_data must be Some");
        assert_eq!(exec["schema_version"], "1.0");
    }

    #[test]
    fn test_save_project_never_writes_execution_data_field() {
        // The Budget Application must never write this field itself — only
        // erc-execution does. Guards against a future accidental regression
        // (e.g. a default derive) reintroducing it as `null` in every file.
        let project = make_project();
        let path = temp_path("no-exec-data-on-save");
        save_project(&project, &path).unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        std::fs::remove_file(&path).ok();
        assert!(
            !raw.contains("execution_data"),
            "save_project must not emit the execution_data field"
        );
    }

    #[test]
    fn test_save_and_load_roundtrip_preserves_project() {
        let project = make_project();
        let path = temp_path("roundtrip");
        save_project(&project, &path).unwrap();
        let reloaded = load_project(&path).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(project.id, reloaded.id);
        assert_eq!(project.config.project_title, reloaded.config.project_title);
    }

    #[test]
    fn test_save_project_preserves_created_at_on_resave() {
        let project = make_project();
        let path = temp_path("preserve-created-at");
        save_project(&project, &path).unwrap();
        let first_created_at = read_created_at(&path).unwrap();

        // Resave — created_at must not change, only updated_at.
        save_project(&project, &path).unwrap();
        let second_created_at = read_created_at(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(first_created_at, second_created_at);
    }

    #[test]
    fn test_auto_save_writes_autosave_sibling() {
        let project = make_project();
        let path = temp_path("autosave-base");
        let autosave_path = path.with_extension("ercbudget.autosave");
        std::fs::remove_file(&autosave_path).ok();

        auto_save(&project, Some(&path)).unwrap();
        assert!(autosave_path.exists());
        std::fs::remove_file(&autosave_path).ok();
    }
}
