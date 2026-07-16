//! IPC commands for Other Direct Costs (C3) and subcontracting (B).
//!
//! CFS items are managed separately from regular OC items:
//! - Regular items: created/updated/deleted via add/update/delete_other_cost
//! - CFS item: added via add_cfs_item (auto-fill with official name and is_cfs_item=true)
//! - CFS warning: dismissed via dismiss_cfs_warning (sets flag, does not add item)

use tauri::State;
use uuid::Uuid;
use rust_decimal::Decimal;
use crate::AppState;
use crate::domain::entities::OtherDirectCostItem;
use crate::domain::dto::{OtherCostInputDto, BudgetSummaryDto};
use crate::validation::validate_other_cost;
use crate::calculation::calculate_budget_summary;
use crate::persistence::auto_save;
use crate::error::AppError;

const CFS_ITEM_NAME: &str = "Certificate on Financial Statements (CFS)";

/// Add a regular Other Direct Cost item (C3).
#[tauri::command]
pub fn add_other_cost(
    state: State<'_, AppState>,
    input: OtherCostInputDto,
) -> Result<BudgetSummaryDto, AppError> {
    let mut lock = state.project.lock().unwrap();
    let project = lock.as_mut().ok_or(AppError::NoProject)?;

    validate_other_cost(&input, project.config.work_package_count, &project.other_cost_items)?;

    let item = OtherDirectCostItem {
        id: Uuid::new_v4(),
        name: input.name,
        amount_eur: input.amount_eur,
        is_cfs_item: false, // regular items are never CFS items
        notes: input.notes,
        work_package_ids: input.work_package_ids,
    };
    project.other_cost_items.push(item);

    let summary = calculate_budget_summary(project, &state.rate_data)?;

    let path_lock = state.project_path.lock().unwrap();
    let _ = auto_save(project, path_lock.as_deref());

    Ok(summary)
}

/// Update a regular Other Direct Cost item by UUID.
#[tauri::command]
pub fn update_other_cost(
    state: State<'_, AppState>,
    id: Uuid,
    input: OtherCostInputDto,
) -> Result<BudgetSummaryDto, AppError> {
    let mut lock = state.project.lock().unwrap();
    let project = lock.as_mut().ok_or(AppError::NoProject)?;

    {
        // Build a list of existing items excluding this one for uniqueness check
        let others: Vec<_> = project.other_cost_items.iter()
            .filter(|i| i.id != id)
            .cloned()
            .collect();
        validate_other_cost(&input, project.config.work_package_count, &others)?;
    }

    let item = project
        .other_cost_items
        .iter_mut()
        .find(|i| i.id == id && !i.is_cfs_item)
        .ok_or_else(|| AppError::NotFound(format!("Other cost item {id} not found (or is a CFS item).")))?;

    item.name = input.name;
    item.amount_eur = input.amount_eur;
    item.notes = input.notes;
    item.work_package_ids = input.work_package_ids;

    let summary = calculate_budget_summary(project, &state.rate_data)?;

    let path_lock = state.project_path.lock().unwrap();
    let _ = auto_save(project, path_lock.as_deref());

    Ok(summary)
}

