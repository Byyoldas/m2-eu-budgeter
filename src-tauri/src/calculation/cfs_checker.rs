//! CALC-18 — CFS Threshold Check
//!
//! Monitors the running project total and determines whether a Certificate on
//! Financial Statements (CFS) is required, present, or needs to be requested.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::domain::dto::CfsStatus;
use crate::error::{AppError, calc_error};

/// The ERC-mandated threshold above which a CFS is required.
/// Fixed at €430,000 — not configurable.
pub const CFS_THRESHOLD_EUR: Decimal = Decimal::from_parts(430000, 0, 0, false, 0);

/// CALC-18: Determine the CFS requirement status.
///
/// # Arguments
/// * `requested_eu_contribution` — Output of CALC-17 (live total).
/// * `has_cfs_item` — True if a `is_cfs_item = true` entry exists in the C3 list.
/// * `warning_dismissed` — True if the user explicitly clicked "Remind Me Later".
pub fn check_cfs_threshold(
    requested_eu_contribution: Decimal,
    has_cfs_item: bool,
    warning_dismissed: bool,
) -> Result<CfsCheckResult, AppError> {
    if requested_eu_contribution < Decimal::ZERO {
        return Err(calc_error(
            "INTERNAL_CALC_ERROR",
            "Requested EU contribution is negative. This is a bug.",
        ));
    }

    let threshold = Decimal::from(430_000u32);
    let threshold_exceeded = requested_eu_contribution > threshold;

    if !threshold_exceeded {
        return Ok(CfsCheckResult {
            cfs_status: CfsStatus::NotRequired,
            threshold_exceeded: false,
            warning_active: false,
            prompt_required: false,
        });
    }

    if has_cfs_item {
        return Ok(CfsCheckResult {
            cfs_status: CfsStatus::RequiredAndPresent,
            threshold_exceeded: true,
            warning_active: false,
            prompt_required: false,
        });
    }

    if warning_dismissed {
        return Ok(CfsCheckResult {
            cfs_status: CfsStatus::RequiredButDismissed,
            threshold_exceeded: true,
            warning_active: true,
            prompt_required: false,
        });
    }

    Ok(CfsCheckResult {
        cfs_status: CfsStatus::RequiredAndUnaddressed,
        threshold_exceeded: true,
        warning_active: true,
        prompt_required: true,
    })
}

/// Output of CALC-18.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfsCheckResult {
    pub cfs_status: CfsStatus,
    pub threshold_exceeded: bool,
    pub warning_active: bool,
    pub prompt_required: bool,
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_calc_18_below_threshold_not_required() {
        let result = check_cfs_threshold(dec!(425000), false, false).unwrap();
        assert_eq!(result.cfs_status, CfsStatus::NotRequired);
        assert!(!result.threshold_exceeded);
        assert!(!result.warning_active);
        assert!(!result.prompt_required);
    }

    #[test]
    fn test_calc_18_exactly_at_threshold_not_required() {
        // Threshold is > 430,000 (strict), so exactly 430,000 is not required
        let result = check_cfs_threshold(dec!(430000), false, false).unwrap();
        assert_eq!(result.cfs_status, CfsStatus::NotRequired);
        assert!(!result.threshold_exceeded);
    }

    #[test]
    fn test_calc_18_one_cent_over_threshold_required_unaddressed() {
        let result = check_cfs_threshold(dec!(430000.01), false, false).unwrap();
        assert_eq!(result.cfs_status, CfsStatus::RequiredAndUnaddressed);
        assert!(result.threshold_exceeded);
        assert!(result.warning_active);
        assert!(result.prompt_required);
    }

    #[test]
    fn test_calc_18_over_threshold_with_cfs_item_compliant() {
        let result = check_cfs_threshold(dec!(462000), true, false).unwrap();
        assert_eq!(result.cfs_status, CfsStatus::RequiredAndPresent);
        assert!(result.threshold_exceeded);
        assert!(!result.warning_active);
        assert!(!result.prompt_required);
    }

    #[test]
    fn test_calc_18_over_threshold_dismissed_badge_shown() {
        let result = check_cfs_threshold(dec!(460000), false, true).unwrap();
        assert_eq!(result.cfs_status, CfsStatus::RequiredButDismissed);
        assert!(result.threshold_exceeded);
        assert!(result.warning_active);
        assert!(!result.prompt_required); // dismissed = no prompt, just badge
    }

    #[test]
    fn test_calc_18_adding_cfs_item_clears_warning() {
        // Scenario: budget was 460k (dismissed), user adds CFS item
        let before = check_cfs_threshold(dec!(460000), false, true).unwrap();
        assert_eq!(before.cfs_status, CfsStatus::RequiredButDismissed);
        let after = check_cfs_threshold(dec!(472000), true, true).unwrap();
        assert_eq!(after.cfs_status, CfsStatus::RequiredAndPresent);
        assert!(!after.warning_active);
    }

    #[test]
    fn test_calc_18_budget_drops_below_threshold_clears_status() {
        // User had CFS, but removed items and budget is back below threshold
        let result = check_cfs_threshold(dec!(400000), true, false).unwrap();
        assert_eq!(result.cfs_status, CfsStatus::NotRequired);
    }
}
