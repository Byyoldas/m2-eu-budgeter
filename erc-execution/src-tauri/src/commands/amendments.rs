//! IPC commands for Amendment Management — a from-scratch design (see
//! `domain::enums::AmendmentType` doc comment for why: `development-roadmap.md`
//! names this module but no business rules, DTOs, or UX exist for it anywhere
//! in the planning docs).

use crate::commands::project::build_summary;
use crate::domain::dto::{AmendmentInputDto, ExecutionProjectSummaryDto};
use crate::domain::execution_entities::Amendment;
use crate::error::AppError;
use crate::persistence;
use crate::validation::validate_amendment;
use crate::AppState;
use tauri::State;
use uuid::Uuid;

/// BR-AMD-01: sequential, immutable once assigned ("AMD-1", "AMD-2", ...).
/// Based on the count of existing amendments rather than a stored counter —
/// simple and correct as long as amendments are never renumbered on delete
/// (they aren't; a gap after a delete is expected and fine for an audit log).
fn next_amendment_number(existing: &[Amendment]) -> String {
    format!("AMD-{}", existing.len() + 1)
}

#[tauri::command]
pub fn record_amendment(
    state: State<'_, AppState>,
    input: AmendmentInputDto,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    validate_amendment(&input, project.config.work_package_count)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    let amendment_number = next_amendment_number(&exec.amendments);
    exec.amendments.push(Amendment {
        id: Uuid::new_v4(),
        amendment_number,
        amendment_type: input.amendment_type,
        title: input.title,
        description: input.description,
        requested_date: input.requested_date,
        decision_date: input.decision_date,
        status: input.status,
        financial_impact_eur: input.financial_impact_eur,
        affected_work_package_ids: input.affected_work_package_ids,
        notes: input.notes,
    });

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}

#[tauri::command]
pub fn update_amendment(
    state: State<'_, AppState>,
    id: Uuid,
    input: AmendmentInputDto,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    validate_amendment(&input, project.config.work_package_count)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    let amendment = exec
        .amendments
        .iter_mut()
        .find(|a| a.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Amendment '{id}' not found.")))?;
    // amendment_number is immutable (BR-AMD-01) — every other field updates.
    amendment.amendment_type = input.amendment_type;
    amendment.title = input.title;
    amendment.description = input.description;
    amendment.requested_date = input.requested_date;
    amendment.decision_date = input.decision_date;
    amendment.status = input.status;
    amendment.financial_impact_eur = input.financial_impact_eur;
    amendment.affected_work_package_ids = input.affected_work_package_ids;
    amendment.notes = input.notes;

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}

#[tauri::command]
pub fn delete_amendment(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    let before = exec.amendments.len();
    exec.amendments.retain(|a| a.id != id);
    if exec.amendments.len() == before {
        return Err(AppError::NotFound(format!("Amendment '{id}' not found.")));
    }

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}