/// Delete a regular Other Direct Cost item by UUID.
/// CFS items cannot be deleted via this command; use remove_cfs_item instead.
#[tauri::command]
pub fn delete_other_cost(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<BudgetSummaryDto, AppError> {
    let mut lock = state.project.lock().unwrap();
    let project = lock.as_mut().ok_or(AppError::NoProject)?;

    let before = project.other_cost_items.len();
    project.other_cost_items.retain(|i| i.id != id || i.is_cfs_item);
    if project.other_cost_items.len() == before {
        return Err(AppError::NotFound(
            format!("Other cost item {id} not found or is a CFS item (use remove_cfs_item).")
        ));
    }

    let summary = calculate_budget_summary(project, &state.rate_data)?;

    let path_lock = state.project_path.lock().unwrap();
    let _ = auto_save(project, path_lock.as_deref());

    Ok(summary)
}

/// Add the Certificate on Financial Statements (CFS) as a C3 item.
/// Called when the user accepts the CFS prompt.
/// Prevents double-addition if a CFS item already exists.
#[tauri::command(rename_all = "snake_case")]
pub fn add_cfs_item(
    state: State<'_, AppState>,
    amount_eur: Decimal,
    work_package_ids: Vec<u8>,
) -> Result<BudgetSummaryDto, AppError> {
    let mut lock = state.project.lock().unwrap();
    let project = lock.as_mut().ok_or(AppError::NoProject)?;

    // Guard: prevent adding a second CFS item
    if project.has_cfs_item() {
        return Err(AppError::Validation(vec![
            crate::error::FieldError::entity(
                "DUPLICATE_CFS_ITEM",
                "A Certificate on Financial Statements item already exists. Remove the existing one before adding another.",
            )
        ]));
    }

    if amount_eur <= Decimal::ZERO {
        return Err(AppError::Validation(vec![
            crate::error::FieldError::new("amount_eur", "INVALID_C3_AMOUNT", "CFS amount must be greater than zero."),
        ]));
    }

    if work_package_ids.is_empty() {
        return Err(AppError::Validation(vec![
            crate::error::FieldError::new("work_package_ids", "NO_WORK_PACKAGE", "At least one Work Package must be selected."),
        ]));
    }
    let work_package_count = project.config.work_package_count;
    if work_package_ids.iter().any(|&wp| wp < 1 || wp > work_package_count) {
        return Err(AppError::Validation(vec![
            crate::error::FieldError::new("work_package_ids", "WP_OUT_OF_RANGE", "Select valid Work Packages."),
        ]));
    }

    let item = OtherDirectCostItem {
        id: Uuid::new_v4(),
        name: CFS_ITEM_NAME.to_string(),
        amount_eur,
        is_cfs_item: true,
        notes: Some("Auto-added: ERC requires a Certificate on Financial Statements when total budget exceeds €430,000.".to_string()),
        work_package_ids,
    };
    project.other_cost_items.push(item);
    project.cfs_warning_dismissed = false; // Reset dismissal since we now have the item

    let summary = calculate_budget_summary(project, &state.rate_data)?;

    let path_lock = state.project_path.lock().unwrap();
    let _ = auto_save(project, path_lock.as_deref());

    Ok(summary)
}

/// Remove the CFS item (if any).
#[tauri::command]
pub fn remove_cfs_item(
    state: State<'_, AppState>,
) -> Result<BudgetSummaryDto, AppError> {
    let mut lock = state.project.lock().unwrap();
    let project = lock.as_mut().ok_or(AppError::NoProject)?;

    project.other_cost_items.retain(|i| !i.is_cfs_item);

    let summary = calculate_budget_summary(project, &state.rate_data)?;

    let path_lock = state.project_path.lock().unwrap();
    let _ = auto_save(project, path_lock.as_deref());

    Ok(summary)
}

/// Dismiss the CFS warning without adding the item.
/// Sets a flag so the prompt does not reappear in this session.
#[tauri::command]
pub fn dismiss_cfs_warning(
    state: State<'_, AppState>,
) -> Result<BudgetSummaryDto, AppError> {
    let mut lock = state.project.lock().unwrap();
    let project = lock.as_mut().ok_or(AppError::NoProject)?;

    project.cfs_warning_dismissed = true;

    let summary = calculate_budget_summary(project, &state.rate_data)?;

    let path_lock = state.project_path.lock().unwrap();
    let _ = auto_save(project, path_lock.as_deref());

    Ok(summary)
}

/// Set the subcontracting (category B) total for the project.
/// Subcontracting is a single project-wide value, not a list of items.
#[tauri::command(rename_all = "snake_case")]
pub fn set_subcontracting(
    state: State<'_, AppState>,
    amount_eur: Decimal,
    work_package_id: u8,
) -> Result<BudgetSummaryDto, AppError> {
    if amount_eur < Decimal::ZERO {
        return Err(AppError::Validation(vec![
            crate::error::FieldError::new("amount_eur", "INVALID_SUBCONTRACTING", "Subcontracting amount cannot be negative."),
        ]));
    }

    let mut lock = state.project.lock().unwrap();
    let project = lock.as_mut().ok_or(AppError::NoProject)?;

    if work_package_id < 1 || work_package_id > project.config.work_package_count {
        return Err(AppError::Validation(vec![
            crate::error::FieldError::new("work_package_id", "WP_OUT_OF_RANGE", "Select a valid Work Package."),
        ]));
    }

    project.subcontracting.amount_eur = amount_eur;
    project.subcontracting.work_package_id = work_package_id;

    let summary = calculate_budget_summary(project, &state.rate_data)?;

    let path_lock = state.project_path.lock().unwrap();
    let _ = auto_save(project, path_lock.as_deref());

    Ok(summary)
}
