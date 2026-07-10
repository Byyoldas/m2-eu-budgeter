//! CALC-05 — Equipment Eligible Depreciation
//! CALC-06 — Total Equipment Cost (Category C2)
//!
//! Calculates how much of each equipment item's purchase cost can be claimed,
//! applying the EU depreciation formula with a hard cap at the grant-attributable value.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::error::{AppError, calc_error};

// ─── Output Types ─────────────────────────────────────────────────────────────

/// Full depreciation breakdown for one equipment item (CALC-05 output).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepreciationResult {
    /// (cost / lifetime) × usage% × months — before cap.
    #[serde(with = "rust_decimal::serde::str")]
    pub theoretical_eligible_eur: Decimal,
    /// cost × usage% — the maximum claimable amount.
    #[serde(with = "rust_decimal::serde::str")]
    pub maximum_eligible_eur: Decimal,
    /// True when theoretical ≥ maximum (item is fully depreciated within grant).
    pub is_capped: bool,
    /// Final claimable amount = min(theoretical, maximum).
    #[serde(with = "rust_decimal::serde::str")]
    pub eligible_depreciation_eur: Decimal,
}

// ─── CALC-05 ─────────────────────────────────────────────────────────────────

/// CALC-05: Calculate the eligible depreciation amount for one equipment item.
///
/// Algorithm:
///   theoretical = (cost / lifetime_months) × (usage_pct / 100) × grant_usage_months
///   maximum     = cost × (usage_pct / 100)
///   eligible    = min(theoretical, maximum)
///
/// # Arguments
/// * `purchase_cost_eur` — Total purchase price. Must be > 0.
/// * `useful_lifetime_months` — Standard economic lifetime in months. Must be ≥ 1.
/// * `grant_usage_pct` — Share of use for grant (0–100]. Stored as percent.
/// * `grant_usage_months` — Months the item is used during the grant. Must be ≥ 1.
pub fn calculate_depreciation(
    purchase_cost_eur: Decimal,
    useful_lifetime_months: u32,
    grant_usage_pct: Decimal,
    grant_usage_months: u32,
) -> Result<DepreciationResult, AppError> {
    // Input validation
    if purchase_cost_eur <= Decimal::ZERO {
        return Err(calc_error(
            "INVALID_PURCHASE_COST",
            "Purchase cost must be greater than zero.",
        ));
    }
    if useful_lifetime_months < 1 {
        return Err(calc_error(
            "INVALID_LIFETIME",
            "Useful lifetime must be at least 1 month.",
        ));
    }
    if grant_usage_pct <= Decimal::ZERO || grant_usage_pct > Decimal::ONE_HUNDRED {
        return Err(calc_error(
            "INVALID_USAGE_PCT",
            "Grant usage percentage must be between 0% (exclusive) and 100% (inclusive).",
        ));
    }
    if grant_usage_months < 1 {
        return Err(calc_error(
            "INVALID_USAGE_MONTHS",
            "Months used for the grant must be at least 1.",
        ));
    }

    let usage_fraction = grant_usage_pct / Decimal::ONE_HUNDRED;
    let lifetime = Decimal::from(useful_lifetime_months);
    let months = Decimal::from(grant_usage_months);

    // Step 1: Theoretical depreciation
    let theoretical = (purchase_cost_eur / lifetime) * usage_fraction * months;

    // Step 2: Maximum (cap)
    let maximum = purchase_cost_eur * usage_fraction;

    // Step 3: Apply cap
    let is_capped = theoretical >= maximum;
    let eligible = if is_capped { maximum } else { theoretical };

    // Post-condition: eligible must be > 0
    if eligible <= Decimal::ZERO {
        return Err(calc_error(
            "INTERNAL_CALC_ERROR",
            "Eligible depreciation is zero or negative. This is a bug.",
        ));
    }
    // Post-condition: eligible must not exceed maximum or purchase cost
    if eligible > maximum || eligible > purchase_cost_eur {
        return Err(calc_error(
            "INTERNAL_CALC_ERROR",
            "Eligible depreciation exceeds the maximum allowed. This is a bug.",
        ));
    }

    Ok(DepreciationResult {
        theoretical_eligible_eur: theoretical,
        maximum_eligible_eur: maximum,
        is_capped,
        eligible_depreciation_eur: eligible,
    })
}

// ─── CALC-06 ─────────────────────────────────────────────────────────────────

