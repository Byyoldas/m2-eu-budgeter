//! Financial Engine — M-07 Financial Reporting (Planned vs. Actual).
//!
//! Reuses erc-core's aggregation functions (`calculate_indirect_costs`,
//! `calculate_total_direct_costs`, `calculate_total_eligible_costs`,
//! `calculate_requested_contribution`, `check_cfs_threshold`,
//! `calculate_depreciation`, `calculate_itemized_trip_cost`/
//! `calculate_flat_trip_cost`) with actual instead of planned category
//! totals — the same pure functions the Budget App uses for planned figures,
//! per `docs/executer/execution-architecture.md` §7.2.

use crate::domain::dto::ActualFinancialsDto;
use crate::domain::enums::EntryStatus;
use crate::domain::execution_entities::ExecutionData;
use crate::engines::progress_engine::calculate_person_month_salary_estimate;
use erc_core::calculation::budget_aggregator::{
    calculate_indirect_costs, calculate_requested_contribution, calculate_total_direct_costs,
    calculate_total_eligible_costs,
};
use erc_core::calculation::cfs_checker::check_cfs_threshold;
use erc_core::calculation::equipment_depreciation::calculate_depreciation;
use erc_core::calculation::trip_cost::{calculate_flat_trip_cost, calculate_itemized_trip_cost};
use erc_core::domain::dto::BudgetSummaryDto;
use erc_core::domain::entities::{Project, Trip, TripType};
use erc_core::domain::rate_data::RateData;
use erc_core::error::AppError;
use rust_decimal::Decimal;
use uuid::Uuid;

/// BR-FIN-01: sum of (approved person-months × inflation-adjusted monthly salary).
pub fn calculate_category_a_actual(
    project: &Project,
    exec: &ExecutionData,
) -> Result<Decimal, AppError> {
    let mut total = Decimal::ZERO;
    for record in &exec.person_month_records {
        let Some(person) = exec.persons.iter().find(|p| p.id == record.person_id) else {
            continue;
        };
        let Some(role) = project
            .personnel_roles
            .iter()
            .find(|r| r.id == person.linked_role_id)
        else {
            continue;
        };
        if let Some(estimate) = calculate_person_month_salary_estimate(record, role, project)? {
            total += estimate;
        }
    }
    Ok(total)
}

/// BR-FIN-02: sum of approved subcontracting line amounts.
pub fn calculate_category_b_actual(exec: &ExecutionData) -> Decimal {
    exec.subcontracting_lines
        .iter()
        .filter(|l| l.status == EntryStatus::Approved)
        .map(|l| l.amount_eur)
        .sum()
}

/// BR-TR-05: sum of approved trip execution actual costs.
pub fn calculate_category_c1_actual(exec: &ExecutionData) -> Decimal {
    exec.trip_executions
        .iter()
        .filter(|t| t.status == EntryStatus::Approved)
        .map(|t| t.actual_cost_eur)
        .sum()
}

/// BR-EQ-01/04: sum of eligible depreciation (CALC-05, actual purchase cost)
/// for procurements with confirmed delivery.
pub fn calculate_category_c2_actual(
    project: &Project,
    exec: &ExecutionData,
) -> Result<Decimal, AppError> {
    let mut total = Decimal::ZERO;
    for procurement in exec
        .equipment_procurements
        .iter()
        .filter(|p| p.delivery_confirmed)
    {
        let Some(item) = project
            .equipment_items
            .iter()
            .find(|i| i.id == procurement.equipment_item_id)
        else {
            continue;
        };
        let result = calculate_depreciation(
            procurement.actual_purchase_cost_eur,
            item.useful_lifetime_months,
            item.grant_usage_pct,
            item.grant_usage_months,
        )?;
        total += result.eligible_depreciation_eur;
    }
    Ok(total)
}

/// BR-FIN-02: sum of approved Category C3 actual cost entries.
pub fn calculate_category_c3_actual(exec: &ExecutionData) -> Decimal {
    exec.actual_cost_entries
        .iter()
        .filter(|e| e.status == EntryStatus::Approved)
        .map(|e| e.amount_eur)
        .sum()
}

