//! IPC commands for M-03 `PersonMonthRecord`s (planned-vs-actual FTE-months
//! per calendar project month — see the scoping note in
//! `domain::execution_entities` on why this is per-month, not per-period).

use crate::commands::project::build_summary;
use crate::domain::dto::{ExecutionProjectSummaryDto, PersonMonthRecordInputDto};
use crate::domain::execution_entities::PersonMonthRecord;
use crate::engines::progress_engine::calculate_planned_fte_months_for_month;
use crate::error::AppError;
use crate::persistence;
use crate::validation::validate_person_month_record;
use crate::AppState;
use rust_decimal::Decimal;
use tauri::State;
use uuid::Uuid;

/// Sum of `approved_months` for every record in `project_month`, optionally
/// excluding one record id (used when updating that same record).
fn approved_total_for_month(
    records: &[PersonMonthRecord],
    project_month: u32,
    exclude_id: Option<Uuid>,
) -> Decimal {
    records
        .iter()
        .filter(|r| r.project_month == project_month && Some(r.id) != exclude_id)
        .filter_map(|r| r.approved_months)
        .sum()
}

#[tauri::command]
pub fn add_person_month_record(
    state: State<'_, AppState>,
    input: PersonMonthRecordInputDto,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    let planned =
        calculate_planned_fte_months_for_month(&project.personnel_roles, input.project_month);
    let approved_excluding_self =
        approved_total_for_month(&exec.person_month_records, input.project_month, None);

    validate_person_month_record(&input, &exec.persons, planned, approved_excluding_self)?;

    exec.person_month_records.push(PersonMonthRecord {
        id: Uuid::new_v4(),
        person_id: input.person_id,
        project_month: input.project_month,
        reported_months: input.reported_months,
        approved_months: input.approved_months,
    });

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}

#[tauri::command]
pub fn update_person_month_record(
    state: State<'_, AppState>,
    id: Uuid,
    input: PersonMonthRecordInputDto,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    let planned =
        calculate_planned_fte_months_for_month(&project.personnel_roles, input.project_month);
    let approved_excluding_self =
        approved_total_for_month(&exec.person_month_records, input.project_month, Some(id));

    validate_person_month_record(&input, &exec.persons, planned, approved_excluding_self)?;

    let record = exec
        .person_month_records
        .iter_mut()
        .find(|r| r.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Person-month record '{id}' not found.")))?;
    record.person_id = input.person_id;
    record.project_month = input.project_month;
    record.reported_months = input.reported_months;
    record.approved_months = input.approved_months;

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}

#[tauri::command]
pub fn delete_person_month_record(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    let before = exec.person_month_records.len();
    exec.person_month_records.retain(|r| r.id != id);
    if exec.person_month_records.len() == before {
        return Err(AppError::NotFound(format!(
            "Person-month record '{id}' not found."
        )));
    }

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}
