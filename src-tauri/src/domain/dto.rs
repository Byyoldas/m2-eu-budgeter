//! Data Transfer Objects (DTOs) for the IPC boundary.
//!
//! DTOs are what the TypeScript frontend sends in and receives back.
//! They are strictly separate from the domain entities:
//! - Inputs DTOs: what the frontend sends to create/update entities.
//! - Output DTOs: what the backend returns after every mutation.
//!
//! All monetary Decimal values are serialised as strings for JSON safety.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::domain::entities::{RoleType, TripType};

// ─── Input DTOs ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfigDto {
    pub project_title: String,
    pub pi_name: String,
    pub call_reference: String,
    pub duration_years: u8,
    pub work_package_count: u8,
    pub work_package_names: Vec<Option<String>>,
    #[serde(default)]
    pub work_package_start_months: Vec<u32>,
    #[serde(default)]
    pub work_package_end_months: Vec<u32>,
    #[serde(with = "rust_decimal::serde::str")]
    pub default_inflation_rate_pct: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub try_eur_rate: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub indirect_cost_rate_pct: Decimal,
    pub rate_version_id: String,
    pub call_opening_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonnelRoleInputDto {
    pub role_label: String,
    pub role_type: RoleType,
    #[serde(with = "rust_decimal::serde::str")]
    pub current_monthly_salary_try: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub fte_fraction: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub inflation_rate_pct: Decimal,
    pub start_month: u32,
    pub end_month: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentItemInputDto {
    pub name: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub purchase_cost_eur: Decimal,
    pub useful_lifetime_months: u32,
    #[serde(with = "rust_decimal::serde::str")]
    pub grant_usage_pct: Decimal,
    pub grant_usage_months: u32,
    pub work_package_id: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TripInputDto {
    pub name: String,
    pub trip_type: TripType,
    pub number_of_instances: u32,
    pub work_package_ids: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtherCostInputDto {
    pub name: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount_eur: Decimal,
    pub notes: Option<String>,
    pub work_package_ids: Vec<u8>,
}

// ─── Output / Result DTOs ─────────────────────────────────────────────────────

/// A Work Package's share of some cost, used in per-WP breakdowns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WpCostAmountDto {
    pub work_package_id: u8,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount_eur: Decimal,
}

/// Per-role annual cost breakdown included in the detailed dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleCostLineDto {
    pub year: u8,
    pub is_active: bool,
    /// Number of months (0-12) of the role's Start/End Month period that fall in this year.
    pub active_months: u8,
    #[serde(with = "rust_decimal::serde::str")]
    pub monthly_salary_eur: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub annual_cost_eur: Decimal,
}

/// Complete cost data for one personnel role (for expandable dashboard rows).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonnelRoleDetailDto {
    pub id: Uuid,
    pub role_label: String,
    pub role_type: RoleType,
    /// Current monthly gross salary in TRY (raw input, exposed for exports that
    /// need to rebuild the salary-projection formula rather than a static total).
    #[serde(with = "rust_decimal::serde::str")]
    pub current_monthly_salary_try: Decimal,
    /// Per-role annual salary inflation rate (%), raw input (see above).
    #[serde(with = "rust_decimal::serde::str")]
    pub inflation_rate_pct: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub fte_fraction: Decimal,
    pub start_month: u32,
    pub end_month: u32,
    pub cost_lines: Vec<RoleCostLineDto>,
    #[serde(with = "rust_decimal::serde::str")]
    pub total_cost_eur: Decimal,
    pub wp_breakdown: Vec<WpCostAmountDto>,
}

/// Live preview returned by `preview_role_cost` (shown while typing, before save).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleCostPreviewDto {
    /// Base monthly salary in EUR after TRY→EUR conversion.
    #[serde(with = "rust_decimal::serde::str")]
    pub base_monthly_eur: Decimal,
    pub cost_lines: Vec<RoleCostLineDto>,
    #[serde(with = "rust_decimal::serde::str")]
    pub total_cost_eur: Decimal,
    pub wp_breakdown: Vec<WpCostAmountDto>,
}

