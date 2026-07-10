//! CALC-19 — Full Budget Summary Orchestration
//!
//! This is the master calculation function. It is called after every project mutation
//! and executes all CALC-01 through CALC-18 in the correct dependency order.
//!
//! Returns a complete `BudgetSummaryDto` that the frontend renders directly.
//! If any sub-calculation fails, execution stops and the error propagates immediately.
//! A partial BudgetSummaryDto is never returned.

use rust_decimal::Decimal;
use crate::domain::entities::{Project, TripType};
use crate::domain::dto::*;
use crate::domain::rate_data::RateData;
use crate::error::AppError;
use crate::calculation::{
    salary_projection::{convert_try_to_eur, project_salary_chain},
    personnel_cost::{calculate_personnel_cost_lines, aggregate_personnel_costs},
    equipment_depreciation::{calculate_depreciation, aggregate_equipment_costs},
    trip_cost::{
        calculate_itemized_trip_cost, calculate_flat_trip_cost,
        aggregate_travel_by_year, TripCostResult,
    },
    budget_aggregator::{
        aggregate_c3_costs, calculate_indirect_costs, calculate_total_direct_costs,
        calculate_total_eligible_costs, calculate_requested_contribution,
        YearCostEntry,
    },
    cfs_checker::check_cfs_threshold,
};

