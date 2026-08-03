//! IPC commands for M-06 Milestone Tracking.

use crate::commands::project::build_summary;
use crate::domain::dto::{ExecutionProjectSummaryDto, MilestoneInputDto};
use crate::domain::enums::MilestoneStatus;
use crate::domain::execution_entities::Milestone;
use crate::engines::progress_engine::derive_current_project_month;
use crate::error::AppError;
use crate::persistence;
use crate::validation::validate_milestone;
use crate::AppState;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn add_milestone(
    state: State<'_, AppState>,
    input: MilestoneInputDto,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let max_month = project.config.duration_years as u32 * 12;
    validate_milestone(&input, project.config.work_package_count, max_month)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    exec.milestones.push(Milestone {
        id: Uuid::new_v4(),
        title: input.title,
        work_package_id: input.work_package_id,
        planned_month: input.planned_month,
        status: input.status,
        actual_completion_month: input.actual_completion_month,
    });

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}

#[tauri::command]
pub fn update_milestone(
    state: State<'_, AppState>,
    id: Uuid,
    input: MilestoneInputDto,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let max_month = project.config.duration_years as u32 * 12;
    validate_milestone(&input, project.config.work_package_count, max_month)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    let milestone = exec
        .milestones
        .iter_mut()
        .find(|m| m.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Milestone '{id}' not found.")))?;
    milestone.title = input.title;
    milestone.work_package_id = input.work_package_id;
    milestone.planned_month = input.planned_month;
    milestone.status = input.status;
    milestone.actual_completion_month = input.actual_completion_month;

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}

/// Convenience command: marks a milestone `Completed` at the current project
/// month in one call (BR-MS-02's deliverable-acceptance gate is deferred —
/// Deliverable Tracking, M-05, doesn't exist yet).
#[tauri::command]
pub fn complete_milestone(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;
    let current_month = derive_current_project_month(project);

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    let milestone = exec
        .milestones
        .iter_mut()
        .find(|m| m.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Milestone '{id}' not found.")))?;
    milestone.status = MilestoneStatus::Completed;
    milestone.actual_completion_month = Some(current_month);

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}

#[tauri::command]
pub fn delete_milestone(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    let before = exec.milestones.len();
    exec.milestones.retain(|m| m.id != id);
    if exec.milestones.len() == before {
        return Err(AppError::NotFound(format!("Milestone '{id}' not found.")));
    }

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}