/// Depreciation result for one equipment item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentItemDetailDto {
    pub id: Uuid,
    pub name: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub theoretical_eligible_eur: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub maximum_eligible_eur: Decimal,
    pub is_capped: bool,
    #[serde(with = "rust_decimal::serde::str")]
    pub eligible_depreciation_eur: Decimal,
}

/// Live preview returned by `preview_equipment_depreciation`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentPreviewDto {
    #[serde(with = "rust_decimal::serde::str")]
    pub theoretical_eligible_eur: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub maximum_eligible_eur: Decimal,
    pub is_capped: bool,
    #[serde(with = "rust_decimal::serde::str")]
    pub eligible_depreciation_eur: Decimal,
}

/// One Other Direct Cost (C3) item, for the expandable dashboard list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtherCostItemDetailDto {
    pub id: Uuid,
    pub name: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount_eur: Decimal,
    pub is_cfs_item: bool,
    pub notes: Option<String>,
    pub work_package_ids: Vec<u8>,
}

/// Cost breakdown for one trip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TripDetailDto {
    pub id: Uuid,
    pub name: String,
    pub work_package_ids: Vec<u8>,
    pub number_of_instances: u32,
    /// Per-instance cost details (None for flat-amount trips that have no breakdown).
    pub flight_cost_per_instance: Option<String>,
    pub accommodation_cost_per_instance: Option<String>,
    pub subsistence_cost_per_instance: Option<String>,
    pub domestic_transport_per_instance: Option<String>,
    #[serde(with = "rust_decimal::serde::str")]
    pub per_instance_total_eur: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub total_trip_cost_eur: Decimal,
}

/// Live preview returned by `preview_trip_cost`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TripCostPreviewDto {
    pub flight_cost_per_instance: Option<String>,
    pub accommodation_cost_per_instance: Option<String>,
    pub subsistence_cost_per_instance: Option<String>,
    pub domestic_transport_per_instance: Option<String>,
    #[serde(with = "rust_decimal::serde::str")]
    pub per_instance_total_eur: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub total_trip_cost_eur: Decimal,
    /// Band label for display (e.g. "4,501–6,000 km → €857").
    pub flight_band_label: Option<String>,
    pub no_flight_applicable: bool,
    /// Country rates for display (accommodation + subsistence per unit).
    pub accommodation_rate_eur: Option<String>,
    pub subsistence_rate_eur: Option<String>,
}

/// The CFS (Certificate on Financial Statements) status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CfsStatus {
    /// Budget ≤ €430,000. No CFS needed.
    NotRequired,
    /// Budget > €430,000 and CFS item is present. Compliant.
    RequiredAndPresent,
    /// Budget > €430,000, no CFS, user dismissed the prompt.
    RequiredButDismissed,
    /// Budget > €430,000, no CFS, user has not been prompted yet (or prompt is open).
    RequiredAndUnaddressed,
}

/// A Work Package's total cost broken down by category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WpBudgetDto {
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