/// The planned per-instance cost of a trip, for BR-TR-04's overspend check.
/// Reuses CALC-10/11 exactly as the Budget App does for the planned total,
/// with `number_of_instances = 1`.
pub fn calculate_planned_trip_cost_per_instance(
    trip: &Trip,
    rate_data: &RateData,
    rate_version_id: &str,
) -> Result<Decimal, AppError> {
    match &trip.trip_type {
        TripType::FlatAmount {
            flat_amount_per_instance_eur,
        } => Ok(
            calculate_flat_trip_cost(*flat_amount_per_instance_eur, 1)?.flat_amount_per_instance
        ),
        TripType::Itemized {
            destination_country_code,
            one_way_distance_km,
            number_of_nights,
            number_of_days,
            domestic_transport_per_instance_eur,
        } => {
            let rate_version = rate_data.find_version(rate_version_id).ok_or_else(|| {
                AppError::NotFound(format!("Rate version '{rate_version_id}' not found."))
            })?;
            let result = calculate_itemized_trip_cost(
                destination_country_code,
                *one_way_distance_km,
                *number_of_nights,
                *number_of_days,
                *domestic_transport_per_instance_eur,
                1,
                rate_version,
            )?;
            Ok(result.per_instance_total_eur)
        }
    }
}

/// BR-OC-02's "planned item's actual total" — sum of approved
/// `ActualCostEntry` amounts linked to one `OtherDirectCostItem`.
pub fn calculate_other_cost_actual_total(item_id: Uuid, exec: &ExecutionData) -> Decimal {
    exec.actual_cost_entries
        .iter()
        .filter(|e| e.linked_entity_id == Some(item_id) && e.status == EntryStatus::Approved)
        .map(|e| e.amount_eur)
        .sum()
}

