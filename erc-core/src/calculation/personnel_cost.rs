//! CALC-03 — Annual Personnel Cost per Role
//! CALC-04 — Total Personnel Cost (Category A)
//! CALC-20a — Personnel Cost Allocation by Work Package
//!
//! Converts year-specific projected salaries to annual grant costs
//! using FTE fraction and a Start Month/End Month charging period, prorating
//! by the number of months of that period that fall in each project year.
//!
//! Cost is driven by **Person-Months rounded to 1 decimal place** (standard
//! round-half-up — a second decimal of 5 or higher rounds the first decimal
//! up), matching the EU Funding & Tenders Portal's own Person-Months
//! convention: the portal itself computes requested cost as PM × unit rate
//! using the *rounded* PM, not the exact fractional value, so this app must
//! round at the same point to reproduce the same requested amount.

use crate::calculation::salary_projection::SalaryProjection;
use crate::error::{calc_error, AppError};
use rust_decimal::{Decimal, RoundingStrategy};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Round a Person-Months figure to 1 decimal place, standard round-half-up
/// (e.g. 2.37 -> 2.4, 2.34 -> 2.3), matching the EU Funding & Tenders
/// Portal's Person-Months display and the rate at which it computes cost.
pub fn round_person_months(pm: Decimal) -> Decimal {
    pm.round_dp_with_strategy(1, RoundingStrategy::MidpointAwayFromZero)
}

// ─── Output Types ─────────────────────────────────────────────────────────────

/// Cost contribution of one role in one project year.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonnelCostLine {
    pub year: u8,
    pub is_active: bool,
    /// Number of months (0–12) of the role's Start/End Month period that fall
    /// within this project year.
    pub active_months: u8,
    /// Person-Months for this year (`active_months × fte_fraction`), rounded
    /// to 1 decimal place — this, not the exact fractional value, is what
    /// `annual_cost_eur` is actually computed from.
    #[serde(with = "rust_decimal::serde::str")]
    pub person_months: Decimal,
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

/// A Work Package's share of one role's personnel cost.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WpCostAmount {
    pub work_package_id: u8,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount_eur: Decimal,
}

// ─── CALC-03 ─────────────────────────────────────────────────────────────────

