//! CALC-20 — Per-Work-Package Budget Aggregation
//!
//! Builds one budget line per Work Package, summing each category's
//! contribution:
//! - Personnel: pre-computed per-role WP allocations (CALC-20a).
//! - Equipment: each item's cost goes entirely to its single WP.
//! - Travel / Other Direct Costs: each item's cost is split evenly across
//!   every WP it is tagged with.
//! - Subcontracting: the lump sum goes entirely to its single WP.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::calculation::personnel_cost::WpCostAmount;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WpBudgetAmount {
    pub work_package_id: u8,
    pub work_package_name: Option<String>,
    #[serde(with = "rust_decimal::serde::str")]
    pub personnel_eur: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub equipment_eur: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub travel_eur: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub other_costs_eur: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub subcontracting_eur: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub total_eur: Decimal,
}

/// CALC-20: Aggregate every category's cost into one budget line per WP.
///
/// # Arguments
/// * `work_package_count` — Number of WPs (1..=10).
/// * `work_package_names` — Optional display name per WP, length `work_package_count`.
/// * `personnel_allocations` — One `Vec<WpCostAmount>` per registered role (CALC-20a output).
/// * `equipment_items` — `(work_package_id, amount_eur)` per equipment item (single WP each).
/// * `travel_items` — `(work_package_ids, amount_eur)` per trip (cost split evenly across WPs).
/// * `other_cost_items` — `(work_package_ids, amount_eur)` per C3 item (cost split evenly across WPs).
/// * `subcontracting` — `(work_package_id, amount_eur)` for the single subcontracting lump sum.
pub fn aggregate_wp_budgets(
    work_package_count: u8,
    work_package_names: &[Option<String>],
    personnel_allocations: &[Vec<WpCostAmount>],
    equipment_items: &[(u8, Decimal)],
    travel_items: &[(Vec<u8>, Decimal)],
    other_cost_items: &[(Vec<u8>, Decimal)],
    subcontracting: (u8, Decimal),
) -> Result<Vec<WpBudgetAmount>, AppError> {
    let n = work_package_count as usize;
    let mut personnel = vec![Decimal::ZERO; n];
    let mut equipment = vec![Decimal::ZERO; n];
    let mut travel = vec![Decimal::ZERO; n];
    let mut other_costs = vec![Decimal::ZERO; n];
    let mut subcontracting_amounts = vec![Decimal::ZERO; n];

    let add = |bucket: &mut Vec<Decimal>, wp_id: u8, amount: Decimal| {
        if let Some(idx) = (wp_id as usize).checked_sub(1) {
            if idx < bucket.len() {
                bucket[idx] += amount;
            }
        }
    };

    for role_allocation in personnel_allocations {
        for wp_cost in role_allocation {
            add(&mut personnel, wp_cost.work_package_id, wp_cost.amount_eur);
        }
    }

    for &(wp_id, amount) in equipment_items {
        add(&mut equipment, wp_id, amount);
    }

    for (wp_ids, amount) in travel_items {
        if wp_ids.is_empty() {
            continue;
        }
        let share = *amount / Decimal::from(wp_ids.len() as u32);
        for &wp_id in wp_ids {
            add(&mut travel, wp_id, share);
        }
    }

    for (wp_ids, amount) in other_cost_items {
        if wp_ids.is_empty() {
            continue;
        }
        let share = *amount / Decimal::from(wp_ids.len() as u32);
        for &wp_id in wp_ids {
            add(&mut other_costs, wp_id, share);
        }
    }

    let (sub_wp_id, sub_amount) = subcontracting;
    add(&mut subcontracting_amounts, sub_wp_id, sub_amount);

    let mut result = Vec::with_capacity(n);
    for i in 0..n {
        let personnel_eur = personnel[i];
        let equipment_eur = equipment[i];
        let travel_eur = travel[i];
        let other_costs_eur = other_costs[i];
        let subcontracting_eur = subcontracting_amounts[i];
        result.push(WpBudgetAmount {
            work_package_id: (i + 1) as u8,
            work_package_name: work_package_names.get(i).cloned().flatten(),
            personnel_eur,
            equipment_eur,
            travel_eur,
            other_costs_eur,
            subcontracting_eur,
            total_eur: personnel_eur + equipment_eur + travel_eur + other_costs_eur + subcontracting_eur,
        });
    }

    Ok(result)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_calc_20_personnel_allocation_lands_in_right_wp() {
        let allocations = vec![vec![
            WpCostAmount { work_package_id: 1, amount_eur: dec!(5000) },
            WpCostAmount { work_package_id: 2, amount_eur: dec!(3000) },
        ]];
        let result = aggregate_wp_budgets(2, &[None, None], &allocations, &[], &[], &[], (1, Decimal::ZERO)).unwrap();
        assert_eq!(result[0].personnel_eur, dec!(5000));
        assert_eq!(result[1].personnel_eur, dec!(3000));
    }

    #[test]
    fn test_calc_20_equipment_goes_entirely_to_single_wp() {
        let equipment = vec![(2u8, dec!(2500))];
        let result = aggregate_wp_budgets(2, &[None, None], &[], &equipment, &[], &[], (1, Decimal::ZERO)).unwrap();
        assert_eq!(result[0].equipment_eur, Decimal::ZERO);
        assert_eq!(result[1].equipment_eur, dec!(2500));
    }

    #[test]
    fn test_calc_20_travel_split_evenly_across_multiple_wps() {
        let travel = vec![(vec![1u8, 2u8], dec!(1000))];
        let result = aggregate_wp_budgets(2, &[None, None], &[], &[], &travel, &[], (1, Decimal::ZERO)).unwrap();
        assert_eq!(result[0].travel_eur, dec!(500));
        assert_eq!(result[1].travel_eur, dec!(500));
    }

    #[test]
    fn test_calc_20_other_costs_split_evenly_across_multiple_wps() {
        let other = vec![(vec![1u8, 2u8, 3u8], dec!(300))];
        let result = aggregate_wp_budgets(3, &[None, None, None], &[], &[], &[], &other, (1, Decimal::ZERO)).unwrap();
        assert_eq!(result[0].other_costs_eur, dec!(100));
        assert_eq!(result[1].other_costs_eur, dec!(100));
        assert_eq!(result[2].other_costs_eur, dec!(100));
    }

    #[test]
    fn test_calc_20_subcontracting_goes_to_its_single_wp() {
        let result = aggregate_wp_budgets(2, &[None, None], &[], &[], &[], &[], (2, dec!(15000))).unwrap();
        assert_eq!(result[0].subcontracting_eur, Decimal::ZERO);
        assert_eq!(result[1].subcontracting_eur, dec!(15000));
    }

    #[test]
    fn test_calc_20_total_is_sum_of_all_categories() {
        let allocations = vec![vec![WpCostAmount { work_package_id: 1, amount_eur: dec!(1000) }]];
        let equipment = vec![(1u8, dec!(500))];
        let travel = vec![(vec![1u8], dec!(200))];
        let other = vec![(vec![1u8], dec!(100))];
        let result = aggregate_wp_budgets(1, &[None], &allocations, &equipment, &travel, &other, (1, dec!(300))).unwrap();
        assert_eq!(result[0].total_eur, dec!(2100));
    }

    #[test]
    fn test_calc_20_wp_name_is_included() {
        let names = vec![Some("Data Collection".to_string())];
        let result = aggregate_wp_budgets(1, &names, &[], &[], &[], &[], (1, Decimal::ZERO)).unwrap();
        assert_eq!(result[0].work_package_name, Some("Data Collection".to_string()));
    }
}
