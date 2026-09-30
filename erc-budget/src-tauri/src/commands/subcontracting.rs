//! IPC commands for Subcontracting (Category B) items.
//!
//! Entered item-by-item, the same way Other Direct Cost items are —
//! see `commands::other_costs` for the C3 equivalent this mirrors.

use crate::calculation::calculate_budget_summary;
use crate::domain::dto::{BudgetSummaryDto, SubcontractingInputDto};
use crate::domain::entities::SubcontractingItem;
use crate::error::AppError;
use crate::persistence::auto_save;
use crate::validation::validate_subcontracting_item;
use crate::AppState;
use tauri::State;
use uuid::Uuid;

/// Add a new Subcontracting item.
#[tauri::command]
pub fn add_subcontracting_item(
    state: State<'_, AppState>,
    input: SubcontractingInputDto,
) -> Result<BudgetSummaryDto, AppError> {
    let mut lock = state.project.lock().unwrap();
    let project = lock.as_mut().ok_or(AppError::NoProject)?;

    validate_subcontracting_item(
        &input,
        project.config.work_package_count,
        &project.subcontracting_items,
    )?;

    let item = SubcontractingItem {
        id: Uuid::new_v4(),
        name: input.name,
        amount_eur: input.amount_eur,
        notes: input.notes,
        work_package_ids: input.work_package_ids,
    };
    project.subcontracting_items.push(item);

    let summary = calculate_budget_summary(project, &state.rate_data)?;

    let path_lock = state.project_path.lock().unwrap();
    let _ = auto_save(project, path_lock.as_deref());

    Ok(summary)
}

/// Update an existing Subcontracting item by UUID.
#[tauri::command]
pub fn update_subcontracting_item(
    state: State<'_, AppState>,
    id: Uuid,
    input: SubcontractingInputDto,
) -> Result<BudgetSummaryDto, AppError> {
    let mut lock = state.project.lock().unwrap();
    let project = lock.as_mut().ok_or(AppError::NoProject)?;

    {
        let others: Vec<_> = project
            .subcontracting_items
            .iter()
            .filter(|i| i.id != id)
            .cloned()
            .collect();
        validate_subcontracting_item(&input, project.config.work_package_count, &others)?;
    }

    let item = project
        .subcontracting_items
        .iter_mut()
        .find(|i| i.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Subcontracting item {id} not found.")))?;

    item.name = input.name;
    item.amount_eur = input.amount_eur;
    item.notes = input.notes;
    item.work_package_ids = input.work_package_ids;

    let summary = calculate_budget_summary(project, &state.rate_data)?;

    let path_lock = state.project_path.lock().unwrap();
    let _ = auto_save(project, path_lock.as_deref());

    Ok(summary)
}

/// Delete a Subcontracting item by UUID.
#[tauri::command]
pub fn delete_subcontracting_item(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<BudgetSummaryDto, AppError> {
    let mut lock = state.project.lock().unwrap();
    let project = lock.as_mut().ok_or(AppError::NoProject)?;

    let before = project.subcontracting_items.len();
    project.subcontracting_items.retain(|i| i.id != id);
    if project.subcontracting_items.len() == before {
        return Err(AppError::NotFound(format!(
            "Subcontracting item {id} not found."
        )));
    }

    let summary = calculate_budget_summary(project, &state.rate_data)?;

    let path_lock = state.project_path.lock().unwrap();
    let _ = auto_save(project, path_lock.as_deref());

    Ok(summary)
}
