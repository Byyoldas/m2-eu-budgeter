//! IPC commands for equipment item management.

use tauri::State;
use uuid::Uuid;
use crate::AppState;
use crate::domain::entities::EquipmentItem;
use crate::domain::dto::{EquipmentItemInputDto, BudgetSummaryDto, EquipmentPreviewDto};
use crate::validation::validate_equipment_item;
use crate::calculation::calculate_budget_summary;
use crate::calculation::equipment_depreciation::calculate_depreciation;
use crate::persistence::auto_save;
use crate::error::AppError;

/// Add a new equipment item to the project.
#[tauri::command]
pub fn add_equipment_item(
    state: State<'_, AppState>,
    input: EquipmentItemInputDto,
) -> Result<BudgetSummaryDto, AppError> {
    let mut lock = state.project.lock().unwrap();
    let project = lock.as_mut().ok_or(AppError::NoProject)?;

    validate_equipment_item(&input, project.config.duration_years, project.config.work_package_count)?;

    let item = EquipmentItem {
        id: Uuid::new_v4(),
        name: input.name,
        purchase_cost_eur: input.purchase_cost_eur,
        useful_lifetime_months: input.useful_lifetime_months,
        grant_usage_pct: input.grant_usage_pct,
        grant_usage_months: input.grant_usage_months,
        work_package_id: input.work_package_id,
    };
    project.equipment_items.push(item);

    let summary = calculate_budget_summary(project, &state.rate_data)?;

    let path_lock = state.project_path.lock().unwrap();
    let _ = auto_save(project, path_lock.as_deref());

    Ok(summary)
}

/// Update an existing equipment item by UUID.
#[tauri::command]
pub fn update_equipment_item(
    state: State<'_, AppState>,
    id: Uuid,
    input: EquipmentItemInputDto,
) -> Result<BudgetSummaryDto, AppError> {
    let mut lock = state.project.lock().unwrap();
    let project = lock.as_mut().ok_or(AppError::NoProject)?;

    validate_equipment_item(&input, project.config.duration_years, project.config.work_package_count)?;

    let item = project
        .equipment_items
        .iter_mut()
        .find(|i| i.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Equipment item {id} not found.")))?;

    item.name = input.name;
    item.purchase_cost_eur = input.purchase_cost_eur;
    item.useful_lifetime_months = input.useful_lifetime_months;
    item.grant_usage_pct = input.grant_usage_pct;
    item.grant_usage_months = input.grant_usage_months;
    item.work_package_id = input.work_package_id;

    let summary = calculate_budget_summary(project, &state.rate_data)?;

    let path_lock = state.project_path.lock().unwrap();
    let _ = auto_save(project, path_lock.as_deref());

    Ok(summary)
}

/// Delete an equipment item by UUID.
#[tauri::command]
pub fn delete_equipment_item(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<BudgetSummaryDto, AppError> {
    let mut lock = state.project.lock().unwrap();
    let project = lock.as_mut().ok_or(AppError::NoProject)?;

    let before = project.equipment_items.len();
    project.equipment_items.retain(|i| i.id != id);
    if project.equipment_items.len() == before {
        return Err(AppError::NotFound(format!("Equipment item {id} not found.")));
    }

    let summary = calculate_budget_summary(project, &state.rate_data)?;

    let path_lock = state.project_path.lock().unwrap();
    let _ = auto_save(project, path_lock.as_deref());

    Ok(summary)
}

/// Live depreciation preview for an item being edited — does NOT mutate state.
/// Used to show "Eligible Depreciation" in real time as the user types.
#[tauri::command]
pub fn preview_equipment_depreciation(
    input: EquipmentItemInputDto,
) -> Result<EquipmentPreviewDto, AppError> {
    let result = calculate_depreciation(
        input.purchase_cost_eur,
        input.useful_lifetime_months,
        input.grant_usage_pct,
        input.grant_usage_months,
    )?;

    Ok(EquipmentPreviewDto {
        theoretical_eligible_eur: result.theoretical_eligible_eur,
        maximum_eligible_eur: result.maximum_eligible_eur,
        is_capped: result.is_capped,
        eligible_depreciation_eur: result.eligible_depreciation_eur,
    })
}
