//! IPC commands for M-14 Reporting Period Management. Defaults are
//! pre-populated on project open (BR-RP-05, see
//! `commands::project::open_execution_project`); these commands let the PI
//! adjust the auto-populated periods or add custom ones.

use crate::commands::project::build_summary;
use crate::domain::dto::{ExecutionProjectSummaryDto, ReportingPeriodInputDto};
use crate::domain::execution_entities::ReportingPeriod;
use crate::error::AppError;
use crate::persistence;
use crate::validation::validate_reporting_period;
use crate::AppState;
use tauri::State;
use uuid::Uuid;

fn next_period_number(existing: &[ReportingPeriod]) -> u32 {
    existing.iter().map(|p| p.period_number).max().unwrap_or(0) + 1
}

#[tauri::command]
pub fn add_reporting_period(
    state: State<'_, AppState>,
    input: ReportingPeriodInputDto,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;
    let max_month = project.config.duration_years as u32 * 12;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    validate_reporting_period(&input, &exec.reporting_periods, max_month, None)?;

    let period_number = next_period_number(&exec.reporting_periods);
    exec.reporting_periods.push(ReportingPeriod {
        id: Uuid::new_v4(),
        period_number,
        start_month: input.start_month,
        end_month: input.end_month,
        submission_deadline: input.submission_deadline,
        technical_report_submitted: input.technical_report_submitted,
        financial_report_submitted: input.financial_report_submitted,
        status: input.status,
    });

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}

#[tauri::command]
pub fn update_reporting_period(
    state: State<'_, AppState>,
    id: Uuid,
    input: ReportingPeriodInputDto,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;
    let max_month = project.config.duration_years as u32 * 12;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    validate_reporting_period(&input, &exec.reporting_periods, max_month, Some(id))?;

    let period = exec
        .reporting_periods
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Reporting period '{id}' not found.")))?;
    period.start_month = input.start_month;
    period.end_month = input.end_month;
    period.submission_deadline = input.submission_deadline;
    period.technical_report_submitted = input.technical_report_submitted;
    period.financial_report_submitted = input.financial_report_submitted;
    period.status = input.status;

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}

#[tauri::command]
pub fn delete_reporting_period(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    let before = exec.reporting_periods.len();
    exec.reporting_periods.retain(|p| p.id != id);
    if exec.reporting_periods.len() == before {
        return Err(AppError::NotFound(format!(
            "Reporting period '{id}' not found."
        )));
    }

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}
