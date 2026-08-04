//! IPC commands for opening/saving the shared `.ercbudget` file from the
//! Execution Application. Unlike the Budget App, this app never creates a
//! new project — `.ercbudget` files are always produced by the Budget App
//! first (see `docs/executer/execution-requirements.md` M-02).

use crate::domain::dto::{
    ActualCostEntryDetailDto, AmendmentDetailDto, DeliverableDetailDto,
    EquipmentProcurementDetailDto, ExecutionProjectSummaryDto, IssueEntryDetailDto,
    MilestoneDetailDto, PersonDetailDto, PersonMonthDetailDto, PersonnelRoleSummaryDto,
    PlannedEquipmentSummaryDto, PlannedOtherCostSummaryDto, PlannedTripSummaryDto, ProjectInfoDto,
    ReportingPeriodDetailDto, RiskEntryDetailDto, SubcontractingLineDetailDto,
    TripExecutionDetailDto, WorkPackageExecutionDetailDto,
};
use crate::domain::execution_entities::ExecutionData;
use crate::engines::notification_engine::WarningContext;
use crate::engines::{
    financial_engine, notification_engine, progress_engine, reporting_period_engine, risk_engine,
};
use crate::error::AppError;
use crate::persistence;
use crate::AppState;
use erc_core::calculation::calculate_budget_summary;
use erc_core::domain::entities::Project;
use rust_decimal::Decimal;
use tauri::State;

