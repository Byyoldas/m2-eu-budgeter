//! CALC-13 — Total Other Direct Costs (Category C3)
//! CALC-14 — Indirect Costs (Category E)
//! CALC-15 — Total Direct Costs
//! CALC-16 — Total Eligible Costs
//! CALC-17 — Requested EU Contribution
//!
//! These functions aggregate all cost categories into the final budget figures.
//! Category B (Subcontracting) is included in direct costs but excluded from
//! the indirect cost base — this is an ERC rule.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::error::{AppError, calc_error};

// ─── Output Types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearCostEntry {
    pub year: u8,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount_eur: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C3CategoryTotals {
    pub by_year: Vec<YearCostEntry>,
    #[serde(with = "rust_decimal::serde::str")]
    pub total: Decimal,
    pub has_cfs_item: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndirectCostResult {
    #[serde(with = "rust_decimal::serde::str")]
    pub base: Decimal,
    pub by_year: Vec<YearCostEntry>,
    #[serde(with = "rust_decimal::serde::str")]
    pub total: Decimal,
}

// ─── CALC-13 ─────────────────────────────────────────────────────────────────

/// CALC-13: Aggregate all C3 items (including any CFS item).
///
/// # Arguments
/// * `c3_items` — Vec of (year, amount_eur, is_cfs_item) tuples.
/// * `duration_years` — Total project years.
pub fn aggregate_c3_costs(
    c3_items: &[(u8, Decimal, bool)], // (year, amount, is_cfs)
    duration_years: u8,
) -> Result<C3CategoryTotals, AppError> {
    // Validate: at most one CFS item
    let cfs_count = c3_items.iter().filter(|(_, _, is_cfs)| *is_cfs).count();
    if cfs_count > 1 {
        return Err(calc_error(
            "DUPLICATE_CFS_ITEM",
            "Only one Certificate on Financial Statements item is allowed.",
        ));
    }

    let mut year_totals: Vec<Decimal> = vec![Decimal::ZERO; duration_years as usize];

    for &(year, amount, _) in c3_items {
        if amount <= Decimal::ZERO {
            return Err(calc_error(
                "INVALID_C3_AMOUNT",
                "Cost item amount must be greater than zero.",
            ));
        }
        if year < 1 || year > duration_years {
            return Err(calc_error(
                "YEAR_OUT_OF_RANGE",
                format!("Cost item is assigned to year {year}, which is outside the project duration."),
            ));
        }
        year_totals[(year - 1) as usize] += amount;
    }

    let total: Decimal = year_totals.iter().sum();
    let by_year = year_totals
        .into_iter()
        .enumerate()
        .map(|(i, amt)| YearCostEntry { year: (i + 1) as u8, amount_eur: amt })
        .collect();

    Ok(C3CategoryTotals {
        by_year,
        total,
        has_cfs_item: cfs_count == 1,
    })
}

// ─── CALC-14 ─────────────────────────────────────────────────────────────────

/// CALC-14: Calculate indirect costs (Category E) per year and total.
///
/// Base = A + C1 + C2 + C3 per year.
/// Category B (Subcontracting) is explicitly excluded from the base.
///
/// C2 has no year assignment — it is distributed evenly across years for the
/// per-year display and indirect calculation. The project-total indirect cost
/// is calculated from the project-level totals (exact, no distribution needed).
///
/// # Arguments
/// * `a_by_year` — Personnel costs per year (from CALC-04).
/// * `c1_by_year` — Travel costs per year (from CALC-12).
/// * `c2_total` — Total equipment depreciation (from CALC-06, year-agnostic).
/// * `c3_by_year` — Other direct costs per year (from CALC-13).
/// * `indirect_rate_pct` — Overhead rate as a percentage (e.g. 25.0 for 25%).
/// * `duration_years` — Total project duration.
pub fn calculate_indirect_costs(
    a_by_year: &[YearCostEntry],
    c1_by_year: &[YearCostEntry],
    c2_total: Decimal,
    c3_by_year: &[YearCostEntry],
    indirect_rate_pct: Decimal,
    duration_years: u8,
) -> Result<IndirectCostResult, AppError> {
    if indirect_rate_pct < Decimal::ZERO || indirect_rate_pct > Decimal::from(50u8) {
        return Err(calc_error(
            "INVALID_INDIRECT_RATE",
            "Indirect cost rate must be between 0% and 50%.",
        ));
    }

    let rate_fraction = indirect_rate_pct / Decimal::ONE_HUNDRED;
    // Distribute C2 evenly across years for per-year indirect calculation.
    let c2_per_year = if duration_years > 0 {
        c2_total / Decimal::from(duration_years)
    } else {
        Decimal::ZERO
    };

    let mut by_year: Vec<YearCostEntry> = Vec::with_capacity(duration_years as usize);
    let mut total_base = Decimal::ZERO;

    for year in 1..=duration_years {
        let idx = (year - 1) as usize;
        let a = a_by_year.get(idx).map(|y| y.amount_eur).unwrap_or(Decimal::ZERO);
        let c1 = c1_by_year.get(idx).map(|y| y.amount_eur).unwrap_or(Decimal::ZERO);
        let c2 = c2_per_year;
        let c3 = c3_by_year.get(idx).map(|y| y.amount_eur).unwrap_or(Decimal::ZERO);

        let year_base = a + c1 + c2 + c3;
        total_base += year_base;
        let year_indirect = year_base * rate_fraction;

        by_year.push(YearCostEntry { year, amount_eur: year_indirect });
    }

    // Project-level total uses the exact base (not summed year estimates).
    // Recalculate from the summed base for precision.
    let total = total_base * rate_fraction;

    Ok(IndirectCostResult {
        base: total_base,
        by_year,
        total,
    })
}

// ─── CALC-15 ─────────────────────────────────────────────────────────────────

/// CALC-15: Total Direct Costs = A + B + C1 + C2 + C3.
pub fn calculate_total_direct_costs(
    a_total: Decimal,
    b_total: Decimal,
    c1_total: Decimal,
    c2_total: Decimal,
    c3_total: Decimal,
) -> Result<Decimal, AppError> {
    for (name, val) in [("A", a_total), ("B", b_total), ("C1", c1_total), ("C2", c2_total), ("C3", c3_total)] {
        if val < Decimal::ZERO {
            return Err(calc_error(
                "INTERNAL_CALC_ERROR",
                format!("Category {name} total is negative. This is a bug."),
            ));
        }
    }
    Ok(a_total + b_total + c1_total + c2_total + c3_total)
}

// ─── CALC-16 ─────────────────────────────────────────────────────────────────

/// CALC-16: Total Eligible Costs = (Total Direct Costs − Subcontracting) + Indirect Costs (E).
///
/// Category B (Subcontracting) is not an eligible cost under ERC rules: it is
/// tracked as part of the project's total direct spend but must not count
/// toward eligible costs or the requested EU contribution.
pub fn calculate_total_eligible_costs(
    total_direct: Decimal,
    category_b_total: Decimal,
    category_e: Decimal,
) -> Result<Decimal, AppError> {
    if total_direct < Decimal::ZERO {
        return Err(calc_error("INTERNAL_CALC_ERROR", "Total direct costs is negative."));
    }
    if category_b_total < Decimal::ZERO {
        return Err(calc_error("INTERNAL_CALC_ERROR", "Subcontracting total is negative."));
    }
    if category_e < Decimal::ZERO {
        return Err(calc_error("INTERNAL_CALC_ERROR", "Indirect costs is negative."));
    }
    let eligible_direct = total_direct - category_b_total;
    if eligible_direct < Decimal::ZERO {
        return Err(calc_error(
            "INTERNAL_CALC_ERROR",
            "Subcontracting total exceeds total direct costs. This is a bug.",
        ));
    }
    Ok(eligible_direct + category_e)
}

// ─── CALC-17 ─────────────────────────────────────────────────────────────────

/// CALC-17: Requested EU Contribution = Total Eligible Costs (100% EU funding).
///
/// Kept as a named function because the funding model is an explicit business rule
/// that may change in future versions.
pub fn calculate_requested_contribution(total_eligible: Decimal) -> Result<Decimal, AppError> {
    // EU_FUNDING_RATE = 1.0 for ERC Actual Costs (100%).
    const EU_FUNDING_RATE: u8 = 1;
    if total_eligible < Decimal::ZERO {
        return Err(calc_error("INTERNAL_CALC_ERROR", "Total eligible costs is negative."));
    }
    Ok(total_eligible * Decimal::from(EU_FUNDING_RATE))
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn year_entries(values: &[(u8, &str)]) -> Vec<YearCostEntry> {
        values.iter().map(|(y, v)| YearCostEntry { year: *y, amount_eur: v.parse().unwrap() }).collect()
    }

    // ── CALC-13 tests ──

    #[test]
    fn test_calc_13_aggregate_c3_across_years() {
        let items = vec![
            (1, dec!(9870), false),   // MAXQDA
            (3, dec!(5000), false),   // publications year 3
            (4, dec!(5000), false),   // publications year 4
            (4, dec!(12000), true),   // CFS
            (5, dec!(5000), false),   // publications year 5
        ];
        let result = aggregate_c3_costs(&items, 5).unwrap();
        assert_eq!(result.by_year[0].amount_eur, dec!(9870));  // year 1
        assert_eq!(result.by_year[1].amount_eur, dec!(0));     // year 2
        assert_eq!(result.by_year[2].amount_eur, dec!(5000));  // year 3
        assert_eq!(result.by_year[3].amount_eur, dec!(17000)); // year 4: 5000+12000
        assert_eq!(result.by_year[4].amount_eur, dec!(5000));  // year 5
        assert_eq!(result.total, dec!(36870));
        assert!(result.has_cfs_item);
    }

    #[test]
    fn test_calc_13_no_items_zero_total() {
        let result = aggregate_c3_costs(&[], 5).unwrap();
        assert_eq!(result.total, Decimal::ZERO);
        assert!(!result.has_cfs_item);
    }

    #[test]
    fn test_calc_13_duplicate_cfs_returns_error() {
        let items = vec![(1, dec!(5000), true), (2, dec!(6000), true)];
        let result = aggregate_c3_costs(&items, 3);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "DUPLICATE_CFS_ITEM"));
    }

    #[test]
    fn test_calc_13_zero_amount_returns_error() {
        let items = vec![(1, Decimal::ZERO, false)];
        let result = aggregate_c3_costs(&items, 3);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_C3_AMOUNT"));
    }

    #[test]
    fn test_calc_13_year_out_of_range_returns_error() {
        let items = vec![(6, dec!(1000), false)];
        let result = aggregate_c3_costs(&items, 5);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "YEAR_OUT_OF_RANGE"));
    }

    // ── CALC-14 tests ──

    #[test]
    fn test_calc_14_indirect_25pct_excludes_b() {
        // A=100, C1=20, C2=10, C3=30 → base=160 → E=40
        let a = year_entries(&[(1, "100")]);
        let c1 = year_entries(&[(1, "20")]);
        let c3 = year_entries(&[(1, "30")]);
        let result = calculate_indirect_costs(&a, &c1, dec!(10), &c3, dec!(25), 1).unwrap();
        assert_eq!(result.base, dec!(160));
        assert_eq!(result.total, dec!(40));
    }

    #[test]
    fn test_calc_14_zero_rate_gives_zero_indirect() {
        let a = year_entries(&[(1, "100000")]);
        let result = calculate_indirect_costs(&a, &[], Decimal::ZERO, &[], dec!(0), 1).unwrap();
        assert_eq!(result.total, Decimal::ZERO);
    }

    #[test]
    fn test_calc_14_invalid_rate_over_50_returns_error() {
        let result = calculate_indirect_costs(&[], &[], Decimal::ZERO, &[], dec!(51), 1);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_INDIRECT_RATE"));
    }

    #[test]
    fn test_calc_14_by_year_length_matches_duration() {
        let a = year_entries(&[(1, "50000"), (2, "60000"), (3, "70000")]);
        let result = calculate_indirect_costs(&a, &[], Decimal::ZERO, &[], dec!(25), 3).unwrap();
        assert_eq!(result.by_year.len(), 3);
    }

    // ── CALC-15 tests ──

    #[test]
    fn test_calc_15_sum_all_categories() {
        let total = calculate_total_direct_costs(
            dec!(400000), dec!(0), dec!(21661), dec!(2536), dec!(36870),
        ).unwrap();
        assert_eq!(total, dec!(461067));
    }

    #[test]
    fn test_calc_15_with_subcontracting() {
        let total = calculate_total_direct_costs(
            dec!(100000), dec!(15000), dec!(5000), dec!(2000), dec!(10000),
        ).unwrap();
        assert_eq!(total, dec!(132000));
    }

    // ── CALC-16 tests ──

    #[test]
    fn test_calc_16_total_plus_indirect() {
        let result = calculate_total_eligible_costs(dec!(461067), Decimal::ZERO, dec!(115266.75)).unwrap();
        assert_eq!(result, dec!(576333.75));
    }

    #[test]
    fn test_calc_16_excludes_subcontracting() {
        // Total direct = 132000 (includes 15000 subcontracting), indirect = 10000.
        // Eligible = (132000 - 15000) + 10000 = 127000.
        let result = calculate_total_eligible_costs(dec!(132000), dec!(15000), dec!(10000)).unwrap();
        assert_eq!(result, dec!(127000));
    }

    #[test]
    fn test_calc_16_subcontracting_exceeds_direct_returns_error() {
        let result = calculate_total_eligible_costs(dec!(1000), dec!(2000), dec!(0));
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INTERNAL_CALC_ERROR"));
    }

    // ── CALC-17 tests ──

    #[test]
    fn test_calc_17_equals_total_eligible() {
        let contribution = calculate_requested_contribution(dec!(576333.75)).unwrap();
        assert_eq!(contribution, dec!(576333.75));
    }

    #[test]
    fn test_calc_17_zero_eligible_gives_zero_contribution() {
        let contribution = calculate_requested_contribution(Decimal::ZERO).unwrap();
        assert_eq!(contribution, Decimal::ZERO);
    }
}
