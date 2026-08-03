//! IPC commands for M-08 Travel Tracking.

use crate::commands::project::build_summary;
use crate::domain::dto::{ExecutionProjectSummaryDto, TripExecutionInputDto};
use crate::domain::execution_entities::TripExecution;
use crate::error::AppError;
use crate::persistence;
use crate::validation::validate_trip_execution;
use crate::AppState;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn add_trip_execution(
    state: State<'_, AppState>,
    input: TripExecutionInputDto,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    validate_trip_execution(
        &input,
        &project.trips,
        &exec.persons,
        &exec.trip_executions,
        None,
    )?;

    exec.trip_executions.push(TripExecution {
        id: Uuid::new_v4(),
        trip_id: input.trip_id,
        instance_number: input.instance_number,
        traveller_person_id: input.traveller_person_id,
        actual_travel_date: input.actual_travel_date,
        actual_cost_eur: input.actual_cost_eur,
        status: input.status,
    });

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}

#[tauri::command]
pub fn update_trip_execution(
    state: State<'_, AppState>,
    id: Uuid,
    input: TripExecutionInputDto,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    validate_trip_execution(
        &input,
        &project.trips,
        &exec.persons,
        &exec.trip_executions,
        Some(id),
    )?;

    let record = exec
        .trip_executions
        .iter_mut()
        .find(|t| t.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Trip execution '{id}' not found.")))?;
    record.trip_id = input.trip_id;
    record.instance_number = input.instance_number;
    record.traveller_person_id = input.traveller_person_id;
    record.actual_travel_date = input.actual_travel_date;
    record.actual_cost_eur = input.actual_cost_eur;
    record.status = input.status;

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}

#[tauri::command]
pub fn delete_trip_execution(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    let before = exec.trip_executions.len();
    exec.trip_executions.retain(|t| t.id != id);
    if exec.trip_executions.len() == before {
        return Err(AppError::NotFound(format!(
            "Trip execution '{id}' not found."
        )));
    }

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}
