//! IPC commands for M-03 Person records (the "who" side of Personnel &
//! Person-Month Tracking — see `commands::person_months` for the "how many
//! months" side).

use crate::commands::project::build_summary;
use crate::domain::dto::{ExecutionProjectSummaryDto, PersonInputDto};
use crate::domain::execution_entities::Person;
use crate::error::AppError;
use crate::persistence;
use crate::validation::validate_person;
use crate::AppState;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn add_person(
    state: State<'_, AppState>,
    input: PersonInputDto,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    validate_person(
        &input,
        &exec.persons,
        &project.personnel_roles,
        None,
        project.config.call_opening_date.as_deref(),
    )?;

    exec.persons.push(Person {
        id: Uuid::new_v4(),
        full_name: input.full_name,
        email: input.email,
        institution: input.institution,
        orcid: input.orcid,
        linked_role_id: input.linked_role_id,
        actual_start_date: input.actual_start_date,
        actual_end_date: input.actual_end_date,
    });

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}

#[tauri::command]
pub fn update_person(
    state: State<'_, AppState>,
    id: Uuid,
    input: PersonInputDto,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    validate_person(
        &input,
        &exec.persons,
        &project.personnel_roles,
        Some(id),
        project.config.call_opening_date.as_deref(),
    )?;

    let person = exec
        .persons
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Person '{id}' not found.")))?;
    person.full_name = input.full_name;
    person.email = input.email;
    person.institution = input.institution;
    person.orcid = input.orcid;
    person.linked_role_id = input.linked_role_id;
    person.actual_start_date = input.actual_start_date;
    person.actual_end_date = input.actual_end_date;

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}

#[tauri::command]
pub fn delete_person(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    let before = exec.persons.len();
    exec.persons.retain(|p| p.id != id);
    if exec.persons.len() == before {
        return Err(AppError::NotFound(format!("Person '{id}' not found.")));
    }
    // Cascade: a deleted person's month records no longer reference anyone.
    exec.person_month_records.retain(|r| r.person_id != id);

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}
