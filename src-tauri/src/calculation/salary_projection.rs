//! CALC-01 — Currency Conversion (TRY → EUR)
//! CALC-02 — Salary Projection Chain
//!
//! Converts a monthly TRY salary to EUR and computes the year-by-year
//! projected monthly salary for all project years, applying compounding inflation.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::error::{AppError, calc_error};

// ─── Output Types ─────────────────────────────────────────────────────────────

/// One year's projected monthly salary in EUR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalaryProjection {
    /// Project year number (1-indexed).
    pub year: u8,
    /// Projected monthly salary in EUR for this year.
    #[serde(with = "rust_decimal::serde::str")]
    pub projected_monthly_eur: Decimal,
}

// ─── CALC-01 ─────────────────────────────────────────────────────────────────

/// CALC-01: Convert a monthly salary in TRY to EUR.
///
/// # Arguments
/// * `monthly_salary_try` — Current monthly gross salary in Turkish Lira. Must be > 0.
/// * `try_eur_rate` — TRY per 1 EUR (e.g. 50.62). Must be > 0.
///
/// # Returns
/// The monthly salary in EUR, at full `Decimal` precision (no rounding).
pub fn convert_try_to_eur(
    monthly_salary_try: Decimal,
    try_eur_rate: Decimal,
) -> Result<Decimal, AppError> {
    if monthly_salary_try <= Decimal::ZERO {
        return Err(calc_error(
            "INVALID_SALARY_TRY",
            "Monthly salary must be greater than zero.",
        ));
    }
    if try_eur_rate <= Decimal::ZERO {
        return Err(calc_error(
            "INVALID_EXCHANGE_RATE",
            "Exchange rate must be a positive number greater than zero.",
        ));
    }
    Ok(monthly_salary_try / try_eur_rate)
}

// ─── CALC-02 ─────────────────────────────────────────────────────────────────

