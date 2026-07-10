//! CALC-03 — Annual Personnel Cost per Role
//! CALC-04 — Total Personnel Cost (Category A)
//!
//! Converts year-specific projected salaries to annual grant costs
//! using FTE fraction and active-year status.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::calculation::salary_projection::SalaryProjection;
use crate::error::{AppError, calc_error};

// ─── Output Types ─────────────────────────────────────────────────────────────

/// Cost contribution of one role in one project year.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonnelCostLine {
    pub year: u8,
    pub is_active: bool,
    /// Always 12 if active, 0 if not. No other values are valid.
    pub active_months: u8,
    #[serde(with = "rust_decimal::serde::str")]
    pub monthly_salary_eur: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub annual_cost_eur: Decimal,
}

/// Per-year totals aggregated across all registered roles (CALC-04 output).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonnelCategoryTotals {
    pub by_year: Vec<YearAmount>,
    #[serde(with = "rust_decimal::serde::str")]
    pub total: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearAmount {
    pub year: u8,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount_eur: Decimal,
}

// ─── CALC-03 ─────────────────────────────────────────────────────────────────

/// CALC-03: Compute year-by-year personnel cost lines for one role.
///
/// For each project year:
///   - If year is in `active_years`: cost = monthly_salary × 12 × fte_fraction
///   - Otherwise: cost = 0
///
/// # Arguments
/// * `salary_projections` — Output of CALC-02. One entry per project year.
/// * `fte_fraction` — Fraction of time dedicated to grant. Range: (0, 1].
/// * `active_years` — Project years (1-indexed) when this role is charged.
pub fn calculate_personnel_cost_lines(
    salary_projections: &[SalaryProjection],
    fte_fraction: Decimal,
    active_years: &[u8],
) -> Result<Vec<PersonnelCostLine>, AppError> {
    if fte_fraction <= Decimal::ZERO || fte_fraction > Decimal::ONE {
        return Err(calc_error(
            "INVALID_FTE",
            "PM fraction must be between 0 (exclusive) and 1 (inclusive).",
        ));
    }
    if active_years.is_empty() {
        return Err(calc_error(
            "NO_ACTIVE_YEARS",
            "At least one active project year must be selected.",
        ));
    }

    // Validate that all active_years appear in the projections.
    let max_year = salary_projections.iter().map(|p| p.year).max().unwrap_or(0);
    for &y in active_years {
        if y < 1 || y > max_year {
            return Err(calc_error(
                "YEAR_OUT_OF_RANGE",
                format!("Active year {y} is outside the project duration of {max_year} years."),
            ));
        }
    }

    let twelve = Decimal::from(12u8);
    let mut cost_lines = Vec::with_capacity(salary_projections.len());

    for projection in salary_projections {
        let is_active = active_years.contains(&projection.year);
        let active_months: u8 = if is_active { 12 } else { 0 };
        let annual_cost = if is_active {
            projection.projected_monthly_eur * twelve * fte_fraction
        } else {
            Decimal::ZERO
        };

        // Post-condition: active years must have positive cost.
        if is_active && annual_cost <= Decimal::ZERO {
            return Err(calc_error(
                "INTERNAL_CALC_ERROR",
                "Annual cost for an active year is zero or negative. This is a bug.",
            ));
        }
        // Post-condition: inactive years must have zero cost.
        if !is_active && annual_cost != Decimal::ZERO {
            return Err(calc_error(
                "INTERNAL_CALC_ERROR",
                "Annual cost for an inactive year is non-zero. This is a bug.",
            ));
        }

        cost_lines.push(PersonnelCostLine {
            year: projection.year,
            is_active,
            active_months,
            monthly_salary_eur: projection.projected_monthly_eur,
            annual_cost_eur: annual_cost,
        });
    }

    Ok(cost_lines)
}

// ─── CALC-04 ─────────────────────────────────────────────────────────────────

