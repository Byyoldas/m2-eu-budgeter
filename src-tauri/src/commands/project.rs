//! IPC commands for project lifecycle management.
//!
//! Every command acquires the project mutex, performs its operation,
//! and releases the lock before returning. Commands never hold the lock
//! across an await point.

use tauri::State;
use crate::AppState;
use crate::domain::entities::{Project, ProjectConfig};
use crate::domain::dto::{ProjectConfigDto, BudgetSummaryDto};
use crate::domain::rate_data::{RateVersionSummary, CountrySummary};
use crate::validation::validate_project_config;
use crate::calculation::calculate_budget_summary;
use crate::persistence;
use crate::error::AppError;

/// Create a new empty project from configuration.
/// Returns the initial (zero) BudgetSummaryDto so the right panel renders immediately.
#[tauri::command]
pub fn create_project(
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
        work_package_start_years: config.work_package_start_years,
        work_package_end_years: config.work_package_end_years,
        default_inflation_rate_pct: config.default_inflation_rate_pct,
        try_eur_rate: config.try_eur_rate,
        indirect_cost_rate_pct: config.indirect_cost_rate_pct,
        rate_version_id: config.rate_version_id,
        call_opening_date: config.call_opening_date,
    };

    let project = Project::new(project_config);
    let summary = calculate_budget_summary(&project, &state.rate_data)?;

    let mut lock = state.project.lock().unwrap();
    *lock = Some(project);

    Ok(summary)
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
    project.config.work_package_start_years = config.work_package_start_years;
    project.config.work_package_end_years = config.work_package_end_years;
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
pub fn save_project_as(
    state: State<'_, AppState>,
    path: String,
) -> Result<(), AppError> {
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
    let version = state.rate_data.find_version(&version_id).ok_or_else(|| {
        AppError::NotFound(format!("Rate version '{version_id}' not found."))
    })?;
    Ok(version.sorted_countries())
}
