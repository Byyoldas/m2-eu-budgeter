//! IPC commands for opening/saving the shared `.ercbudget` file from the
//! Execution Application. Unlike the Budget App, this app never creates a
//! new project — `.ercbudget` files are always produced by the Budget App
//! first (see `docs/executer/execution-requirements.md` M-02).

use crate::domain::dto::{
    AmendmentDetailDto, ExecutionProjectSummaryDto, MilestoneDetailDto, PersonDetailDto,
    PersonMonthDetailDto, PersonnelRoleSummaryDto, ProjectInfoDto, WorkPackageExecutionDetailDto,
};
use crate::domain::execution_entities::ExecutionData;
use crate::engines::progress_engine;
use crate::error::AppError;
use crate::persistence;
use crate::AppState;
use erc_core::calculation::calculate_budget_summary;
use erc_core::domain::entities::Project;
use rust_decimal::Decimal;
use tauri::State;

/// BR-WP-03's overspend tolerance multiplier (1.05 = 5%).
const WP_OVERSPEND_MULTIPLIER: Decimal = Decimal::from_parts(105, 0, 0, false, 2);

pub(crate) fn build_summary(
    project: &Project,
    exec: &ExecutionData,
    state: &AppState,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let planned = calculate_budget_summary(project, &state.rate_data)?;
    let current_project_month = progress_engine::derive_current_project_month(project);

    let personnel_roles: Vec<PersonnelRoleSummaryDto> = project
        .personnel_roles
        .iter()
        .map(|r| PersonnelRoleSummaryDto {
            id: r.id,
            role_label: r.role_label.clone(),
            role_type: r.role_type.clone(),
        })
        .collect();

    let persons: Vec<PersonDetailDto> = exec
        .persons
        .iter()
        .map(|p| {
            let role_label = project
                .personnel_roles
                .iter()
                .find(|r| r.id == p.linked_role_id)
                .map(|r| r.role_label.clone())
                .unwrap_or_else(|| "Unknown role".to_string());
            PersonDetailDto {
                id: p.id,
                full_name: p.full_name.clone(),
                email: p.email.clone(),
                institution: p.institution.clone(),
                orcid: p.orcid.clone(),
                linked_role_id: p.linked_role_id,
                linked_role_label: role_label,
                actual_start_date: p.actual_start_date.clone(),
                actual_end_date: p.actual_end_date.clone(),
            }
        })
        .collect();

    let person_months: Vec<PersonMonthDetailDto> = exec
        .person_month_records
        .iter()
        .map(|record| {
            let estimate = exec
                .persons
                .iter()
                .find(|p| p.id == record.person_id)
                .and_then(|person| {
                    project
                        .personnel_roles
                        .iter()
                        .find(|r| r.id == person.linked_role_id)
                })
                .and_then(|role| {
                    progress_engine::calculate_person_month_salary_estimate(record, role, project)
                        .ok()
                        .flatten()
                });
            PersonMonthDetailDto {
                id: record.id,
                person_id: record.person_id,
                project_month: record.project_month,
                reported_months: record.reported_months,
                approved_months: record.approved_months,
                salary_cost_estimate_eur: estimate,
            }
        })
        .collect();

    let wp_actual_personnel = progress_engine::calculate_wp_actual_personnel_eur(
        project,
        &exec.person_month_records,
        &exec.persons,
    )?;

    let work_packages: Vec<WorkPackageExecutionDetailDto> = project
        .config
        .work_packages()
        .into_iter()
        .map(|wp| {
            let override_data = exec
                .work_package_executions
                .iter()
                .find(|w| w.work_package_id == wp.id);
            let leader_role_id = override_data.and_then(|w| w.leader_role_id);
            let leader_role_label = leader_role_id.and_then(|id| {
                project
                    .personnel_roles
                    .iter()
                    .find(|r| r.id == id)
                    .map(|r| r.role_label.clone())
            });

            let wp_milestones: Vec<&crate::domain::execution_entities::Milestone> = exec
                .milestones
                .iter()
                .filter(|m| m.work_package_id == wp.id)
                .collect();
            let status = progress_engine::derive_wp_status(
                wp.start_month,
                wp.end_month,
                &wp_milestones,
                current_project_month,
            );

            let planned_eur = planned
                .wp_budgets
                .iter()
                .find(|b| b.work_package_id == wp.id)
                .map(|b| b.total_eur)
                .unwrap_or(Decimal::ZERO);
            let actual_eur = wp_actual_personnel
                .get(&wp.id)
                .copied()
                .unwrap_or(Decimal::ZERO);
            let overspend_warning =
                planned_eur > Decimal::ZERO && actual_eur > planned_eur * WP_OVERSPEND_MULTIPLIER;

            WorkPackageExecutionDetailDto {
                work_package_id: wp.id,
                work_package_name: wp.name,
                leader_role_id,
                leader_role_label,
                notes: override_data.and_then(|w| w.notes.clone()),
                status,
                planned_eur,
                actual_eur,
                overspend_warning,
            }
        })
        .collect();

    let milestones: Vec<MilestoneDetailDto> = exec
        .milestones
        .iter()
        .map(|m| MilestoneDetailDto {
            id: m.id,
            title: m.title.clone(),
            work_package_id: m.work_package_id,
            planned_month: m.planned_month,
            status: m.status,
            effective_status: progress_engine::derive_milestone_status(m, current_project_month),
            actual_completion_month: m.actual_completion_month,
        })
        .collect();

    let amendments: Vec<AmendmentDetailDto> = exec
        .amendments
        .iter()
        .map(|a| AmendmentDetailDto {
            id: a.id,
            amendment_number: a.amendment_number.clone(),
            amendment_type: a.amendment_type,
            title: a.title.clone(),
            description: a.description.clone(),
            requested_date: a.requested_date.clone(),
            decision_date: a.decision_date.clone(),
            status: a.status,
            financial_impact_eur: a.financial_impact_eur,
            affected_work_package_ids: a.affected_work_package_ids.clone(),
            notes: a.notes.clone(),
        })
        .collect();

    Ok(ExecutionProjectSummaryDto {
        project_info: ProjectInfoDto {
            project_title: project.config.project_title.clone(),
            pi_name: project.config.pi_name.clone(),
            call_reference: project.config.call_reference.clone(),
            duration_years: project.config.duration_years,
            work_package_count: project.config.work_package_count,
        },
        planned,
        current_project_month,
        personnel_roles,
        persons,
        person_months,
        work_packages,
        milestones,
        amendments,
    })
}