/// CALC-19: Orchestrate all calculations and assemble the BudgetSummaryDto.
///
/// Execution order (must not change — each step feeds the next):
/// 1. Personnel (CALC-01 → CALC-02 → CALC-03 per role → CALC-04)
/// 2. Equipment (CALC-05 per item → CALC-06)
/// 3. Travel (CALC-07/08/09/10 or CALC-11 per trip → CALC-12)
/// 4. C3 (CALC-13)
/// 5. Direct totals (CALC-15)
/// 6. Indirect costs (CALC-14)
/// 7. Eligible totals (CALC-16)
/// 8. EU contribution (CALC-17)
/// 9. CFS check (CALC-18)
/// 10. Assemble BudgetSummaryDto
pub fn calculate_budget_summary(
    project: &Project,
    rate_data: &RateData,
) -> Result<BudgetSummaryDto, AppError> {
    let duration = project.config.duration_years;
    let rate_version = rate_data
        .find_version(&project.config.rate_version_id)
        .ok_or_else(|| AppError::NotFound(format!(
            "Rate version '{}' not found.", project.config.rate_version_id
        )))?;

    // ── Step 1: Personnel ────────────────────────────────────────────────────

    let mut all_role_lines = Vec::new();
    let mut role_detail: Vec<PersonnelRoleDetailDto> = Vec::new();

    for role in &project.personnel_roles {
        let inflation_pct = role.inflation_rate_pct;
        let base_eur = convert_try_to_eur(
            role.current_monthly_salary_try,
            project.config.try_eur_rate,
        )?;
        let projections = project_salary_chain(base_eur, inflation_pct, duration)?;
        let cost_lines = calculate_personnel_cost_lines(
            &projections,
            role.fte_fraction,
            &role.active_years,
        )?;

        let total_role_cost: Decimal = cost_lines.iter().map(|l| l.annual_cost_eur).sum();

        let cost_line_dtos: Vec<RoleCostLineDto> = cost_lines
            .iter()
            .map(|l| RoleCostLineDto {
                year: l.year,
                is_active: l.is_active,
                monthly_salary_eur: l.monthly_salary_eur,
                annual_cost_eur: l.annual_cost_eur,
            })
            .collect();

        role_detail.push(PersonnelRoleDetailDto {
            id: role.id,
            role_label: role.role_label.clone(),
            role_type: role.role_type.clone(),
            fte_fraction: role.fte_fraction,
            cost_lines: cost_line_dtos,
            total_cost_eur: total_role_cost,
        });

        all_role_lines.push(cost_lines);
    }

    let personnel_totals = {
        let converted: Vec<Vec<_>> = all_role_lines;
        aggregate_personnel_costs(
            &converted.iter()
                .map(|lines| lines.iter().map(|l| crate::calculation::personnel_cost::PersonnelCostLine {
                    year: l.year,
                    is_active: l.is_active,
                    active_months: l.active_months,
                    monthly_salary_eur: l.monthly_salary_eur,
                    annual_cost_eur: l.annual_cost_eur,
                }).collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            duration,
        )?
    };

    let a_by_year: Vec<YearCostEntry> = personnel_totals.by_year
        .iter()
        .map(|y| YearCostEntry { year: y.year, amount_eur: y.amount_eur })
        .collect();
    let category_a_total = personnel_totals.total;

    // ── Step 2: Equipment ────────────────────────────────────────────────────

    let mut depreciation_results = Vec::new();
    let mut equipment_detail: Vec<EquipmentItemDetailDto> = Vec::new();

    for item in &project.equipment_items {
        let result = calculate_depreciation(
            item.purchase_cost_eur,
            item.useful_lifetime_months,
            item.grant_usage_pct,
            item.grant_usage_months,
        )?;
        equipment_detail.push(EquipmentItemDetailDto {
            id: item.id,
            name: item.name.clone(),
            theoretical_eligible_eur: result.theoretical_eligible_eur,
            maximum_eligible_eur: result.maximum_eligible_eur,
            is_capped: result.is_capped,
            eligible_depreciation_eur: result.eligible_depreciation_eur,
        });
        depreciation_results.push(result);
    }

    let category_c2_total = aggregate_equipment_costs(&depreciation_results)?;

    // ── Step 3: Travel ───────────────────────────────────────────────────────

    let mut trip_year_costs: Vec<(u8, Decimal)> = Vec::new();
    let mut trip_detail: Vec<TripDetailDto> = Vec::new();

    for trip in &project.trips {
        let trip_result: TripCostResult = match &trip.trip_type {
            TripType::Itemized {
                destination_country_code,
                one_way_distance_km,
                number_of_nights,
                number_of_days,
                domestic_transport_per_instance_eur,
            } => {
                let r = calculate_itemized_trip_cost(
                    destination_country_code,
                    *one_way_distance_km,
                    *number_of_nights,
                    *number_of_days,
                    *domestic_transport_per_instance_eur,
                    trip.number_of_instances,
                    rate_version,
                )?;
                let detail = TripDetailDto {
                    id: trip.id,
                    name: trip.name.clone(),
                    project_year: trip.project_year,
                    number_of_instances: trip.number_of_instances,
                    flight_cost_per_instance: Some(r.flight_cost_per_instance.to_string()),
                    accommodation_cost_per_instance: Some(r.accommodation_cost_per_instance.to_string()),
                    subsistence_cost_per_instance: Some(r.subsistence_cost_per_instance.to_string()),
                    domestic_transport_per_instance: Some(r.domestic_transport_per_instance.to_string()),
                    per_instance_total_eur: r.per_instance_total_eur,
                    total_trip_cost_eur: r.total_trip_cost_eur,
                };
                trip_detail.push(detail);
                TripCostResult::Itemized(r)
            }
            TripType::FlatAmount { flat_amount_per_instance_eur } => {
                let r = calculate_flat_trip_cost(
                    *flat_amount_per_instance_eur,
                    trip.number_of_instances,
                )?;
                let detail = TripDetailDto {
                    id: trip.id,
                    name: trip.name.clone(),
                    project_year: trip.project_year,
                    number_of_instances: trip.number_of_instances,
                    flight_cost_per_instance: None,
                    accommodation_cost_per_instance: None,
                    subsistence_cost_per_instance: None,
                    domestic_transport_per_instance: None,
                    per_instance_total_eur: r.flat_amount_per_instance,
                    total_trip_cost_eur: r.total_trip_cost_eur,
                };
                trip_detail.push(detail);
                TripCostResult::FlatAmount(r)
            }
        };
        trip_year_costs.push((trip.project_year, trip_result.total_cost()));
    }

    let travel_totals = aggregate_travel_by_year(&trip_year_costs, duration)?;
    let c1_by_year: Vec<YearCostEntry> = travel_totals.by_year
        .iter()
        .map(|y| YearCostEntry { year: y.year, amount_eur: y.amount_eur })
        .collect();
    let category_c1_total = travel_totals.total;

    // ── Step 4: C3 ───────────────────────────────────────────────────────────

    let c3_input: Vec<(u8, Decimal, bool)> = project.other_cost_items
        .iter()
        .map(|i| (i.project_year, i.amount_eur, i.is_cfs_item))
        .collect();
    let c3_result = aggregate_c3_costs(&c3_input, duration)?;
    let c3_by_year: Vec<YearCostEntry> = c3_result.by_year
        .iter()
        .map(|y| YearCostEntry { year: y.year, amount_eur: y.amount_eur })
        .collect();
    let category_c3_total = c3_result.total;

    let other_cost_detail: Vec<OtherCostItemDetailDto> = project.other_cost_items
        .iter()
        .map(|i| OtherCostItemDetailDto {
            id: i.id,
            name: i.name.clone(),
            amount_eur: i.amount_eur,
            project_year: i.project_year,
            is_cfs_item: i.is_cfs_item,
            notes: i.notes.clone(),
            work_package_id: i.work_package_id,
        })
        .collect();

    // ── Step 5: Direct Totals ────────────────────────────────────────────────

    let category_b_total = project.subcontracting.amount_eur;
    let total_direct = calculate_total_direct_costs(
        category_a_total, category_b_total, category_c1_total, category_c2_total, category_c3_total,
    )?;

    // ── Step 6: Indirect Costs ───────────────────────────────────────────────

    let indirect_result = calculate_indirect_costs(
        &a_by_year,
        &c1_by_year,
        category_c2_total,
        &c3_by_year,
        project.config.indirect_cost_rate_pct,
        duration,
    )?;

    // ── Step 7: Eligible Totals ──────────────────────────────────────────────

    let total_eligible = calculate_total_eligible_costs(total_direct, category_b_total, indirect_result.total)?;

    // ── Step 8: EU Contribution ──────────────────────────────────────────────

    let requested_contribution = calculate_requested_contribution(total_eligible)?;

    // ── Step 9: CFS Check ────────────────────────────────────────────────────

    let cfs_result = check_cfs_threshold(
        requested_contribution,
        project.has_cfs_item(),
        project.cfs_warning_dismissed,
    )?;

    // ── Step 10: Assemble DTO ────────────────────────────────────────────────

    let category_a_by_year: Vec<YearCostDto> = a_by_year
        .iter()
        .map(|y| YearCostDto { year: y.year, amount_eur: y.amount_eur })
        .collect();

    let category_c1_by_year: Vec<YearCostDto> = c1_by_year
        .iter()
        .map(|y| YearCostDto { year: y.year, amount_eur: y.amount_eur })
        .collect();

    let category_c3_by_year: Vec<YearCostDto> = c3_by_year
        .iter()
        .map(|y| YearCostDto { year: y.year, amount_eur: y.amount_eur })
        .collect();

    let category_e_by_year: Vec<YearCostDto> = indirect_result.by_year
        .iter()
        .map(|y| YearCostDto { year: y.year, amount_eur: y.amount_eur })
        .collect();

    Ok(BudgetSummaryDto {
        category_a_by_year,
        category_a_total,
        category_b_total,
        category_c1_by_year,
        category_c1_total,
        category_c2_total,
        category_c3_by_year,
        category_c3_total,
        indirect_base_total: indirect_result.base,
        category_e_by_year,
        category_e_total: indirect_result.total,
        total_direct_costs: total_direct,
        total_eligible_costs: total_eligible,
        requested_eu_contribution: requested_contribution,
        cfs_status: cfs_result.cfs_status,
        cfs_threshold_exceeded: cfs_result.threshold_exceeded,
        cfs_warning_active: cfs_result.warning_active,
        cfs_prompt_required: cfs_result.prompt_required,
        role_detail,
        equipment_detail,
        trip_detail,
        other_cost_detail,
    })
}