/// CALC-04: Aggregate personnel cost lines across all roles to produce
/// per-year totals and the Category A grand total.
///
/// # Arguments
/// * `all_role_lines` — One `Vec<PersonnelCostLine>` per registered role.
/// * `duration_years` — Total project duration. Initialises zero entries for all years.
pub fn aggregate_personnel_costs(
    all_role_lines: &[Vec<PersonnelCostLine>],
    duration_years: u8,
) -> Result<PersonnelCategoryTotals, AppError> {
    let mut year_totals: Vec<Decimal> = vec![Decimal::ZERO; duration_years as usize];

    for role_lines in all_role_lines {
        for line in role_lines {
            let idx = (line.year as usize).checked_sub(1).ok_or_else(|| {
                calc_error("INTERNAL_CALC_ERROR", "Year index underflow in personnel aggregation.")
            })?;
            if idx >= year_totals.len() {
                return Err(calc_error(
                    "INTERNAL_CALC_ERROR",
                    format!("Year {} is out of range for duration {}.", line.year, duration_years),
                ));
            }
            year_totals[idx] += line.annual_cost_eur;
        }
    }

    let total: Decimal = year_totals.iter().sum();
    let by_year: Vec<YearAmount> = year_totals
        .into_iter()
        .enumerate()
        .map(|(i, amt)| YearAmount { year: (i + 1) as u8, amount_eur: amt })
        .collect();

    Ok(PersonnelCategoryTotals { by_year, total })
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use crate::calculation::salary_projection::SalaryProjection;

    fn make_projections(values: &[(u8, &str)]) -> Vec<SalaryProjection> {
        values
            .iter()
            .map(|(y, v)| SalaryProjection {
                year: *y,
                projected_monthly_eur: v.parse().unwrap(),
            })
            .collect()
    }

    // ── CALC-03 tests ──

    #[test]
    fn test_calc_03_pi_all_years_active() {
        // PI: Year 1 €5,402.61/month, FTE 0.70, active all 5 years
        let projections = make_projections(&[
            (1, "5402.61"), (2, "6483.13"), (3, "7779.75"), (4, "9335.70"), (5, "11202.84"),
        ]);
        let lines = calculate_personnel_cost_lines(&projections, dec!(0.70), &[1, 2, 3, 4, 5]).unwrap();
        assert_eq!(lines.len(), 5);
        // Year 1: 5402.61 × 12 × 0.70 = 45,381.924
        let y1 = lines[0].annual_cost_eur.round_dp(2);
        assert_eq!(y1, dec!(45381.92));
        assert!(lines[0].is_active);
        assert_eq!(lines[0].active_months, 12);
    }

    #[test]
    fn test_calc_03_postdoc_year_2_only() {
        // PostDoc-1: Year 2 €3,967.50/month, FTE 1.00, active Year 2 only
        let projections = make_projections(&[
            (1, "3450.00"), (2, "3967.50"), (3, "4562.63"), (4, "5247.02"), (5, "6034.07"),
        ]);
        let lines = calculate_personnel_cost_lines(&projections, dec!(1.0), &[2]).unwrap();
        assert_eq!(lines.len(), 5);
        // Year 1 inactive
        assert!(!lines[0].is_active);
        assert_eq!(lines[0].annual_cost_eur, Decimal::ZERO);
        assert_eq!(lines[0].active_months, 0);
        // Year 2 active: 3967.50 × 12 × 1.0 = 47,610.00
        assert!(lines[1].is_active);
        assert_eq!(lines[1].annual_cost_eur, dec!(47610.00));
        assert_eq!(lines[1].active_months, 12);
        // Years 3-5 inactive
        for i in 2..5 {
            assert!(!lines[i].is_active);
            assert_eq!(lines[i].annual_cost_eur, Decimal::ZERO);
        }
    }

    #[test]
    fn test_calc_03_expert_year_1_only_fte_04() {
        let projections = make_projections(&[(1, "3450.00")]);
        let lines = calculate_personnel_cost_lines(&projections, dec!(0.4), &[1]).unwrap();
        // 3450 × 12 × 0.4 = 16,560
        assert_eq!(lines[0].annual_cost_eur, dec!(16560.00));
    }

    #[test]
    fn test_calc_03_invalid_fte_zero() {
        let projections = make_projections(&[(1, "3000")]);
        let result = calculate_personnel_cost_lines(&projections, Decimal::ZERO, &[1]);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_FTE"));
    }

    #[test]
    fn test_calc_03_invalid_fte_over_one() {
        let projections = make_projections(&[(1, "3000")]);
        let result = calculate_personnel_cost_lines(&projections, dec!(1.1), &[1]);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_FTE"));
    }

    #[test]
    fn test_calc_03_no_active_years_returns_error() {
        let projections = make_projections(&[(1, "3000"), (2, "3450")]);
        let result = calculate_personnel_cost_lines(&projections, dec!(1.0), &[]);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "NO_ACTIVE_YEARS"));
    }

    #[test]
    fn test_calc_03_active_year_out_of_range() {
        let projections = make_projections(&[(1, "3000"), (2, "3450")]);
        let result = calculate_personnel_cost_lines(&projections, dec!(1.0), &[3]);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "YEAR_OUT_OF_RANGE"));
    }

    #[test]
    fn test_calc_03_non_contiguous_active_years() {
        // Active Year 1 and Year 3, not Year 2
        let projections = make_projections(&[(1, "3000"), (2, "3450"), (3, "3967.50")]);
        let lines = calculate_personnel_cost_lines(&projections, dec!(1.0), &[1, 3]).unwrap();
        assert!(lines[0].is_active);
        assert!(!lines[1].is_active);
        assert!(lines[2].is_active);
        assert_eq!(lines[1].annual_cost_eur, Decimal::ZERO);
    }

    // ── CALC-04 tests ──

    #[test]
    fn test_calc_04_sum_across_roles() {
        let role1_lines = vec![
            PersonnelCostLine { year: 1, is_active: true, active_months: 12, monthly_salary_eur: dec!(5000), annual_cost_eur: dec!(42000) },
            PersonnelCostLine { year: 2, is_active: true, active_months: 12, monthly_salary_eur: dec!(6000), annual_cost_eur: dec!(50400) },
        ];
        let role2_lines = vec![
            PersonnelCostLine { year: 1, is_active: false, active_months: 0, monthly_salary_eur: dec!(3000), annual_cost_eur: Decimal::ZERO },
            PersonnelCostLine { year: 2, is_active: true, active_months: 12, monthly_salary_eur: dec!(3450), annual_cost_eur: dec!(41400) },
        ];
        let totals = aggregate_personnel_costs(&[role1_lines, role2_lines], 2).unwrap();
        assert_eq!(totals.by_year[0].year, 1);
        assert_eq!(totals.by_year[0].amount_eur, dec!(42000));
        assert_eq!(totals.by_year[1].year, 2);
        assert_eq!(totals.by_year[1].amount_eur, dec!(91800)); // 50400 + 41400
        assert_eq!(totals.total, dec!(133800));
    }

    #[test]
    fn test_calc_04_empty_roles_gives_zero() {
        let totals = aggregate_personnel_costs(&[], 5).unwrap();
        assert_eq!(totals.total, Decimal::ZERO);
        assert_eq!(totals.by_year.len(), 5);
        for y in &totals.by_year {
            assert_eq!(y.amount_eur, Decimal::ZERO);
        }
    }

    #[test]
    fn test_calc_04_total_equals_sum_of_year_amounts() {
        let role_lines = vec![
            PersonnelCostLine { year: 1, is_active: true, active_months: 12, monthly_salary_eur: dec!(5000), annual_cost_eur: dec!(42000) },
            PersonnelCostLine { year: 2, is_active: true, active_months: 12, monthly_salary_eur: dec!(6000), annual_cost_eur: dec!(50400) },
            PersonnelCostLine { year: 3, is_active: false, active_months: 0, monthly_salary_eur: dec!(7200), annual_cost_eur: Decimal::ZERO },
        ];
        let totals = aggregate_personnel_costs(&[role_lines], 3).unwrap();
        let sum_of_years: Decimal = totals.by_year.iter().map(|y| y.amount_eur).sum();
        assert_eq!(totals.total, sum_of_years);
    }
}
