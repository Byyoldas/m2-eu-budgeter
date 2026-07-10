//! IPC commands for personnel role management.

use tauri::State;
use uuid::Uuid;
use crate::AppState;
use crate::domain::entities::{PersonnelRole, RoleType};
use crate::domain::dto::{PersonnelRoleInputDto, BudgetSummaryDto, RoleCostPreviewDto, RoleCostLineDto};
use crate::validation::validate_personnel_role;
use crate::calculation::calculate_budget_summary;
use crate::calculation::salary_projection::{convert_try_to_eur, project_salary_chain};
use crate::calculation::personnel_cost::calculate_personnel_cost_lines;
use crate::persistence::auto_save;
use crate::error::AppError;

/// Add a new personnel role to the project.
/// Returns the full recalculated BudgetSummaryDto.
#[tauri::command]
pub fn add_personnel_role(
    state: State<'_, AppState>,
    input: PersonnelRoleInputDto,
) -> Result<BudgetSummaryDto, AppError> {
    let mut lock = state.project.lock().unwrap();
    let project = lock.as_mut().ok_or(AppError::NoProject)?;

    validate_personnel_role(
        &input,
        &project.personnel_roles,
        project.config.duration_years,
        None,
    )?;

    let role = PersonnelRole {
        id: Uuid::new_v4(),
        role_label: input.role_label,
        role_type: input.role_type,
        current_monthly_salary_try: input.current_monthly_salary_try,
        fte_fraction: input.fte_fraction,
        inflation_rate_pct: input.inflation_rate_pct,
        active_years: input.active_years,
        work_package_ids: input.work_package_ids,
    };
    project.personnel_roles.push(role);

    let summary = calculate_budget_summary(project, &state.rate_data)?;

    let path_lock = state.project_path.lock().unwrap();
    let _ = auto_save(project, path_lock.as_deref());

    Ok(summary)
}

/// Update an existing personnel role by UUID.
/// Returns the full recalculated BudgetSummaryDto.
#[tauri::command]
pub fn update_personnel_role(
    state: State<'_, AppState>,
    id: Uuid,
    input: PersonnelRoleInputDto,
) -> Result<BudgetSummaryDto, AppError> {
    let mut lock = state.project.lock().unwrap();
    let project = lock.as_mut().ok_or(AppError::NoProject)?;

    validate_personnel_role(
        &input,
        &project.personnel_roles,
        project.config.duration_years,
        Some(id),
    )?;

    let role = project
        .personnel_roles
        .iter_mut()
        .find(|r| r.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Personnel role {id} not found.")))?;

    role.role_label = input.role_label;
    role.role_type = input.role_type;
    role.current_monthly_salary_try = input.current_monthly_salary_try;
    role.fte_fraction = input.fte_fraction;
    role.inflation_rate_pct = input.inflation_rate_pct;
    role.active_years = input.active_years;
    role.work_package_ids = input.work_package_ids;

    let summary = calculate_budget_summary(project, &state.rate_data)?;

    let path_lock = state.project_path.lock().unwrap();
    let _ = auto_save(project, path_lock.as_deref());

    Ok(summary)
}

/// Delete a personnel role by UUID.
/// Returns the full recalculated BudgetSummaryDto.
#[tauri::command]
pub fn delete_personnel_role(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<BudgetSummaryDto, AppError> {
    let mut lock = state.project.lock().unwrap();
    let project = lock.as_mut().ok_or(AppError::NoProject)?;

    let before = project.personnel_roles.len();
    project.personnel_roles.retain(|r| r.id != id);
    if project.personnel_roles.len() == before {
        return Err(AppError::NotFound(format!("Personnel role {id} not found.")));
    }

    let summary = calculate_budget_summary(project, &state.rate_data)?;

    let path_lock = state.project_path.lock().unwrap();
    let _ = auto_save(project, path_lock.as_deref());

    Ok(summary)
}

/// Live cost preview for a role being edited — does NOT mutate project state.
/// Used to drive the live preview box in the Personnel form.
#[tauri::command]
pub fn preview_role_cost(
    state: State<'_, AppState>,
    input: PersonnelRoleInputDto,
) -> Result<RoleCostPreviewDto, AppError> {
    let lock = state.project.lock().unwrap();
    let project = lock.as_ref().ok_or(AppError::NoProject)?;

    let base_eur = convert_try_to_eur(
        input.current_monthly_salary_try,
        project.config.try_eur_rate,
    )?;

    let projections = project_salary_chain(
        base_eur,
        input.inflation_rate_pct,
        project.config.duration_years,
    )?;

    let cost_lines = calculate_personnel_cost_lines(
        &projections,
        input.fte_fraction,
        &input.active_years,
    )?;

    let total: rust_decimal::Decimal = cost_lines.iter().map(|l| l.annual_cost_eur).sum();

    let cost_line_dtos: Vec<RoleCostLineDto> = cost_lines
        .iter()
        .map(|l| RoleCostLineDto {
            year: l.year,
            is_active: l.is_active,
            monthly_salary_eur: l.monthly_salary_eur,
            annual_cost_eur: l.annual_cost_eur,
        })
        .collect();

    Ok(RoleCostPreviewDto {
        base_monthly_eur: base_eur,
        cost_lines: cost_line_dtos,
        total_cost_eur: total,
    })
}