/// Master orchestrator: assembles `ActualFinancialsDto`, reusing the same
/// erc-core aggregation chain the Budget App uses for planned figures.
pub fn calculate_actuals(
    project: &Project,
    exec: &ExecutionData,
    planned: &BudgetSummaryDto,
) -> Result<ActualFinancialsDto, AppError> {
    let a_actual = calculate_category_a_actual(project, exec)?;
    let b_actual = calculate_category_b_actual(exec);
    let c1_actual = calculate_category_c1_actual(exec);
    let c2_actual = calculate_category_c2_actual(project, exec)?;
    let c3_actual = calculate_category_c3_actual(exec);

    let indirect = calculate_indirect_costs(
        a_actual,
        c1_actual,
        c2_actual,
        c3_actual,
        project.config.indirect_cost_rate_pct,
    )?;
    let total_direct_actual =
        calculate_total_direct_costs(a_actual, b_actual, c1_actual, c2_actual, c3_actual)?;
    let total_eligible_actual =
        calculate_total_eligible_costs(total_direct_actual, indirect.total)?;
    let requested_eu_contribution_actual = calculate_requested_contribution(total_eligible_actual)?;

    // BR-FIN-06: re-evaluate CFS status against the running actual total.
    let cfs_result = check_cfs_threshold(
        requested_eu_contribution_actual,
        project.has_cfs_item(),
        project.cfs_warning_dismissed,
    )?;

    // BR-FIN-04: actual exceeds planned by more than 15%.
    let overrun = |actual: Decimal, planned_total: Decimal| {
        planned_total > Decimal::ZERO && actual > planned_total * Decimal::new(115, 2)
    };

    Ok(ActualFinancialsDto {
        a_actual,
        b_actual,
        c1_actual,
        c2_actual,
        c3_actual,
        e_actual: indirect.total,
        total_direct_actual,
        total_eligible_actual,
        requested_eu_contribution_actual,
        cfs_status_actual: cfs_result.cfs_status,
        category_a_overrun: overrun(a_actual, planned.category_a_total),
        category_b_overrun: overrun(b_actual, planned.category_b_total),
        category_c1_overrun: overrun(c1_actual, planned.category_c1_total),
        category_c2_overrun: overrun(c2_actual, planned.category_c2_total),
        category_c3_overrun: overrun(c3_actual, planned.category_c3_total),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::execution_entities::{
        ActualCostEntry, EquipmentProcurement, Person, PersonMonthRecord, SubcontractingLine,
        TripExecution,
    };
    use erc_core::domain::entities::{EquipmentItem, PersonnelRole, ProjectConfig, RoleType};
    use rust_decimal_macros::dec;

    fn make_project() -> Project {
        let config = ProjectConfig {
            project_title: "Test".to_string(),
            pi_name: "PI".to_string(),
            call_reference: "ERC-2025-CoG".to_string(),
            duration_years: 2,
            work_package_count: 1,
            work_package_names: vec![None],
            work_package_start_months: vec![1],
            work_package_end_months: vec![24],
            default_inflation_rate_pct: dec!(0),
            try_eur_rate: dec!(50),
            indirect_cost_rate_pct: dec!(25),
            rate_version_id: "from_2025_05_13".to_string(),
            call_opening_date: None,
        };
        Project::new(config)
    }

    fn make_role(id: Uuid) -> PersonnelRole {
        PersonnelRole {
            id,
            role_label: "PostDoc-1".to_string(),
            role_type: RoleType::PostDoc,
            current_monthly_salary_try: dec!(50000),
            fte_fraction: dec!(1),
            inflation_rate_pct: dec!(0),
            start_month: 1,
            end_month: 12,
        }
    }

    #[test]
    fn test_category_a_actual_sums_approved_records_only() {
        let mut project = make_project();
        let role_id = Uuid::new_v4();
        project.personnel_roles.push(make_role(role_id));

        let person_id = Uuid::new_v4();
        let mut exec = ExecutionData::default();
        exec.persons.push(Person {
            id: person_id,
            full_name: "Ada".to_string(),
            email: None,
            institution: None,
            orcid: None,
            linked_role_id: role_id,
            actual_start_date: "2026-01-01".to_string(),
            actual_end_date: None,
        });
        exec.person_month_records.push(PersonMonthRecord {
            id: Uuid::new_v4(),
            person_id,
            project_month: 1,
            reported_months: dec!(1),
            approved_months: Some(dec!(1)),
        });
        exec.person_month_records.push(PersonMonthRecord {
            id: Uuid::new_v4(),
            person_id,
            project_month: 2,
            reported_months: dec!(1),
            approved_months: None, // unapproved — excluded
        });

        // 50000 TRY / 50 = 1000 EUR base, 0% inflation.
        assert_eq!(
            calculate_category_a_actual(&project, &exec).unwrap(),
            dec!(1000)
        );
    }

    #[test]
    fn test_category_b_actual_sums_approved_lines_only() {
        let mut exec = ExecutionData::default();
        exec.subcontracting_lines.push(SubcontractingLine {
            id: Uuid::new_v4(),
            vendor: "Vendor A".to_string(),
            contract_reference: "CTR-1".to_string(),
            amount_eur: dec!(1000),
            work_package_id: 1,
            status: EntryStatus::Approved,
            vendor_is_host_institution: false,
            payment_date: None,
        });
        exec.subcontracting_lines.push(SubcontractingLine {
            id: Uuid::new_v4(),
            vendor: "Vendor B".to_string(),
            contract_reference: "CTR-2".to_string(),
            amount_eur: dec!(500),
            work_package_id: 1,
            status: EntryStatus::Pending,
            vendor_is_host_institution: false,
            payment_date: None,
        });
        assert_eq!(calculate_category_b_actual(&exec), dec!(1000));
    }

    #[test]
    fn test_category_c1_actual_sums_approved_trips_only() {
        let mut exec = ExecutionData::default();
        exec.trip_executions.push(TripExecution {
            id: Uuid::new_v4(),
            trip_id: Uuid::new_v4(),
            instance_number: 1,
            traveller_person_id: Uuid::new_v4(),
            actual_travel_date: "2026-03-01".to_string(),
            actual_cost_eur: dec!(600),
            status: EntryStatus::Approved,
        });
        exec.trip_executions.push(TripExecution {
            id: Uuid::new_v4(),
            trip_id: Uuid::new_v4(),
            instance_number: 1,
            traveller_person_id: Uuid::new_v4(),
            actual_travel_date: "2026-03-01".to_string(),
            actual_cost_eur: dec!(400),
            status: EntryStatus::Rejected,
        });
        assert_eq!(calculate_category_c1_actual(&exec), dec!(600));
    }

    #[test]
    fn test_category_c2_actual_excludes_unconfirmed_delivery() {
        let mut project = make_project();
        let item_id = Uuid::new_v4();
        project.equipment_items.push(EquipmentItem {
            id: item_id,
            name: "Laptop".to_string(),
            purchase_cost_eur: dec!(2000),
            // 2000 / 25 = 80 exactly, avoiding a repeating decimal from the
            // CALC-05 division so the capped/theoretical values compare equal.
            useful_lifetime_months: 25,
            grant_usage_pct: dec!(100),
            grant_usage_months: 25,
            work_package_id: 1,
        });

        let mut exec = ExecutionData::default();
        exec.equipment_procurements.push(EquipmentProcurement {
            id: Uuid::new_v4(),
            equipment_item_id: item_id,
            actual_purchase_cost_eur: dec!(2000),
            purchase_date: "2026-01-01".to_string(),
            delivery_confirmed: true,
        });
        exec.equipment_procurements.push(EquipmentProcurement {
            id: Uuid::new_v4(),
            equipment_item_id: item_id,
            actual_purchase_cost_eur: dec!(1000),
            purchase_date: "2026-01-01".to_string(),
            delivery_confirmed: false, // excluded — BR-EQ-04
        });

        // (2000/25) * 1.0 * 25 = 2000, capped at 2000*1.0 = 2000.
        assert_eq!(
            calculate_category_c2_actual(&project, &exec).unwrap(),
            dec!(2000)
        );
    }

    #[test]
    fn test_category_c3_actual_sums_approved_entries_only() {
        let mut exec = ExecutionData::default();
        exec.actual_cost_entries.push(ActualCostEntry {
            id: Uuid::new_v4(),
            linked_entity_id: None,
            amount_eur: dec!(300),
            description: "Fees".to_string(),
            incurred_date: "2026-01-01".to_string(),
            status: EntryStatus::Approved,
            justification: Some("Unplanned".to_string()),
        });
        assert_eq!(calculate_category_c3_actual(&exec), dec!(300));
    }

    #[test]
    fn test_other_cost_actual_total_filters_by_item_and_status() {
        let item_id = Uuid::new_v4();
        let mut exec = ExecutionData::default();
        exec.actual_cost_entries.push(ActualCostEntry {
            id: Uuid::new_v4(),
            linked_entity_id: Some(item_id),
            amount_eur: dec!(500),
            description: "Part 1".to_string(),
            incurred_date: "2026-01-01".to_string(),
            status: EntryStatus::Approved,
            justification: None,
        });
        exec.actual_cost_entries.push(ActualCostEntry {
            id: Uuid::new_v4(),
            linked_entity_id: Some(item_id),
            amount_eur: dec!(500),
            description: "Part 2 (pending)".to_string(),
            incurred_date: "2026-01-01".to_string(),
            status: EntryStatus::Pending,
            justification: None,
        });
        exec.actual_cost_entries.push(ActualCostEntry {
            id: Uuid::new_v4(),
            linked_entity_id: Some(Uuid::new_v4()), // different item
            amount_eur: dec!(999),
            description: "Other item".to_string(),
            incurred_date: "2026-01-01".to_string(),
            status: EntryStatus::Approved,
            justification: None,
        });
        assert_eq!(calculate_other_cost_actual_total(item_id, &exec), dec!(500));
    }

    #[test]
    fn test_calculate_actuals_assembles_full_dto() {
        let mut project = make_project();
        let role_id = Uuid::new_v4();
        project.personnel_roles.push(make_role(role_id));

        let person_id = Uuid::new_v4();
        let mut exec = ExecutionData::default();
        exec.persons.push(Person {
            id: person_id,
            full_name: "Ada".to_string(),
            email: None,
            institution: None,
            orcid: None,
            linked_role_id: role_id,
            actual_start_date: "2026-01-01".to_string(),
            actual_end_date: None,
        });
        exec.person_month_records.push(PersonMonthRecord {
            id: Uuid::new_v4(),
            person_id,
            project_month: 1,
            reported_months: dec!(1),
            approved_months: Some(dec!(1)),
        });

        let rate_data = erc_core::domain::rate_data::RateData::load_embedded().unwrap();
        let planned =
            erc_core::calculation::calculate_budget_summary(&project, &rate_data).unwrap();

        let actuals = calculate_actuals(&project, &exec, &planned).unwrap();
        assert_eq!(actuals.a_actual, dec!(1000));
        assert_eq!(actuals.b_actual, Decimal::ZERO);
        // e_actual = (1000 + 0 + 0 + 0) * 25% = 250.
        assert_eq!(actuals.e_actual, dec!(250));
        assert_eq!(actuals.total_direct_actual, dec!(1000));
        assert_eq!(actuals.total_eligible_actual, dec!(1250));
        assert_eq!(actuals.requested_eu_contribution_actual, dec!(1250));
    }
}