/// CALC-03: Compute year-by-year personnel cost lines for one role.
///
/// For each project year, the role's `[start_month, end_month]` charging period
/// is intersected with that year's month range to find `active_months` (0–12):
///   cost = monthly_salary × active_months × fte_fraction
///
/// # Arguments
/// * `salary_projections` — Output of CALC-02. One entry per project year.
/// * `fte_fraction` — Fraction of time dedicated to grant. Range: (0, 1].
/// * `start_month` — First project month (1-indexed) this role is charged.
/// * `end_month` — Last project month (1-indexed, inclusive) this role is charged.
pub fn calculate_personnel_cost_lines(
    salary_projections: &[SalaryProjection],
    fte_fraction: Decimal,
    start_month: u32,
    end_month: u32,
) -> Result<Vec<PersonnelCostLine>, AppError> {
    if fte_fraction <= Decimal::ZERO || fte_fraction > Decimal::ONE {
        return Err(calc_error(
            "INVALID_FTE",
            "PM fraction must be between 0 (exclusive) and 1 (inclusive).",
        ));
    }
    if start_month < 1 || end_month < start_month {
        return Err(calc_error(
            "INVALID_MONTH_RANGE",
            "Start month must be on or before end month.",
        ));
    }

    let max_year = salary_projections.iter().map(|p| p.year).max().unwrap_or(0);
    let max_month = max_year as u32 * 12;
    if end_month > max_month {
        return Err(calc_error(
            "MONTH_OUT_OF_RANGE",
            format!("End month {end_month} is outside the project duration of {max_month} months."),
        ));
    }

    let mut cost_lines = Vec::with_capacity(salary_projections.len());

    for projection in salary_projections {
        let year_start = (projection.year as u32 - 1) * 12 + 1;
        let year_end = projection.year as u32 * 12;
        let overlap_start = start_month.max(year_start);
        let overlap_end = end_month.min(year_end);
        let active_months: u8 = if overlap_end >= overlap_start {
            (overlap_end - overlap_start + 1) as u8
        } else {
            0
        };
        let is_active = active_months > 0;
        let person_months = if is_active {
            round_person_months(Decimal::from(active_months) * fte_fraction)
        } else {
            Decimal::ZERO
        };
        let annual_cost = projection.projected_monthly_eur * person_months;

        // Post-condition: inactive years must have zero Person-Months and cost.
        // (An active year can legitimately round to 0.0 PM — and so €0 — only
        // for a vanishingly small FTE fraction on a single-month period; that
        // matches what the portal itself would report, not a bug.)
        if !is_active && (person_months != Decimal::ZERO || annual_cost != Decimal::ZERO) {
            return Err(calc_error(
                "INTERNAL_CALC_ERROR",
                "Person-Months or cost for an inactive year is non-zero. This is a bug.",
            ));
        }
        if annual_cost < Decimal::ZERO {
            return Err(calc_error(
                "INTERNAL_CALC_ERROR",
                "Annual cost is negative. This is a bug.",
            ));
        }

        cost_lines.push(PersonnelCostLine {
            year: projection.year,
            is_active,
            active_months,
            person_months,
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
                calc_error(
                    "INTERNAL_CALC_ERROR",
                    "Year index underflow in personnel aggregation.",
                )
            })?;
            if idx >= year_totals.len() {
                return Err(calc_error(
                    "INTERNAL_CALC_ERROR",
                    format!(
                        "Year {} is out of range for duration {}.",
                        line.year, duration_years
                    ),
                ));
            }
            year_totals[idx] += line.annual_cost_eur;
        }
    }

    let total: Decimal = year_totals.iter().sum();
    let by_year: Vec<YearAmount> = year_totals
        .into_iter()
        .enumerate()
        .map(|(i, amt)| YearAmount {
            year: (i + 1) as u8,
            amount_eur: amt,
        })
        .collect();

    Ok(PersonnelCategoryTotals { by_year, total })
}

// ─── CALC-20a ────────────────────────────────────────────────────────────────

