//! IPC commands for M-11 Subcontracting Tracking.

use crate::commands::project::build_summary;
use crate::domain::dto::{ExecutionProjectSummaryDto, SubcontractingLineInputDto};
use crate::domain::execution_entities::SubcontractingLine;
use crate::error::AppError;
use crate::persistence;
use crate::validation::validate_subcontracting_line;
use crate::AppState;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn add_subcontracting_line(
    state: State<'_, AppState>,
    input: SubcontractingLineInputDto,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    validate_subcontracting_line(
        &input,
        &exec.subcontracting_lines,
        project.subcontracting.amount_eur,
        project.config.work_package_count,
        None,
    )?;

    exec.subcontracting_lines.push(SubcontractingLine {
        id: Uuid::new_v4(),
        vendor: input.vendor,
        contract_reference: input.contract_reference,
        amount_eur: input.amount_eur,
        work_package_id: input.work_package_id,
        status: input.status,
        vendor_is_host_institution: input.vendor_is_host_institution,
        payment_date: input.payment_date,
    });

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}

#[tauri::command]
pub fn update_subcontracting_line(
    state: State<'_, AppState>,
    id: Uuid,
    input: SubcontractingLineInputDto,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    validate_subcontracting_line(
        &input,
        &exec.subcontracting_lines,
        project.subcontracting.amount_eur,
        project.config.work_package_count,
        Some(id),
    )?;

    let line = exec
        .subcontracting_lines
        .iter_mut()
        .find(|l| l.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Subcontracting line '{id}' not found.")))?;
    line.vendor = input.vendor;
    line.contract_reference = input.contract_reference;
    line.amount_eur = input.amount_eur;
    line.work_package_id = input.work_package_id;
    line.status = input.status;
    line.vendor_is_host_institution = input.vendor_is_host_institution;
    line.payment_date = input.payment_date;

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}

#[tauri::command]
pub fn delete_subcontracting_line(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    let before = exec.subcontracting_lines.len();
    exec.subcontracting_lines.retain(|l| l.id != id);
    if exec.subcontracting_lines.len() == before {
        return Err(AppError::NotFound(format!(
            "Subcontracting line '{id}' not found."
        )));
    }

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}
