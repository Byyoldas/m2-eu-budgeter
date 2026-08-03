//! IPC commands for M-10 Other Costs Tracking.

use crate::commands::project::build_summary;
use crate::domain::dto::{ActualCostEntryInputDto, ExecutionProjectSummaryDto};
use crate::domain::execution_entities::ActualCostEntry;
use crate::error::AppError;
use crate::persistence;
use crate::validation::validate_actual_cost_entry;
use crate::AppState;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn add_actual_cost_entry(
    state: State<'_, AppState>,
    input: ActualCostEntryInputDto,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    validate_actual_cost_entry(&input, &project.other_cost_items)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    exec.actual_cost_entries.push(ActualCostEntry {
        id: Uuid::new_v4(),
        linked_entity_id: input.linked_entity_id,
        amount_eur: input.amount_eur,
        description: input.description,
        incurred_date: input.incurred_date,
        status: input.status,
        justification: input.justification,
    });

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}

#[tauri::command]
pub fn update_actual_cost_entry(
    state: State<'_, AppState>,
    id: Uuid,
    input: ActualCostEntryInputDto,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    validate_actual_cost_entry(&input, &project.other_cost_items)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    let entry = exec
        .actual_cost_entries
        .iter_mut()
        .find(|e| e.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Actual cost entry '{id}' not found.")))?;
    entry.linked_entity_id = input.linked_entity_id;
    entry.amount_eur = input.amount_eur;
    entry.description = input.description;
    entry.incurred_date = input.incurred_date;
    entry.status = input.status;
    entry.justification = input.justification;

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}

#[tauri::command]
pub fn delete_actual_cost_entry(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    let before = exec.actual_cost_entries.len();
    exec.actual_cost_entries.retain(|e| e.id != id);
    if exec.actual_cost_entries.len() == before {
        return Err(AppError::NotFound(format!(
            "Actual cost entry '{id}' not found."
        )));
    }

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}
