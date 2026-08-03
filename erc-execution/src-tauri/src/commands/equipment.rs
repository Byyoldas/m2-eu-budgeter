//! IPC commands for M-09 Equipment Tracking.

use crate::commands::project::build_summary;
use crate::domain::dto::{EquipmentProcurementInputDto, ExecutionProjectSummaryDto};
use crate::domain::execution_entities::EquipmentProcurement;
use crate::error::AppError;
use crate::persistence;
use crate::validation::validate_equipment_procurement;
use crate::AppState;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn add_equipment_procurement(
    state: State<'_, AppState>,
    input: EquipmentProcurementInputDto,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    validate_equipment_procurement(&input, &project.equipment_items)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    exec.equipment_procurements.push(EquipmentProcurement {
        id: Uuid::new_v4(),
        equipment_item_id: input.equipment_item_id,
        actual_purchase_cost_eur: input.actual_purchase_cost_eur,
        purchase_date: input.purchase_date,
        delivery_confirmed: input.delivery_confirmed,
    });

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}

#[tauri::command]
pub fn update_equipment_procurement(
    state: State<'_, AppState>,
    id: Uuid,
    input: EquipmentProcurementInputDto,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    validate_equipment_procurement(&input, &project.equipment_items)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    let record = exec
        .equipment_procurements
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Equipment procurement '{id}' not found.")))?;
    record.equipment_item_id = input.equipment_item_id;
    record.actual_purchase_cost_eur = input.actual_purchase_cost_eur;
    record.purchase_date = input.purchase_date;
    record.delivery_confirmed = input.delivery_confirmed;

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}

#[tauri::command]
pub fn delete_equipment_procurement(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let mut exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_mut().ok_or(AppError::NoProject)?;

    let before = exec.equipment_procurements.len();
    exec.equipment_procurements.retain(|p| p.id != id);
    if exec.equipment_procurements.len() == before {
        return Err(AppError::NotFound(format!(
            "Equipment procurement '{id}' not found."
        )));
    }

    let summary = build_summary(project, exec, &state)?;
    if let Some(path) = state.project_path.lock().unwrap().as_deref() {
        persistence::auto_save(project, exec, path)?;
    }
    Ok(summary)
}
