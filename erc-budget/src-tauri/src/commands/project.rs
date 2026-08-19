//! IPC commands for project lifecycle management.
//!
//! Every command acquires the project mutex, performs its operation,
//! and releases the lock before returning. Commands never hold the lock
//! across an await point.

use crate::calculation::calculate_budget_summary;
use crate::domain::dto::{BudgetSummaryDto, ProjectConfigDto};
use crate::domain::entities::{Project, ProjectConfig};
use crate::domain::rate_data::{CountrySummary, RateVersionSummary};
use crate::error::AppError;
use crate::persistence;
use crate::validation::validate_project_config;
use crate::AppState;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, State};

/// Turn a project title into a filesystem-safe base filename (no extension).
/// Strips characters illegal on Windows and falls back to a generic name
/// if the title sanitizes down to nothing.
fn sanitize_filename(title: &str) -> String {
    let cleaned: String = title
        .chars()
        .map(|c| if r#"<>:"/\|?*"#.contains(c) { '_' } else { c })
        .collect();
    let trimmed = cleaned.trim().trim_end_matches('.').trim();
    if trimmed.is_empty() {
        "Untitled Project".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Find a non-colliding `.ercbudget` path in `dir` for `base_name`,
/// appending " (2)", " (3)", etc. the way Finder/Explorer do.
fn unique_project_path(dir: &Path, base_name: &str) -> PathBuf {
    let mut candidate = dir.join(format!("{base_name}.ercbudget"));
    let mut n = 2;
    while candidate.exists() {
        candidate = dir.join(format!("{base_name} ({n}).ercbudget"));
        n += 1;
    }
    candidate
}

/// Create a new empty project from configuration.
///
/// Immediately assigns and writes a default file path on the user's Desktop
/// (named after the project title, de-duplicated like Finder/Explorer would)
/// so the project is a real, saved file from the moment it's created — the
/// Save button and auto-save both need a known path to target.
///
/// Returns the initial (zero) BudgetSummaryDto so the right panel renders immediately.
#[tauri::command]
pub fn create_project(
    app: AppHandle,
    state: State<'_, AppState>,
    config: ProjectConfigDto,
) -> Result<BudgetSummaryDto, AppError> {
    validate_project_config(&config)?;

    let project_config = ProjectConfig {
        project_title: config.project_title,
        pi_name: config.pi_name,
        call_reference: config.call_reference,
        duration_years: config.duration_years,
        work_package_count: config.work_package_count,
        work_package_names: config.work_package_names,
        work_package_start_months: config.work_package_start_months,
        work_package_end_months: config.work_package_end_months,
        default_inflation_rate_pct: config.default_inflation_rate_pct,
        try_eur_rate: config.try_eur_rate,
        indirect_cost_rate_pct: config.indirect_cost_rate_pct,
        rate_version_id: config.rate_version_id,
        call_opening_date: config.call_opening_date,
    };

    let project = Project::new(project_config);
    let summary = calculate_budget_summary(&project, &state.rate_data)?;

    let desktop = app
        .path()
        .desktop_dir()
        .map_err(|e| AppError::Persistence(format!("Could not locate the Desktop folder: {e}")))?;
    let base_name = sanitize_filename(&project.config.project_title);
    let path = unique_project_path(&desktop, &base_name);
    persistence::save_project(&project, &path)?;

    let mut lock = state.project.lock().unwrap();
    *lock = Some(project);
    drop(lock);

    let mut path_lock = state.project_path.lock().unwrap();
    *path_lock = Some(path);

    Ok(summary)
}

/// Get the current project's file path, if any (e.g. right after `create_project`
/// assigned a default Desktop path, so the frontend can mirror it).
#[tauri::command]
pub fn get_project_path(state: State<'_, AppState>) -> Option<String> {
    let lock = state.project_path.lock().unwrap();
    lock.as_ref().map(|p| p.display().to_string())
}

/// Update the project configuration (e.g. exchange rate, inflation rate, indirect rate).
/// Triggers full recalculation as changing rates affects all personnel costs.
#[tauri::command]
pub fn update_project_config(
    state: State<'_, AppState>,
    config: ProjectConfigDto,
) -> Result<BudgetSummaryDto, AppError> {
    validate_project_config(&config)?;

    let mut lock = state.project.lock().unwrap();
    let project = lock.as_mut().ok_or(AppError::NoProject)?;

    project.config.project_title = config.project_title;
    project.config.pi_name = config.pi_name;
    project.config.call_reference = config.call_reference;
    project.config.duration_years = config.duration_years;
    project.config.work_package_count = config.work_package_count;
    project.config.work_package_names = config.work_package_names;
    project.config.work_package_start_months = config.work_package_start_months;
    project.config.work_package_end_months = config.work_package_end_months;
    project.config.default_inflation_rate_pct = config.default_inflation_rate_pct;
    project.config.try_eur_rate = config.try_eur_rate;
    project.config.indirect_cost_rate_pct = config.indirect_cost_rate_pct;
    project.config.rate_version_id = config.rate_version_id;
    project.config.call_opening_date = config.call_opening_date;

    let summary = calculate_budget_summary(project, &state.rate_data)?;
    Ok(summary)
}

/// Load a project from a .ercbudget file.
#[tauri::command]
pub fn load_project(
    state: State<'_, AppState>,
    path: String,
) -> Result<BudgetSummaryDto, AppError> {
    let file_path = std::path::PathBuf::from(&path);
    let project = persistence::load_project(&file_path)?;
    let summary = calculate_budget_summary(&project, &state.rate_data)?;

    let mut project_lock = state.project.lock().unwrap();
    *project_lock = Some(project);

    let mut path_lock = state.project_path.lock().unwrap();
    *path_lock = Some(file_path);

    Ok(summary)
}

/// Save the current project to its known file path.
/// If no path has been set, returns an error (caller should use save_as instead).
#[tauri::command]
pub fn save_project(state: State<'_, AppState>) -> Result<(), AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let path_lock = state.project_path.lock().unwrap();
    let path = path_lock.as_ref().ok_or_else(|| {
        AppError::Persistence("No file path set. Use 'Save As' to choose a location.".to_string())
    })?;

    persistence::save_project(project, path)
}

/// Save the project to a new path (Save As).
#[tauri::command]
pub fn save_project_as(state: State<'_, AppState>, path: String) -> Result<(), AppError> {
    let file_path = std::path::PathBuf::from(&path);

    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    persistence::save_project(project, &file_path)?;
    drop(project_lock);

    let mut path_lock = state.project_path.lock().unwrap();
    *path_lock = Some(file_path);

    Ok(())
}

/// Get the current project's BudgetSummaryDto (e.g. on app startup after load).
#[tauri::command]
pub fn get_project(state: State<'_, AppState>) -> Result<BudgetSummaryDto, AppError> {
    let lock = state.project.lock().unwrap();
    let project = lock.as_ref().ok_or(AppError::NoProject)?;
    calculate_budget_summary(project, &state.rate_data)
}

/// Get the current project's configuration.
///
/// `load_project` only returns a BudgetSummaryDto (the calculated numbers) —
/// the config that produced them (title, PI, WP names/timeline, rates) lives
/// only in backend state until this is called. The frontend needs it to
/// re-populate the Project Setup / Budget Settings / Work Packages forms,
/// and to drive per-WP formatting in the Excel/CSV exporters, after opening
/// an existing file.
#[tauri::command]
pub fn get_project_config(state: State<'_, AppState>) -> Result<ProjectConfigDto, AppError> {
    let lock = state.project.lock().unwrap();
    let project = lock.as_ref().ok_or(AppError::NoProject)?;
    Ok(project_config_to_dto(&project.config))
}

fn project_config_to_dto(config: &ProjectConfig) -> ProjectConfigDto {
    ProjectConfigDto {
        project_title: config.project_title.clone(),
        pi_name: config.pi_name.clone(),
        call_reference: config.call_reference.clone(),
        duration_years: config.duration_years,
        work_package_count: config.work_package_count,
        work_package_names: config.work_package_names.clone(),
        work_package_start_months: config.work_package_start_months.clone(),
        work_package_end_months: config.work_package_end_months.clone(),
        default_inflation_rate_pct: config.default_inflation_rate_pct,
        try_eur_rate: config.try_eur_rate,
        indirect_cost_rate_pct: config.indirect_cost_rate_pct,
        rate_version_id: config.rate_version_id.clone(),
        call_opening_date: config.call_opening_date.clone(),
    }
}

/// Return all available EU rate version descriptors for the UI dropdown.
#[tauri::command]
pub fn get_rate_versions(state: State<'_, AppState>) -> Result<Vec<RateVersionSummary>, AppError> {
    Ok(state.rate_data.version_summaries())
}

/// Return the country list for a given rate version (for the travel form dropdown).
#[tauri::command(rename_all = "snake_case")]
pub fn get_countries(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<Vec<CountrySummary>, AppError> {
    let version = state
        .rate_data
        .find_version(&version_id)
        .ok_or_else(|| AppError::NotFound(format!("Rate version '{version_id}' not found.")))?;
    Ok(version.sorted_countries())
}

#[cfg(test)]
mod default_path_tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "erc-budget-default-path-test-{name}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn sanitize_filename_strips_illegal_characters() {
        assert_eq!(
            sanitize_filename(r#"A/B\C:D"E<F>G|H?I*J"#),
            "A_B_C_D_E_F_G_H_I_J"
        );
    }

    #[test]
    fn sanitize_filename_trims_whitespace_and_trailing_dots() {
        assert_eq!(sanitize_filename("  My Project.  "), "My Project");
    }

    #[test]
    fn sanitize_filename_falls_back_when_empty() {
        assert_eq!(sanitize_filename(""), "Untitled Project");
        assert_eq!(sanitize_filename("   "), "Untitled Project");
    }

    #[test]
    fn sanitize_filename_keeps_replaced_characters_when_not_blank() {
        // Illegal characters are replaced, not stripped to nothing — a title
        // of all-illegal characters still yields a valid, if ugly, filename
        // rather than silently falling back to a generic name.
        assert_eq!(sanitize_filename("///"), "___");
    }

    #[test]
    fn sanitize_filename_leaves_normal_titles_untouched() {
        assert_eq!(sanitize_filename("ERC CoG 2026"), "ERC CoG 2026");
    }

    #[test]
    fn unique_project_path_uses_plain_name_when_free() {
        let dir = temp_dir("free");
        let path = unique_project_path(&dir, "My Project");
        assert_eq!(path, dir.join("My Project.ercbudget"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn unique_project_path_appends_counter_on_collision() {
        let dir = temp_dir("collision");
        std::fs::write(dir.join("My Project.ercbudget"), "x").unwrap();
        std::fs::write(dir.join("My Project (2).ercbudget"), "x").unwrap();

        let path = unique_project_path(&dir, "My Project");
        assert_eq!(path, dir.join("My Project (3).ercbudget"));
        std::fs::remove_dir_all(&dir).ok();
    }

    // Guards the exact bug this was written to fix: after re-opening a saved
    // project, get_project_config must hand back every field the Project
    // Setup / Budget Settings / Work Packages forms (and the Excel/CSV
    // exporters) need — not a partial or default-filled config.
    #[test]
    fn project_config_to_dto_carries_every_field() {
        let config = ProjectConfig {
            project_title: "ERC CoG 2026".to_string(),
            pi_name: "Dr. Test".to_string(),
            call_reference: "ERC-2026-CoG".to_string(),
            duration_years: 5,
            work_package_count: 3,
            work_package_names: vec![
                Some("Management".to_string()),
                None,
                Some("Dissemination".to_string()),
            ],
            work_package_start_months: vec![1, 13, 37],
            work_package_end_months: vec![60, 48, 60],
            default_inflation_rate_pct: rust_decimal_macros::dec!(10),
            try_eur_rate: rust_decimal_macros::dec!(50.62),
            indirect_cost_rate_pct: rust_decimal_macros::dec!(25),
            rate_version_id: "from_2025_05_13".to_string(),
            call_opening_date: Some("2026-01-15".to_string()),
        };

        let dto = project_config_to_dto(&config);

        assert_eq!(dto.project_title, config.project_title);
        assert_eq!(dto.pi_name, config.pi_name);
        assert_eq!(dto.call_reference, config.call_reference);
        assert_eq!(dto.duration_years, config.duration_years);
        assert_eq!(dto.work_package_count, config.work_package_count);
        assert_eq!(dto.work_package_names, config.work_package_names);
        assert_eq!(
            dto.work_package_start_months,
            config.work_package_start_months
        );
        assert_eq!(dto.work_package_end_months, config.work_package_end_months);
        assert_eq!(
            dto.default_inflation_rate_pct,
            config.default_inflation_rate_pct
        );
        assert_eq!(dto.try_eur_rate, config.try_eur_rate);
        assert_eq!(dto.indirect_cost_rate_pct, config.indirect_cost_rate_pct);
        assert_eq!(dto.rate_version_id, config.rate_version_id);
        assert_eq!(dto.call_opening_date, config.call_opening_date);
    }
}