/// CALC-06: Sum all eligible depreciation amounts to produce Category C2 total.
pub fn aggregate_equipment_costs(results: &[DepreciationResult]) -> Result<Decimal, AppError> {
    let total: Decimal = results.iter().map(|r| r.eligible_depreciation_eur).sum();
    if total < Decimal::ZERO {
        return Err(calc_error(
            "INTERNAL_CALC_ERROR",
            "Equipment total is negative. This is a bug.",
        ));
    }
    Ok(total)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    // ── CALC-05 tests ──

    #[test]
    fn test_calc_05_laptop_capped() {
        // Cost €2,500, lifetime 48m, usage 100%, used 55m → theoretical €2,864.58, cap €2,500
        let result = calculate_depreciation(dec!(2500), 48, dec!(100), 55).unwrap();
        assert!(result.is_capped);
        assert_eq!(result.eligible_depreciation_eur, dec!(2500));
        assert_eq!(result.maximum_eligible_eur, dec!(2500));
        let theoretical = result.theoretical_eligible_eur.round_dp(2);
        assert_eq!(theoretical, dec!(2864.58));
    }

    #[test]
    fn test_calc_05_audio_recorder_not_capped() {
        // Cost €60, lifetime 60m, usage 100%, used 36m → theoretical €36, cap €60
        let result = calculate_depreciation(dec!(60), 60, dec!(100), 36).unwrap();
        assert!(!result.is_capped);
        assert_eq!(result.eligible_depreciation_eur, dec!(36));
        assert_eq!(result.maximum_eligible_eur, dec!(60));
        assert_eq!(result.theoretical_eligible_eur, dec!(36));
    }

    #[test]
    fn test_calc_05_laptop_80pct_usage_capped() {
        // Cost €2,500, lifetime 48m, usage 80%, used 55m → cap at €2,500 × 80% = €2,000
        let result = calculate_depreciation(dec!(2500), 48, dec!(80), 55).unwrap();
        assert!(result.is_capped);
        assert_eq!(result.eligible_depreciation_eur, dec!(2000));
        assert_eq!(result.maximum_eligible_eur, dec!(2000));
    }

    #[test]
    fn test_calc_05_server_partial_use_not_capped() {
        // Cost €8,000, lifetime 60m, usage 50%, used 24m
        // theoretical = (8000/60) × 0.50 × 24 = 1600
        // maximum = 8000 × 0.50 = 4000
        let result = calculate_depreciation(dec!(8000), 60, dec!(50), 24).unwrap();
        assert!(!result.is_capped);
        assert_eq!(result.eligible_depreciation_eur, dec!(1600));
        assert_eq!(result.maximum_eligible_eur, dec!(4000));
    }

    #[test]
    fn test_calc_05_exactly_at_lifetime_boundary() {
        // used months = lifetime → theoretical equals maximum (just reaches cap)
        let result = calculate_depreciation(dec!(1200), 12, dec!(100), 12).unwrap();
        // theoretical = (1200/12) × 1.0 × 12 = 1200
        // maximum = 1200 × 1.0 = 1200
        // is_capped because theoretical >= maximum
        assert!(result.is_capped);
        assert_eq!(result.eligible_depreciation_eur, dec!(1200));
    }

    #[test]
    fn test_calc_05_one_month_usage() {
        let result = calculate_depreciation(dec!(2400), 24, dec!(100), 1).unwrap();
        // theoretical = (2400/24) × 1.0 × 1 = 100
        assert_eq!(result.eligible_depreciation_eur, dec!(100));
        assert!(!result.is_capped);
    }

    #[test]
    fn test_calc_05_zero_cost_returns_error() {
        let result = calculate_depreciation(Decimal::ZERO, 48, dec!(100), 36);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_PURCHASE_COST"));
    }

    #[test]
    fn test_calc_05_negative_cost_returns_error() {
        let result = calculate_depreciation(dec!(-100), 48, dec!(100), 36);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_PURCHASE_COST"));
    }

    #[test]
    fn test_calc_05_zero_lifetime_returns_error() {
        let result = calculate_depreciation(dec!(2500), 0, dec!(100), 36);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_LIFETIME"));
    }

    #[test]
    fn test_calc_05_zero_usage_pct_returns_error() {
        let result = calculate_depreciation(dec!(2500), 48, Decimal::ZERO, 36);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_USAGE_PCT"));
    }

    #[test]
    fn test_calc_05_usage_pct_over_100_returns_error() {
        let result = calculate_depreciation(dec!(2500), 48, dec!(101), 36);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_USAGE_PCT"));
    }

    #[test]
    fn test_calc_05_zero_usage_months_returns_error() {
        let result = calculate_depreciation(dec!(2500), 48, dec!(100), 0);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_USAGE_MONTHS"));
    }

    // ── CALC-06 tests ──

    #[test]
    fn test_calc_06_sum_multiple_items() {
        let r1 = DepreciationResult {
            theoretical_eligible_eur: dec!(2864.58),
            maximum_eligible_eur: dec!(2500),
            is_capped: true,
            eligible_depreciation_eur: dec!(2500),
        };
        let r2 = DepreciationResult {
            theoretical_eligible_eur: dec!(36),
            maximum_eligible_eur: dec!(60),
            is_capped: false,
            eligible_depreciation_eur: dec!(36),
        };
        let total = aggregate_equipment_costs(&[r1, r2]).unwrap();
        assert_eq!(total, dec!(2536));
    }

    #[test]
    fn test_calc_06_empty_items_gives_zero() {
        let total = aggregate_equipment_costs(&[]).unwrap();
        assert_eq!(total, Decimal::ZERO);
    }
}
