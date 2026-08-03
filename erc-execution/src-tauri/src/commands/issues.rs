//! IPC commands for M-13 Issue Log.

use crate::commands::project::build_summary;
use crate::domain::dto::{ExecutionProjectSummaryDto, IssueEntryInputDto};
use crate::domain::execution_entities::IssueEntry;
use crate::error::AppError;
use crate::persistence;
use crate::validation::validate_issue_entry;
use crate::AppState;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn add_issue_entry(
    state: State<'_, AppState>,
    input: IssueEntryInputDto,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    validate_issue_entry(
        &input,
        &project.personnel_roles,
        project.config.work_package_count,
        &exec.risks,
        chrono::Utc::now().date_naive(),
    )?;

    exec.issues.push(IssueEntry {
        id: Uuid::new_v4(),
        description: input.description,
        work_package_id: input.work_package_id,
        raised_date: input.raised_date,
        priority: input.priority,
        owner_role_id: input.owner_role_id,
        status: input.status,
        resolution: input.resolution,
        linked_risk_id: input.linked_risk_id,
    });

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}

#[tauri::command]
pub fn update_issue_entry(
    state: State<'_, AppState>,
    id: Uuid,
    input: IssueEntryInputDto,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    validate_issue_entry(
        &input,
        &project.personnel_roles,
        project.config.work_package_count,
        &exec.risks,
        chrono::Utc::now().date_naive(),
    )?;

    let issue = exec
        .issues
        .iter_mut()
        .find(|i| i.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Issue '{id}' not found.")))?;
    issue.description = input.description;
    issue.work_package_id = input.work_package_id;
    issue.raised_date = input.raised_date;
    issue.priority = input.priority;
    issue.owner_role_id = input.owner_role_id;
    issue.status = input.status;
    issue.resolution = input.resolution;
    issue.linked_risk_id = input.linked_risk_id;

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}

#[tauri::command]
pub fn delete_issue_entry(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    let before = exec.issues.len();
    exec.issues.retain(|i| i.id != id);
    if exec.issues.len() == before {
        return Err(AppError::NotFound(format!("Issue '{id}' not found.")));
    }

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}
