//! IPC commands for M-12 Risk Register.

use crate::commands::project::build_summary;
use crate::domain::dto::{ExecutionProjectSummaryDto, RiskEntryInputDto};
use crate::domain::execution_entities::RiskEntry;
use crate::error::AppError;
use crate::persistence;
use crate::validation::validate_risk_entry;
use crate::AppState;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn add_risk_entry(
    state: State<'_, AppState>,
    input: RiskEntryInputDto,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    validate_risk_entry(
        &input,
        &project.personnel_roles,
        project.config.work_package_count,
        None,
        chrono::Utc::now().date_naive(),
    )?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    exec.risks.push(RiskEntry {
        id: Uuid::new_v4(),
        title: input.title,
        description: input.description,
        work_package_id: input.work_package_id,
        probability: input.probability,
        impact: input.impact,
        mitigation: input.mitigation,
        status: input.status,
        owner_role_id: input.owner_role_id,
        identified_date: input.identified_date,
        review_date: input.review_date,
        closed_date: input.closed_date,
    });

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}

#[tauri::command]
pub fn update_risk_entry(
    state: State<'_, AppState>,
    id: Uuid,
    input: RiskEntryInputDto,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    let existing_status = exec
        .risks
        .iter()
        .find(|r| r.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Risk '{id}' not found.")))?
        .status;

    validate_risk_entry(
        &input,
        &project.personnel_roles,
        project.config.work_package_count,
        Some(existing_status),
        chrono::Utc::now().date_naive(),
    )?;

    let risk = exec.risks.iter_mut().find(|r| r.id == id).unwrap();
    risk.title = input.title;
    risk.description = input.description;
    risk.work_package_id = input.work_package_id;
    risk.probability = input.probability;
    risk.impact = input.impact;
    risk.mitigation = input.mitigation;
    risk.status = input.status;
    risk.owner_role_id = input.owner_role_id;
    risk.identified_date = input.identified_date;
    risk.review_date = input.review_date;
    risk.closed_date = input.closed_date;

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}

#[tauri::command]
pub fn delete_risk_entry(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    let before = exec.risks.len();
    exec.risks.retain(|r| r.id != id);
    if exec.risks.len() == before {
        return Err(AppError::NotFound(format!("Risk '{id}' not found.")));
    }
    // An issue linked to a deleted risk loses the (now dangling) reference.
    for issue in exec.issues.iter_mut() {
        if issue.linked_risk_id == Some(id) {
            issue.linked_risk_id = None;
        }
    }

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}