/// Open a `.ercbudget` file produced by the Budget App and load it into the
/// Execution App's state, creating a fresh `execution_data` block if the
/// file doesn't have one yet (BR-IO-03).
#[tauri::command]
pub fn open_execution_project(
    state: State<'_, AppState>,
    path: String,
) -> Result<ExecutionProjectSummaryDto, AppError> {
    let file_path = std::path::PathBuf::from(&path);
    let (project, exec) = persistence::load_execution(&file_path)?;
    let summary = build_summary(&project, &exec, &state)?;

    *state.project.lock().unwrap() = Some(project);
    *state.execution_data.lock().unwrap() = Some(exec);
    *state.project_path.lock().unwrap() = Some(file_path);

    Ok(summary)
}

/// Save the current project + execution data back to its known file path.
#[tauri::command]
pub fn save_execution_project(state: State<'_, AppState>) -> Result<(), AppError> {
    let project_lock = state.project.lock().unwrap();
    let project = project_lock.as_ref().ok_or(AppError::NoProject)?;

    let exec_lock = state.execution_data.lock().unwrap();
    let exec = exec_lock.as_ref().ok_or(AppError::NoProject)?;

    let path_lock = state.project_path.lock().unwrap();
    let path = path_lock
        .as_ref()
        .ok_or_else(|| AppError::Persistence("No file path set.".to_string()))?;

    persistence::save_execution(project, exec, path)
}