/// CALC-20a: Split one role's personnel cost across the Work Packages whose
/// month range overlaps the role's `[start_month, end_month]` charging period.
///
/// Iterates month-by-month: each month's cost (that year's projected monthly
/// salary × fte_fraction) is split evenly across every WP whose own
/// `[start_month, end_month]` contains that month. A month that falls outside
/// every WP contributes to the category total (via CALC-03/04) but is not
/// attributed to any WP bucket here.
///
/// # Arguments
/// * `work_packages` — `(work_package_id, start_month, end_month)` for every WP.
pub fn allocate_personnel_cost_by_wp(
    salary_projections: &[SalaryProjection],
    fte_fraction: Decimal,
    start_month: u32,
    end_month: u32,
    work_packages: &[(u8, u32, u32)],
) -> Result<Vec<WpCostAmount>, AppError> {
    // Pass 1: each WP's exact (unrounded) Person-Months share per year, via
    // the same month-by-month reciprocal split as before — in PM units, not
    // money yet, so rounding can happen once per year (matching CALC-03)
    // rather than independently per WP.
    let mut year_wp_pm: BTreeMap<u8, BTreeMap<u8, Decimal>> = BTreeMap::new();

    for month in start_month..=end_month {
        let year = ((month - 1) / 12 + 1) as u8;
        let containing: Vec<u8> = work_packages
            .iter()
            .filter(|&&(_, s, e)| month >= s && month <= e)
            .map(|&(id, _, _)| id)
            .collect();
        if containing.is_empty() {
            continue;
        }
        let month_pm_share = fte_fraction / Decimal::from(containing.len() as u32);
        let year_entry = year_wp_pm.entry(year).or_default();
        for wp_id in containing {
            *year_entry.entry(wp_id).or_insert(Decimal::ZERO) += month_pm_share;
        }
    }

    // Pass 2: round each year's total (WP-covered) Person-Months the same
    // way CALC-03 does, turn that into the year's annual cost, then split
    // that exact rounded amount across WPs in proportion to each WP's
    // unrounded PM share — so the split always sums to precisely the
    // rounded total, with no per-WP rounding drift.
    //
    // Every WP but the last in a year gets its proportional share directly;
    // the last gets the exact remainder instead of its own division. A ratio
    // like 4/12 has no terminating decimal representation, so computing
    // every WP's share independently can leak a sub-cent residual that
    // makes the parts not sum exactly back to annual_cost — giving the
    // last WP the remainder guarantees they always do.
    let mut totals: BTreeMap<u8, Decimal> = BTreeMap::new();
    for (year, wp_pm) in &year_wp_pm {
        let total_pm: Decimal = wp_pm.values().sum();
        if total_pm.is_zero() {
            continue;
        }
        let monthly_eur = salary_projections
            .iter()
            .find(|p| p.year == *year)
            .map(|p| p.projected_monthly_eur)
            .unwrap_or(Decimal::ZERO);
        let annual_cost = monthly_eur * round_person_months(total_pm);

        let mut remaining = annual_cost;
        let wp_count = wp_pm.len();
        for (i, (&wp_id, &pm)) in wp_pm.iter().enumerate() {
            let amount = if i + 1 == wp_count {
                remaining
            } else {
                let share = annual_cost * pm / total_pm;
                remaining -= share;
                share
            };
            *totals.entry(wp_id).or_insert(Decimal::ZERO) += amount;
        }
    }

    Ok(totals
        .into_iter()
        .map(|(work_package_id, amount_eur)| WpCostAmount {
            work_package_id,
            amount_eur,
        })
        .collect())
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculation::salary_projection::SalaryProjection;
    use rust_decimal_macros::dec;

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
        // PI: Year 1 €5,402.61/month, FTE 0.70, active months 1-60 (all 5 years)
        let projections = make_projections(&[
            (1, "5402.61"),
            (2, "6483.13"),
            (3, "7779.75"),
            (4, "9335.70"),
            (5, "11202.84"),
        ]);
        let lines = calculate_personnel_cost_lines(&projections, dec!(0.70), 1, 60).unwrap();
        assert_eq!(lines.len(), 5);
        // Year 1: 5402.61 × 12 × 0.70 = 45,381.924
        let y1 = lines[0].annual_cost_eur.round_dp(2);
        assert_eq!(y1, dec!(45381.92));
        assert!(lines[0].is_active);
        assert_eq!(lines[0].active_months, 12);
    }

    #[test]
    fn test_calc_03_postdoc_year_2_only() {
        // PostDoc-1: Year 2 €3,967.50/month, FTE 1.00, active months 13-24 (Year 2 only)
        let projections = make_projections(&[
            (1, "3450.00"),
            (2, "3967.50"),
            (3, "4562.63"),
            (4, "5247.02"),
            (5, "6034.07"),
        ]);
        let lines = calculate_personnel_cost_lines(&projections, dec!(1.0), 13, 24).unwrap();
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
        for line in &lines[2..5] {
            assert!(!line.is_active);
            assert_eq!(line.annual_cost_eur, Decimal::ZERO);
        }
    }

    #[test]
    fn test_calc_03_expert_year_1_only_fte_04() {
        let projections = make_projections(&[(1, "3450.00")]);
        let lines = calculate_personnel_cost_lines(&projections, dec!(0.4), 1, 12).unwrap();
        // 3450 × 12 × 0.4 = 16,560
        assert_eq!(lines[0].annual_cost_eur, dec!(16560.00));
    }

    #[test]
    fn test_calc_03_partial_year_proration() {
        // Role starts month 9 (Year 1) and ends month 20 (Year 2): 4 months in Year 1, 8 in Year 2.
        let projections = make_projections(&[(1, "3000"), (2, "3450")]);
        let lines = calculate_personnel_cost_lines(&projections, dec!(1.0), 9, 20).unwrap();
        assert_eq!(lines[0].active_months, 4);
        assert_eq!(lines[0].annual_cost_eur, dec!(3000) * dec!(4));
        assert_eq!(lines[1].active_months, 8);
        assert_eq!(lines[1].annual_cost_eur, dec!(3450) * dec!(8));
    }

    #[test]
    fn test_calc_03_invalid_fte_zero() {
        let projections = make_projections(&[(1, "3000")]);
        let result = calculate_personnel_cost_lines(&projections, Decimal::ZERO, 1, 12);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_FTE"));
    }

    #[test]
    fn test_calc_03_invalid_fte_over_one() {
        let projections = make_projections(&[(1, "3000")]);
        let result = calculate_personnel_cost_lines(&projections, dec!(1.1), 1, 12);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_FTE"));
    }

    #[test]
    fn test_calc_03_start_after_end_returns_error() {
        let projections = make_projections(&[(1, "3000"), (2, "3450")]);
        let result = calculate_personnel_cost_lines(&projections, dec!(1.0), 12, 1);
        assert!(
            matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_MONTH_RANGE")
        );
    }

    #[test]
    fn test_calc_03_month_out_of_range() {
        let projections = make_projections(&[(1, "3000"), (2, "3450")]);
        let result = calculate_personnel_cost_lines(&projections, dec!(1.0), 1, 25);
        assert!(
            matches!(result, Err(AppError::Calculation { code, .. }) if code == "MONTH_OUT_OF_RANGE")
        );
    }

    #[test]
    fn test_calc_03_non_contiguous_years_via_full_span() {
        let projections = make_projections(&[(1, "3000"), (2, "3450"), (3, "3967.50")]);
        let lines = calculate_personnel_cost_lines(&projections, dec!(1.0), 1, 36).unwrap();
        assert!(lines[0].is_active);
        assert!(lines[1].is_active);
        assert!(lines[2].is_active);
    }

    // ── Person-Months rounding (matches the EU Funding & Tenders Portal) ──

    #[test]
    fn test_calc_03_person_months_round_half_up() {
        // 3 active months at FTE 0.79 = 2.37 PM exactly -> rounds to 2.4,
        // and cost is computed from the *rounded* PM, not 2.37.
        let projections = make_projections(&[(1, "1000")]);
        let lines = calculate_personnel_cost_lines(&projections, dec!(0.79), 1, 3).unwrap();
        assert_eq!(lines[0].active_months, 3);
        assert_eq!(lines[0].person_months, dec!(2.4));
        assert_eq!(lines[0].annual_cost_eur, dec!(1000) * dec!(2.4));
        // Not the unrounded figure.
        assert_ne!(lines[0].annual_cost_eur, dec!(1000) * dec!(2.37));
    }

    #[test]
    fn test_calc_03_person_months_exact_half_rounds_up() {
        // 2 months at FTE 0.225 = 0.45 PM exactly -> the second decimal is
        // exactly 5, which must round the first decimal *up* (to 0.5), not
        // down and not to even.
        let projections = make_projections(&[(1, "1000")]);
        let lines = calculate_personnel_cost_lines(&projections, dec!(0.225), 1, 2).unwrap();
        assert_eq!(lines[0].person_months, dec!(0.5));
    }

    #[test]
    fn test_calc_03_person_months_below_half_rounds_down() {
        // 3 months at FTE 0.78 = 2.34 PM exactly.
        let projections = make_projections(&[(1, "1000")]);
        let lines = calculate_personnel_cost_lines(&projections, dec!(0.78), 1, 3).unwrap();
        // 3 x 0.78 = 2.34 -> rounds down to 2.3.
        assert_eq!(lines[0].person_months, dec!(2.3));
    }

    #[test]
    fn test_calc_03_tiny_fte_can_round_to_zero_pm_without_erroring() {
        // A pathologically small FTE on a single active month can
        // legitimately round to 0.0 PM (and so €0) -- that's what the
        // portal would show too, not an internal calculation bug.
        let projections = make_projections(&[(1, "1000")]);
        let lines = calculate_personnel_cost_lines(&projections, dec!(0.01), 1, 1).unwrap();
        assert!(lines[0].is_active);
        assert_eq!(lines[0].person_months, Decimal::ZERO);
        assert_eq!(lines[0].annual_cost_eur, Decimal::ZERO);
    }

    // ── CALC-04 tests ──

    #[test]
    fn test_calc_04_sum_across_roles() {
        let role1_lines = vec![
            PersonnelCostLine {
                year: 1,
                is_active: true,
                active_months: 12,
                person_months: dec!(12),
                monthly_salary_eur: dec!(5000),
                annual_cost_eur: dec!(42000),
            },
            PersonnelCostLine {
                year: 2,
                is_active: true,
                active_months: 12,
                person_months: dec!(12),
                monthly_salary_eur: dec!(6000),
                annual_cost_eur: dec!(50400),
            },
        ];
        let role2_lines = vec![
            PersonnelCostLine {
                year: 1,
                is_active: false,
                active_months: 0,
                person_months: Decimal::ZERO,
                monthly_salary_eur: dec!(3000),
                annual_cost_eur: Decimal::ZERO,
            },
            PersonnelCostLine {
                year: 2,
                is_active: true,
                active_months: 12,
                person_months: dec!(12),
                monthly_salary_eur: dec!(3450),
                annual_cost_eur: dec!(41400),
            },
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
            PersonnelCostLine {
                year: 1,
                is_active: true,
                active_months: 12,
                person_months: dec!(12),
                monthly_salary_eur: dec!(5000),
                annual_cost_eur: dec!(42000),
            },
            PersonnelCostLine {
                year: 2,
                is_active: true,
                active_months: 12,
                person_months: dec!(12),
                monthly_salary_eur: dec!(6000),
                annual_cost_eur: dec!(50400),
            },
            PersonnelCostLine {
                year: 3,
                is_active: false,
                active_months: 0,
                person_months: Decimal::ZERO,
                monthly_salary_eur: dec!(7200),
                annual_cost_eur: Decimal::ZERO,
            },
        ];
        let totals = aggregate_personnel_costs(&[role_lines], 3).unwrap();
        let sum_of_years: Decimal = totals.by_year.iter().map(|y| y.amount_eur).sum();
        assert_eq!(totals.total, sum_of_years);
    }

    // ── CALC-20a tests ──

    #[test]
    fn test_calc_20a_single_wp_gets_full_cost() {
        let projections = make_projections(&[(1, "3000")]);
        let wps = vec![(1u8, 1u32, 12u32)];
        let result = allocate_personnel_cost_by_wp(&projections, dec!(1.0), 1, 12, &wps).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].work_package_id, 1);
        assert_eq!(result[0].amount_eur, dec!(3000) * dec!(12));
    }

    #[test]
    fn test_calc_20a_split_across_two_wps_by_month_count() {
        // Role spans months 1-12. WP1 = months 1-8, WP2 = months 9-12.
        let projections = make_projections(&[(1, "1200")]);
        let wps = vec![(1u8, 1u32, 8u32), (2u8, 9u32, 12u32)];
        let result = allocate_personnel_cost_by_wp(&projections, dec!(1.0), 1, 12, &wps).unwrap();
        let wp1 = result.iter().find(|w| w.work_package_id == 1).unwrap();
        let wp2 = result.iter().find(|w| w.work_package_id == 2).unwrap();
        assert_eq!(wp1.amount_eur, dec!(1200) * dec!(8));
        assert_eq!(wp2.amount_eur, dec!(1200) * dec!(4));
    }

    #[test]
    fn test_calc_20a_overlapping_wps_split_evenly() {
        // A single month covered by two overlapping WPs splits 50/50.
        let projections = make_projections(&[(1, "1000")]);
        let wps = vec![(1u8, 1u32, 12u32), (2u8, 1u32, 12u32)];
        let result = allocate_personnel_cost_by_wp(&projections, dec!(1.0), 1, 12, &wps).unwrap();
        let wp1 = result.iter().find(|w| w.work_package_id == 1).unwrap();
        let wp2 = result.iter().find(|w| w.work_package_id == 2).unwrap();
        assert_eq!(wp1.amount_eur, wp2.amount_eur);
        assert_eq!(wp1.amount_eur + wp2.amount_eur, dec!(1000) * dec!(12));
    }

    #[test]
    fn test_calc_20a_orphan_months_not_attributed() {
        // Role spans months 1-12 but only WP1 (months 1-6) is defined.
        let projections = make_projections(&[(1, "1200")]);
        let wps = vec![(1u8, 1u32, 6u32)];
        let result = allocate_personnel_cost_by_wp(&projections, dec!(1.0), 1, 12, &wps).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].amount_eur, dec!(1200) * dec!(6));
    }

    #[test]
    fn test_calc_20a_uses_rounded_pm_and_matches_calc_03_total() {
        // Same role as test_calc_03_person_months_round_half_up (3 months,
        // FTE 0.79, 2.37 PM raw -> 2.4 rounded), but split evenly across two
        // fully-overlapping WPs. Each WP's exact PM share is 2.37/2 = 1.185
        // -- the WP split must still be based on the year's *rounded* 2.4 PM
        // (matching CALC-03 exactly), not on 2.37, and must split it exactly
        // in half since both WPs have an equal underlying share.
        let projections = make_projections(&[(1, "1000")]);
        let wps = vec![(1u8, 1u32, 3u32), (2u8, 1u32, 3u32)];
        let result = allocate_personnel_cost_by_wp(&projections, dec!(0.79), 1, 3, &wps).unwrap();

        let role_lines = calculate_personnel_cost_lines(&projections, dec!(0.79), 1, 3).unwrap();
        let role_total = role_lines[0].annual_cost_eur;
        assert_eq!(role_total, dec!(1000) * dec!(2.4));

        let wp1 = result.iter().find(|w| w.work_package_id == 1).unwrap();
        let wp2 = result.iter().find(|w| w.work_package_id == 2).unwrap();
        assert_eq!(wp1.amount_eur, dec!(1200));
        assert_eq!(wp2.amount_eur, dec!(1200));
        // No reconciliation drift: the WP split sums exactly to the same
        // rounded-PM total CALC-03 computed for this role.
        assert_eq!(wp1.amount_eur + wp2.amount_eur, role_total);
    }

    #[test]
    fn test_calc_20a_uneven_split_sums_exactly_despite_repeating_decimal() {
        // WP1 gets 8 of 12 months, WP2 gets 4 -- 4/12 has no terminating
        // decimal representation, so a naive per-WP division could leak a
        // sub-cent residual. The split must still sum exactly to the total.
        let projections = make_projections(&[(1, "1200")]);
        let wps = vec![(1u8, 1u32, 8u32), (2u8, 9u32, 12u32)];
        let result = allocate_personnel_cost_by_wp(&projections, dec!(1.0), 1, 12, &wps).unwrap();
        let wp1 = result.iter().find(|w| w.work_package_id == 1).unwrap();
        let wp2 = result.iter().find(|w| w.work_package_id == 2).unwrap();
        assert_eq!(wp1.amount_eur, dec!(9600));
        assert_eq!(wp2.amount_eur, dec!(4800));
        assert_eq!(wp1.amount_eur + wp2.amount_eur, dec!(1200) * dec!(12));
    }
}
