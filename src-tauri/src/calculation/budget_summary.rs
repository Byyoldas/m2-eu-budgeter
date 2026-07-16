//! CALC-19 — Full Budget Summary Orchestration
//!
//! This is the master calculation function. It is called after every project mutation
//! and executes all CALC-01 through CALC-20 in the correct dependency order.
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
    personnel_cost::{calculate_personnel_cost_lines, aggregate_personnel_costs, allocate_personnel_cost_by_wp, WpCostAmount},
    equipment_depreciation::{calculate_depreciation, aggregate_equipment_costs},
    trip_cost::{
        calculate_itemized_trip_cost, calculate_flat_trip_cost,
        aggregate_travel_costs, TripCostResult,
    },
    budget_aggregator::{
        aggregate_c3_costs, calculate_indirect_costs, calculate_total_direct_costs,
        calculate_total_eligible_costs, calculate_requested_contribution,
    },
    wp_budget::aggregate_wp_budgets,
    cfs_checker::check_cfs_threshold,
};

/// CALC-19: Orchestrate all calculations and assemble the BudgetSummaryDto.
///
/// Execution order (must not change — each step feeds the next):
/// 1. Personnel (CALC-01 → CALC-02 → CALC-03 per role → CALC-04, plus CALC-20a WP allocation)
/// 2. Equipment (CALC-05 per item → CALC-06)
/// 3. Travel (CALC-07/08/09/10 or CALC-11 per trip → CALC-12)
/// 4. C3 (CALC-13)
/// 5. Direct totals (CALC-15)
/// 6. Indirect costs (CALC-14)
/// 7. Eligible totals (CALC-16)
/// 8. EU contribution (CALC-17)
/// 9. CFS check (CALC-18)
/// 10. Per-WP budget (CALC-20)
/// 11. Assemble BudgetSummaryDto
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

    let work_packages: Vec<(u8, u32, u32)> = (0..project.config.work_package_count as usize)
        .map(|i| {
            let id = (i + 1) as u8;
            let start = project.config.work_package_start_months.get(i).copied().unwrap_or(1);
            let end = project.config.work_package_end_months.get(i).copied().unwrap_or(duration as u32 * 12);
            (id, start, end)
        })
        .collect();

    // ── Step 1: Personnel ────────────────────────────────────────────────────

    let mut all_role_lines = Vec::new();
    let mut role_detail: Vec<PersonnelRoleDetailDto> = Vec::new();
    let mut personnel_wp_allocations: Vec<Vec<WpCostAmount>> = Vec::new();

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
            role.start_month,
            role.end_month,
        )?;

        let wp_breakdown = allocate_personnel_cost_by_wp(
            &projections,
            role.fte_fraction,
            role.start_month,
            role.end_month,
            &work_packages,
        )?;
        personnel_wp_allocations.push(wp_breakdown.clone());

        let total_role_cost: Decimal = cost_lines.iter().map(|l| l.annual_cost_eur).sum();

        let cost_line_dtos: Vec<RoleCostLineDto> = cost_lines
            .iter()
            .map(|l| RoleCostLineDto {
                year: l.year,
                is_active: l.is_active,
                active_months: l.active_months,
                monthly_salary_eur: l.monthly_salary_eur,
                annual_cost_eur: l.annual_cost_eur,
            })
            .collect();

        role_detail.push(PersonnelRoleDetailDto {
            id: role.id,
            role_label: role.role_label.clone(),
            role_type: role.role_type.clone(),
            current_monthly_salary_try: role.current_monthly_salary_try,
            inflation_rate_pct: role.inflation_rate_pct,
            fte_fraction: role.fte_fraction,
            start_month: role.start_month,
            end_month: role.end_month,
            cost_lines: cost_line_dtos,
            total_cost_eur: total_role_cost,
            wp_breakdown: wp_breakdown
                .into_iter()
                .map(|w| WpCostAmountDto { work_package_id: w.work_package_id, amount_eur: w.amount_eur })
                .collect(),
        });

        all_role_lines.push(cost_lines);
    }

    let personnel_totals = aggregate_personnel_costs(&all_role_lines, duration)?;
    let category_a_total = personnel_totals.total;

    // ── Step 2: Equipment ────────────────────────────────────────────────────

    let mut depreciation_results = Vec::new();
    let mut equipment_detail: Vec<EquipmentItemDetailDto> = Vec::new();
    let mut equipment_wp_items: Vec<(u8, Decimal)> = Vec::new();

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
        equipment_wp_items.push((item.work_package_id, result.eligible_depreciation_eur));
        depreciation_results.push(result);
    }

    let category_c2_total = aggregate_equipment_costs(&depreciation_results)?;

    // ── Step 3: Travel ───────────────────────────────────────────────────────

    let mut trip_costs: Vec<Decimal> = Vec::new();
    let mut trip_detail: Vec<TripDetailDto> = Vec::new();
    let mut travel_wp_items: Vec<(Vec<u8>, Decimal)> = Vec::new();

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
                    work_package_ids: trip.work_package_ids.clone(),
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
                    work_package_ids: trip.work_package_ids.clone(),
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
        let total_cost = trip_result.total_cost();
        travel_wp_items.push((trip.work_package_ids.clone(), total_cost));
        trip_costs.push(total_cost);
    }

    let travel_totals = aggregate_travel_costs(&trip_costs)?;
    let category_c1_total = travel_totals.total;

    // ── Step 4: C3 ───────────────────────────────────────────────────────────

    let c3_input: Vec<(Decimal, bool)> = project.other_cost_items
        .iter()
        .map(|i| (i.amount_eur, i.is_cfs_item))
        .collect();
    let c3_result = aggregate_c3_costs(&c3_input)?;
    let category_c3_total = c3_result.total;

    let other_cost_wp_items: Vec<(Vec<u8>, Decimal)> = project.other_cost_items
        .iter()
        .map(|i| (i.work_package_ids.clone(), i.amount_eur))
        .collect();

    let other_cost_detail: Vec<OtherCostItemDetailDto> = project.other_cost_items
        .iter()
        .map(|i| OtherCostItemDetailDto {
            id: i.id,
            name: i.name.clone(),
            amount_eur: i.amount_eur,
            is_cfs_item: i.is_cfs_item,
            notes: i.notes.clone(),
            work_package_ids: i.work_package_ids.clone(),
        })
        .collect();

    // ── Step 5: Direct Totals ────────────────────────────────────────────────

    let category_b_total = project.subcontracting.amount_eur;
    let total_direct = calculate_total_direct_costs(
        category_a_total, category_b_total, category_c1_total, category_c2_total, category_c3_total,
    )?;

    // ── Step 6: Indirect Costs ───────────────────────────────────────────────

    let indirect_result = calculate_indirect_costs(
        category_a_total,
        category_c1_total,
        category_c2_total,
        category_c3_total,
        project.config.indirect_cost_rate_pct,
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

    // ── Step 10: Per-WP Budget ───────────────────────────────────────────────

    let wp_budget_amounts = aggregate_wp_budgets(
        project.config.work_package_count,
        &project.config.work_package_names,
        &personnel_wp_allocations,
        &equipment_wp_items,
        &travel_wp_items,
        &other_cost_wp_items,
        (project.subcontracting.work_package_id, project.subcontracting.amount_eur),
    )?;

    let wp_budgets: Vec<WpBudgetDto> = wp_budget_amounts
        .into_iter()
        .map(|w| WpBudgetDto {
            work_package_id: w.work_package_id,
            work_package_name: w.work_package_name,
            personnel_eur: w.personnel_eur,
            equipment_eur: w.equipment_eur,
            travel_eur: w.travel_eur,
            other_costs_eur: w.other_costs_eur,
            subcontracting_eur: w.subcontracting_eur,
            total_eur: w.total_eur,
        })
        .collect();

    // ── Step 11: Assemble DTO ────────────────────────────────────────────────

    Ok(BudgetSummaryDto {
        wp_budgets,
        category_a_total,
        category_b_total,
        category_c1_total,
        category_c2_total,
        category_c3_total,
        indirect_base_total: indirect_result.base,
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