/// BR-WP-03's overspend tolerance multiplier (1.05 = 5%).
const WP_OVERSPEND_MULTIPLIER: Decimal = Decimal::from_parts(105, 0, 0, false, 2);
/// BR-TR-04's overspend tolerance multiplier (1.20 = 20%).
const TRIP_OVERSPEND_MULTIPLIER: Decimal = Decimal::from_parts(120, 0, 0, false, 2);
/// BR-EQ-02's overspend tolerance multiplier (1.10 = 10%).
const EQUIPMENT_OVERSPEND_MULTIPLIER: Decimal = Decimal::from_parts(110, 0, 0, false, 2);
/// BR-OC-02's overspend tolerance multiplier (1.10 = 10%).
const OTHER_COST_OVERSPEND_MULTIPLIER: Decimal = Decimal::from_parts(110, 0, 0, false, 2);
/// BR-SC-03's competitive-tendering advisory threshold (€200,000).
const SUBCONTRACTING_TENDER_THRESHOLD_EUR: Decimal = Decimal::from_parts(200_000, 0, 0, false, 0);

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
            let wp_deliverables: Vec<&crate::domain::execution_entities::Deliverable> = exec
                .deliverables
                .iter()
                .filter(|d| d.work_package_id == wp.id)
                .collect();
            let status = progress_engine::derive_wp_status(
                wp.start_month,
                wp.end_month,
                &wp_milestones,
                &wp_deliverables,
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
            linked_deliverable_ids: m.linked_deliverable_ids.clone(),
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

    let planned_trips: Vec<PlannedTripSummaryDto> = project
        .trips
        .iter()
        .map(|t| PlannedTripSummaryDto {
            id: t.id,
            name: t.name.clone(),
            number_of_instances: t.number_of_instances,
        })
        .collect();

    let planned_equipment: Vec<PlannedEquipmentSummaryDto> = project
        .equipment_items
        .iter()
        .map(|e| PlannedEquipmentSummaryDto {
            id: e.id,
            name: e.name.clone(),
            planned_cost_eur: e.purchase_cost_eur,
        })
        .collect();

    let planned_other_costs: Vec<PlannedOtherCostSummaryDto> = project
        .other_cost_items
        .iter()
        .map(|o| PlannedOtherCostSummaryDto {
            id: o.id,
            name: o.name.clone(),
            amount_eur: o.amount_eur,
        })
        .collect();

    let trip_executions: Vec<TripExecutionDetailDto> = exec
        .trip_executions
        .iter()
        .map(|te| {
            let trip = project.trips.iter().find(|t| t.id == te.trip_id);
            let planned_cost_per_instance = trip
                .and_then(|t| {
                    financial_engine::calculate_planned_trip_cost_per_instance(
                        t,
                        &state.rate_data,
                        &project.config.rate_version_id,
                    )
                    .ok()
                })
                .unwrap_or(Decimal::ZERO);
            let traveller_name = exec
                .persons
                .iter()
                .find(|p| p.id == te.traveller_person_id)
                .map(|p| p.full_name.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            TripExecutionDetailDto {
                id: te.id,
                trip_id: te.trip_id,
                trip_name: trip
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| "Unknown trip".to_string()),
                instance_number: te.instance_number,
                traveller_person_id: te.traveller_person_id,
                traveller_name,
                actual_travel_date: te.actual_travel_date.clone(),
                actual_cost_eur: te.actual_cost_eur,
                status: te.status,
                planned_cost_per_instance_eur: planned_cost_per_instance,
                overspend_warning: planned_cost_per_instance > Decimal::ZERO
                    && te.actual_cost_eur > planned_cost_per_instance * TRIP_OVERSPEND_MULTIPLIER,
            }
        })
        .collect();

    let equipment_procurements: Vec<EquipmentProcurementDetailDto> = exec
        .equipment_procurements
        .iter()
        .map(|ep| {
            let item = project
                .equipment_items
                .iter()
                .find(|i| i.id == ep.equipment_item_id);
            let actual_eligible_depreciation_eur = if ep.delivery_confirmed {
                item.and_then(|i| {
                    erc_core::calculation::equipment_depreciation::calculate_depreciation(
                        ep.actual_purchase_cost_eur,
                        i.useful_lifetime_months,
                        i.grant_usage_pct,
                        i.grant_usage_months,
                    )
                    .ok()
                })
                .map(|r| r.eligible_depreciation_eur)
            } else {
                None
            };
            let overspend_warning = item.is_some_and(|i| {
                ep.actual_purchase_cost_eur > i.purchase_cost_eur * EQUIPMENT_OVERSPEND_MULTIPLIER
            });
            EquipmentProcurementDetailDto {
                id: ep.id,
                equipment_item_id: ep.equipment_item_id,
                equipment_item_name: item
                    .map(|i| i.name.clone())
                    .unwrap_or_else(|| "Unknown item".to_string()),
                actual_purchase_cost_eur: ep.actual_purchase_cost_eur,
                purchase_date: ep.purchase_date.clone(),
                delivery_confirmed: ep.delivery_confirmed,
                actual_eligible_depreciation_eur,
                overspend_warning,
            }
        })
        .collect();

    let actual_cost_entries: Vec<ActualCostEntryDetailDto> = exec
        .actual_cost_entries
        .iter()
        .map(|entry| {
            let linked_item = entry
                .linked_entity_id
                .and_then(|id| project.other_cost_items.iter().find(|i| i.id == id));
            let overspend_warning = linked_item.is_some_and(|item| {
                let actual_total =
                    financial_engine::calculate_other_cost_actual_total(item.id, exec);
                actual_total > item.amount_eur * OTHER_COST_OVERSPEND_MULTIPLIER
            });
            ActualCostEntryDetailDto {
                id: entry.id,
                linked_entity_id: entry.linked_entity_id,
                linked_entity_name: linked_item.map(|i| i.name.clone()),
                amount_eur: entry.amount_eur,
                description: entry.description.clone(),
                incurred_date: entry.incurred_date.clone(),
                status: entry.status,
                justification: entry.justification.clone(),
                overspend_warning,
            }
        })
        .collect();

    let subcontracting_lines: Vec<SubcontractingLineDetailDto> = exec
        .subcontracting_lines
        .iter()
        .map(|line| SubcontractingLineDetailDto {
            id: line.id,
            vendor: line.vendor.clone(),
            contract_reference: line.contract_reference.clone(),
            amount_eur: line.amount_eur,
            work_package_id: line.work_package_id,
            status: line.status,
            vendor_is_host_institution: line.vendor_is_host_institution,
            payment_date: line.payment_date.clone(),
            competitive_tender_warning: line.amount_eur > SUBCONTRACTING_TENDER_THRESHOLD_EUR,
            host_institution_warning: line.vendor_is_host_institution,
        })
        .collect();

    let actuals = financial_engine::calculate_actuals(project, exec, &planned)?;

    let deliverables: Vec<DeliverableDetailDto> = exec
        .deliverables
        .iter()
        .map(|d| {
            let responsible_role_label = project
                .personnel_roles
                .iter()
                .find(|r| r.id == d.responsible_role_id)
                .map(|r| r.role_label.clone())
                .unwrap_or_else(|| "Unknown role".to_string());
            let effective_planned_month = d.revised_planned_month.unwrap_or(d.planned_month);
            DeliverableDetailDto {
                id: d.id,
                deliverable_number: d.deliverable_number.clone(),
                title: d.title.clone(),
                deliverable_type: d.deliverable_type,
                work_package_id: d.work_package_id,
                planned_month: d.planned_month,
                responsible_role_id: d.responsible_role_id,
                responsible_role_label,
                dissemination_level: d.dissemination_level,
                status: d.status,
                actual_submission_date: d.actual_submission_date.clone(),
                revision_note: d.revision_note.clone(),
                revised_planned_month: d.revised_planned_month,
                cordis_registered: d.cordis_registered,
                notes: d.notes.clone(),
                is_overdue: progress_engine::is_deliverable_overdue(d, current_project_month),
                cordis_warning: d.dissemination_level
                    == crate::domain::enums::DisseminationLevel::Public
                    && !d.cordis_registered,
                reporting_period_number: reporting_period_engine::find_period_for_month(
                    &exec.reporting_periods,
                    effective_planned_month,
                ),
            }
        })
        .collect();

    let reporting_periods: Vec<ReportingPeriodDetailDto> = exec
        .reporting_periods
        .iter()
        .map(|p| {
            let deliverables_in_period: Vec<&crate::domain::execution_entities::Deliverable> = exec
                .deliverables
                .iter()
                .filter(|d| {
                    let effective_month = d.revised_planned_month.unwrap_or(d.planned_month);
                    effective_month >= p.start_month && effective_month <= p.end_month
                })
                .collect();
            let deliverables_submitted = deliverables_in_period
                .iter()
                .filter(|d| d.actual_submission_date.is_some())
                .count() as u32;
            ReportingPeriodDetailDto {
                id: p.id,
                period_number: p.period_number,
                start_month: p.start_month,
                end_month: p.end_month,
                submission_deadline: p.submission_deadline.clone(),
                technical_report_submitted: p.technical_report_submitted,
                financial_report_submitted: p.financial_report_submitted,
                status: p.status,
                deliverables_due: deliverables_in_period.len() as u32,
                deliverables_submitted,
            }
        })
        .collect();

    let max_month = project.config.duration_years as u32 * 12;
    let reporting_period_coverage =
        reporting_period_engine::compute_coverage(&exec.reporting_periods, max_month);

    let role_label = |role_id: Option<uuid::Uuid>| {
        role_id.and_then(|id| {
            project
                .personnel_roles
                .iter()
                .find(|r| r.id == id)
                .map(|r| r.role_label.clone())
        })
    };

    let risks: Vec<RiskEntryDetailDto> = exec
        .risks
        .iter()
        .map(|r| {
            let risk_score = risk_engine::risk_score(r.probability, r.impact);
            RiskEntryDetailDto {
                id: r.id,
                title: r.title.clone(),
                description: r.description.clone(),
                work_package_id: r.work_package_id,
                probability: r.probability,
                impact: r.impact,
                mitigation: r.mitigation.clone(),
                status: r.status,
                owner_role_id: r.owner_role_id,
                owner_role_label: role_label(r.owner_role_id),
                identified_date: r.identified_date.clone(),
                review_date: r.review_date.clone(),
                closed_date: r.closed_date.clone(),
                risk_score,
                priority: risk_engine::derive_risk_priority(risk_score),
            }
        })
        .collect();

    let today = chrono::Utc::now().date_naive();
    let issues: Vec<IssueEntryDetailDto> = exec
        .issues
        .iter()
        .map(|i| IssueEntryDetailDto {
            id: i.id,
            description: i.description.clone(),
            work_package_id: i.work_package_id,
            raised_date: i.raised_date.clone(),
            priority: i.priority,
            owner_role_id: i.owner_role_id,
            owner_role_label: role_label(i.owner_role_id),
            status: i.status,
            resolution: i.resolution.clone(),
            linked_risk_id: i.linked_risk_id,
            is_stale_warning: risk_engine::is_issue_stale_high_priority(i, today),
        })
        .collect();

    let warnings = notification_engine::evaluate_warnings(&WarningContext {
        project,
        exec,
        actuals: &actuals,
        deliverables: &deliverables,
        milestones: &milestones,
        work_packages: &work_packages,
        reporting_periods: &reporting_periods,
        risks: &risks,
        issues: &issues,
        trip_executions: &trip_executions,
        current_project_month,
        today,
    });

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
        planned_trips,
        planned_equipment,
        planned_other_costs,
        persons,
        person_months,
        work_packages,
        milestones,
        amendments,
        actuals,
        trip_executions,
        equipment_procurements,
        actual_cost_entries,
        subcontracting_lines,
        deliverables,
        reporting_periods,
        reporting_period_coverage,
        risks,
        issues,
        warnings,
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
    let (project, mut exec) = persistence::load_execution(&file_path)?;

    // BR-RP-05: pre-populate default reporting periods the first time this
    // project is opened in the Execution App.
    if exec.reporting_periods.is_empty() {
        let max_month = project.config.duration_years as u32 * 12;
        exec.reporting_periods =
            reporting_period_engine::generate_default_reporting_periods(max_month);
    }

    let summary = build_summary(&project, &exec, &state)?;
    persistence::auto_save(&project, &exec, &file_path)?;

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