/// The complete budget summary returned after every mutation.
/// This is the primary output of CALC-19.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetSummaryDto {
    pub wp_budgets: Vec<WpBudgetDto>,

    #[serde(with = "rust_decimal::serde::str")]
    pub category_a_total: Decimal,

    #[serde(with = "rust_decimal::serde::str")]
    pub category_b_total: Decimal,

    #[serde(with = "rust_decimal::serde::str")]
    pub category_c1_total: Decimal,

    #[serde(with = "rust_decimal::serde::str")]
    pub category_c2_total: Decimal,

    #[serde(with = "rust_decimal::serde::str")]
    pub category_c3_total: Decimal,

    // Indirect costs (E)
    #[serde(with = "rust_decimal::serde::str")]
    pub indirect_base_total: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub category_e_total: Decimal,

    // Totals
    #[serde(with = "rust_decimal::serde::str")]
    pub total_direct_costs: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub total_eligible_costs: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub requested_eu_contribution: Decimal,

    // CFS status
    pub cfs_status: CfsStatus,
    pub cfs_threshold_exceeded: bool,
    pub cfs_warning_active: bool,
    pub cfs_prompt_required: bool,

    // Detail rows for expandable dashboard sections
    pub role_detail: Vec<PersonnelRoleDetailDto>,
    pub equipment_detail: Vec<EquipmentItemDetailDto>,
    pub trip_detail: Vec<TripDetailDto>,
    pub other_cost_detail: Vec<OtherCostItemDetailDto>,
}

/// Serialisable snapshot of the full project (for file I/O responses).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSnapshotDto {
    pub format_version: String,
    pub created_at: String,
    pub updated_at: String,
    pub project: serde_json::Value,
}

// ─── Tests ────────────────────────────────────────────────────────────────────
//
// These reproduce the exact JSON shapes sent by the TypeScript frontend
// (src/screens/Personnel.tsx, src/screens/OtherCosts.tsx, src/screens/Travel.tsx)
// to guard against IPC boundary mismatches such as enum casing/tagging.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn personnel_role_input_deserializes_frontend_json() {
        for role_type in ["Pi", "Expert", "PostDoc", "PhdStudent", "MscStudent", "Admin"] {
            let json = format!(
                r#"{{
                    "role_label": "PostDoc-1",
                    "role_type": "{role_type}",
                    "current_monthly_salary_try": "151860",
                    "fte_fraction": "1.0",
                    "inflation_rate_pct": "15",
                    "start_month": 1,
                    "end_month": 24
                }}"#
            );
            let result: Result<PersonnelRoleInputDto, _> = serde_json::from_str(&json);
            assert!(result.is_ok(), "role_type '{role_type}' failed to deserialize: {:?}", result.err());
        }
    }

    #[test]
    fn other_cost_input_deserializes_frontend_json() {
        let json = r#"{
            "name": "MAXQDA License",
            "amount_eur": "9870",
            "notes": null,
            "work_package_ids": [1]
        }"#;
        let result: Result<OtherCostInputDto, _> = serde_json::from_str(json);
        assert!(result.is_ok(), "OtherCostInputDto failed to deserialize: {:?}", result.err());
    }

    #[test]
    fn itemized_trip_input_deserializes_frontend_json() {
        let json = r#"{
            "name": "Field work India",
            "trip_type": {
                "Itemized": {
                    "destination_country_code": "IN",
                    "one_way_distance_km": 5800,
                    "number_of_nights": 4,
                    "number_of_days": 5,
                    "domestic_transport_per_instance_eur": "0"
                }
            },
            "number_of_instances": 4,
            "work_package_ids": [1]
        }"#;
        let result: Result<TripInputDto, _> = serde_json::from_str(json);
        assert!(result.is_ok(), "Itemized TripInputDto failed to deserialize: {:?}", result.err());
        assert!(matches!(result.unwrap().trip_type, crate::domain::entities::TripType::Itemized { .. }));
    }

    #[test]
    fn flat_amount_trip_input_deserializes_frontend_json() {
        let json = r#"{
            "name": "Conference EMNLP",
            "trip_type": {
                "FlatAmount": {
                    "flat_amount_per_instance_eur": "2000"
                }
            },
            "number_of_instances": 3,
            "work_package_ids": [1, 2]
        }"#;
        let result: Result<TripInputDto, _> = serde_json::from_str(json);
        assert!(result.is_ok(), "FlatAmount TripInputDto failed to deserialize: {:?}", result.err());
        assert!(matches!(result.unwrap().trip_type, crate::domain::entities::TripType::FlatAmount { .. }));
    }
}
