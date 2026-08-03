//! IPC commands for M-04 Work Package Management. WPs themselves are
//! read-only (sourced from `ProjectConfig`); this only lets the user set the
//! execution-side overlay (leader assignment, notes) — see
//! `domain::execution_entities::WorkPackageExecution`.

use crate::commands::project::build_summary;
use crate::domain::dto::{ExecutionProjectSummaryDto, WorkPackageExecutionInputDto};
use crate::domain::execution_entities::WorkPackageExecution;
use crate::error::AppError;
use crate::persistence;
use crate::validation::validate_work_package_execution;
use crate::AppState;
use tauri::State;

#[tauri::command(rename_all = "snake_case")]
pub fn set_work_package_execution(
    state: State<'_, AppState>,
    work_package_id: u8,
    input: WorkPackageExecutionInputDto,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    if work_package_id == 0 || work_package_id > project.config.work_package_count {
        return Err(AppError::NotFound(format!(
            "Work package '{work_package_id}' does not exist in this project."
        )));
    }

    validate_work_package_execution(&input, &project.personnel_roles)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    match exec
        .work_package_executions
        .iter_mut()
        .find(|w| w.work_package_id == work_package_id)
    {
        Some(existing) => {
            existing.leader_role_id = input.leader_role_id;
            existing.notes = input.notes;
        }
        None => exec.work_package_executions.push(WorkPackageExecution {
            work_package_id,
            leader_role_id: input.leader_role_id,
            notes: input.notes,
        }),
    }

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}
