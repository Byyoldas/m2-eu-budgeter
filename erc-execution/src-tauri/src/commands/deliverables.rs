//! IPC commands for M-05 Deliverable Tracking.

use crate::commands::project::build_summary;
use crate::domain::dto::{DeliverableInputDto, ExecutionProjectSummaryDto};
use crate::domain::execution_entities::Deliverable;
use crate::error::AppError;
use crate::persistence;
use crate::validation::validate_deliverable;
use crate::AppState;
use tauri::State;
use uuid::Uuid;

/// BR-DEL-02: `D{wp_id}.{sequence}`, server-assigned and immutable once
/// created — same "count existing, don't renumber on delete" pattern as
/// Amendment Management's `AMD-N` (see `commands::amendments::next_amendment_number`).
fn next_deliverable_number(work_package_id: u8, existing: &[Deliverable]) -> String {
    let sequence = existing
        .iter()
        .filter(|d| d.work_package_id == work_package_id)
        .count()
        + 1;
    format!("D{work_package_id}.{sequence}")
}

#[tauri::command]
pub fn add_deliverable(
    state: State<'_, AppState>,
    input: DeliverableInputDto,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let max_month = project.config.duration_years as u32 * 12;
    validate_deliverable(
        &input,
        project.config.work_package_count,
        max_month,
        &project.personnel_roles,
    )?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    let deliverable_number = next_deliverable_number(input.work_package_id, &exec.deliverables);
    exec.deliverables.push(Deliverable {
        id: Uuid::new_v4(),
        deliverable_number,
        title: input.title,
        deliverable_type: input.deliverable_type,
        work_package_id: input.work_package_id,
        planned_month: input.planned_month,
        responsible_role_id: input.responsible_role_id,
        dissemination_level: input.dissemination_level,
        status: input.status,
        actual_submission_date: input.actual_submission_date,
        revision_note: input.revision_note,
        revised_planned_month: input.revised_planned_month,
        cordis_registered: input.cordis_registered,
        notes: input.notes,
    });

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}

#[tauri::command]
pub fn update_deliverable(
    state: State<'_, AppState>,
    id: Uuid,
    input: DeliverableInputDto,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let max_month = project.config.duration_years as u32 * 12;
    validate_deliverable(
        &input,
        project.config.work_package_count,
        max_month,
        &project.personnel_roles,
    )?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    let deliverable = exec
        .deliverables
        .iter_mut()
        .find(|d| d.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Deliverable '{id}' not found.")))?;
    // deliverable_number and work_package_id are immutable once created
    // (BR-DEL-02's numbering is anchored to the WP it was created under).
    deliverable.title = input.title;
    deliverable.deliverable_type = input.deliverable_type;
    deliverable.planned_month = input.planned_month;
    deliverable.responsible_role_id = input.responsible_role_id;
    deliverable.dissemination_level = input.dissemination_level;
    deliverable.status = input.status;
    deliverable.actual_submission_date = input.actual_submission_date;
    deliverable.revision_note = input.revision_note;
    deliverable.revised_planned_month = input.revised_planned_month;
    deliverable.cordis_registered = input.cordis_registered;
    deliverable.notes = input.notes;

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}

#[tauri::command]
pub fn delete_deliverable(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    let before = exec.deliverables.len();
    exec.deliverables.retain(|d| d.id != id);
    if exec.deliverables.len() == before {
        return Err(AppError::NotFound(format!("Deliverable '{id}' not found.")));
    }
    // A deleted deliverable can no longer be linked from any milestone.
    for milestone in exec.milestones.iter_mut() {
        milestone.linked_deliverable_ids.retain(|d_id| *d_id != id);
    }

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}