/// CALC-02: Project a monthly salary (EUR) forward across all project years,
/// applying compounding annual inflation.
///
/// The base salary (from CALC-01) represents "today's" salary (Year 0 conceptually).
/// Year 1 already includes one full cycle of inflation:
///   Year 1 = base × (1 + rate)
///   Year N = Year N-1 × (1 + rate)
///
/// # Arguments
/// * `base_monthly_eur` — Output of CALC-01. Must be > 0.
/// * `inflation_rate_pct` — Annual salary growth as a percentage (e.g. 20.0 for 20%).
///   Stored as percent; this function converts internally. Range: [0, 100].
/// * `duration_years` — Total number of project years. Range: 1–7.
///
/// # Returns
/// A vector of `SalaryProjection`, one per project year, in ascending year order.
pub fn project_salary_chain(
    base_monthly_eur: Decimal,
    inflation_rate_pct: Decimal,
    duration_years: u8,
) -> Result<Vec<SalaryProjection>, AppError> {
    if base_monthly_eur <= Decimal::ZERO {
        return Err(calc_error(
            "INVALID_BASE_SALARY",
            "Base salary must be greater than zero.",
        ));
    }
    if inflation_rate_pct < Decimal::ZERO || inflation_rate_pct > Decimal::ONE_HUNDRED {
        return Err(calc_error(
            "INVALID_INFLATION_RATE",
            "Inflation rate must be between 0% and 100%.",
        ));
    }
    if duration_years < 1 || duration_years > 7 {
        return Err(calc_error(
            "INVALID_DURATION",
            "Project duration must be between 1 and 7 years.",
        ));
    }

    // Convert percentage to fraction: 20.0% → 0.20
    let inflation_fraction = inflation_rate_pct / Decimal::ONE_HUNDRED;
    let multiplier = Decimal::ONE + inflation_fraction;

    let mut projections = Vec::with_capacity(duration_years as usize);
    let mut current = base_monthly_eur;

    for year in 1..=duration_years {
        current = current * multiplier;
        projections.push(SalaryProjection {
            year,
            projected_monthly_eur: current,
        });
    }

    // Post-condition: all projections must be ≥ base (inflation_rate ≥ 0).
    for p in &projections {
        if p.projected_monthly_eur < base_monthly_eur {
            return Err(calc_error(
                "INTERNAL_CALC_ERROR",
                "Salary projection is less than base salary. This is a bug — please report it.",
            ));
        }
    }

    Ok(projections)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    // ── CALC-01 tests ──

    #[test]
    fn test_calc_01_pi_salary_conversion() {
        // 227,900 TRY ÷ 50.62 TRY/EUR ≈ €4,502.17
        let result = convert_try_to_eur(dec!(227900), dec!(50.62)).unwrap();
        // Check to 2 decimal places
        let rounded = result.round_dp(2);
        assert_eq!(rounded, dec!(4502.17));
    }

    #[test]
    fn test_calc_01_postdoc_salary_conversion() {
        // 151,860 TRY ÷ 50.62 ≈ €3,000.00
        let result = convert_try_to_eur(dec!(151860), dec!(50.62)).unwrap();
        let rounded = result.round_dp(2);
        assert_eq!(rounded, dec!(3000.00));
    }

    #[test]
    fn test_calc_01_zero_salary_returns_error() {
        let result = convert_try_to_eur(dec!(0), dec!(50.62));
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_SALARY_TRY"));
    }

    #[test]
    fn test_calc_01_negative_salary_returns_error() {
        let result = convert_try_to_eur(dec!(-1000), dec!(50.62));
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_SALARY_TRY"));
    }

    #[test]
    fn test_calc_01_zero_rate_returns_error() {
        let result = convert_try_to_eur(dec!(100000), dec!(0));
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_EXCHANGE_RATE"));
    }

    #[test]
    fn test_calc_01_negative_rate_returns_error() {
        let result = convert_try_to_eur(dec!(100000), dec!(-50));
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_EXCHANGE_RATE"));
    }

    // ── CALC-02 tests ──

    #[test]
    fn test_calc_02_pi_projection_5_years_20pct() {
        // Base: €4,502.17, 20% inflation, 5 years
        // Year 1: 4502.17 × 1.20 ≈ 5402.61
        // Year 2: 5402.61 × 1.20 ≈ 6483.13
        // Year 5: compounded
        let base = dec!(4502.17);
        let projections = project_salary_chain(base, dec!(20), 5).unwrap();
        assert_eq!(projections.len(), 5);
        assert_eq!(projections[0].year, 1);
        let y1 = projections[0].projected_monthly_eur.round_dp(2);
        assert_eq!(y1, dec!(5402.60)); // 4502.17 * 1.20 = 5402.604
    }

    #[test]
    fn test_calc_02_zero_inflation_flat_salary() {
        // 0% inflation: all years should equal base salary
        let base = dec!(3000);
        let projections = project_salary_chain(base, dec!(0), 5).unwrap();
        assert_eq!(projections.len(), 5);
        for p in &projections {
            assert_eq!(p.projected_monthly_eur, base);
        }
    }

    #[test]
    fn test_calc_02_1_year_project() {
        let base = dec!(2000);
        let projections = project_salary_chain(base, dec!(15), 1).unwrap();
        assert_eq!(projections.len(), 1);
        assert_eq!(projections[0].year, 1);
        assert_eq!(projections[0].projected_monthly_eur, dec!(2300)); // 2000 * 1.15
    }

    #[test]
    fn test_calc_02_7_year_project() {
        let projections = project_salary_chain(dec!(1000), dec!(10), 7).unwrap();
        assert_eq!(projections.len(), 7);
        assert_eq!(projections[6].year, 7);
    }

    #[test]
    fn test_calc_02_compounding_is_applied_each_year() {
        // Year 2 must be Year 1 * multiplier, not base * multiplier^2 (same result, but checks chain)
        let projections = project_salary_chain(dec!(1000), dec!(10), 3).unwrap();
        let y1 = projections[0].projected_monthly_eur; // 1100
        let y2 = projections[1].projected_monthly_eur; // 1210
        let y3 = projections[2].projected_monthly_eur; // 1331
        assert_eq!(y1, dec!(1100));
        assert_eq!(y2, dec!(1210));
        assert_eq!(y3, dec!(1331));
    }

    #[test]
    fn test_calc_02_invalid_inflation_over_100() {
        let result = project_salary_chain(dec!(1000), dec!(101), 5);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_INFLATION_RATE"));
    }

    #[test]
    fn test_calc_02_negative_inflation_returns_error() {
        let result = project_salary_chain(dec!(1000), dec!(-5), 5);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_INFLATION_RATE"));
    }

    #[test]
    fn test_calc_02_zero_duration_returns_error() {
        let result = project_salary_chain(dec!(1000), dec!(10), 0);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_DURATION"));
    }

    #[test]
    fn test_calc_02_duration_over_7_returns_error() {
        let result = project_salary_chain(dec!(1000), dec!(10), 8);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_DURATION"));
    }

    #[test]
    fn test_calc_02_projections_ascending_order() {
        let projections = project_salary_chain(dec!(1000), dec!(5), 5).unwrap();
        for i in 0..projections.len() {
            assert_eq!(projections[i].year, (i + 1) as u8);
            if i > 0 {
                assert!(projections[i].projected_monthly_eur > projections[i - 1].projected_monthly_eur);
            }
        }
    }
}
