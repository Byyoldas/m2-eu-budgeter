//! Validation Engine — Rust-side business rule enforcement.
//!
//! Field-level validation runs in TypeScript/Zod (instant, no round-trip).
//! This module enforces cross-field and cross-entity business rules that
//! require knowledge of the full project state.
//!
//! Every validator returns `Result<(), AppError>` where the error variant is
//! `AppError::Validation(Vec<FieldError>)` with structured error detail.

use crate::domain::entities::{PersonnelRole, EquipmentItem, Trip, OtherDirectCostItem, TripType, RoleType};
use crate::domain::dto::{PersonnelRoleInputDto, EquipmentItemInputDto, TripInputDto, OtherCostInputDto};
use crate::error::{AppError, FieldError, ValidationErrors};
use rust_decimal::Decimal;

// ─── Personnel Validation ─────────────────────────────────────────────────────

/// Validate a PersonnelRole input against all PE-01 constraints.
///
/// # Arguments
/// * `dto` — The incoming role data from the frontend.
/// * `existing_roles` — All roles already in the project (for uniqueness checks).
/// * `duration_years` — Project duration (for start/end month range check).
/// * `exclude_id` — Optional UUID string to exclude from uniqueness check (for updates).
pub fn validate_personnel_role(
    dto: &PersonnelRoleInputDto,
    existing_roles: &[PersonnelRole],
    duration_years: u8,
    exclude_id: Option<uuid::Uuid>,
) -> Result<(), AppError> {
    let mut errors = ValidationErrors::default();

    // Role label must be non-empty
    if dto.role_label.trim().is_empty() {
        errors.push(FieldError::new("role_label", "REQUIRED", "Role label is required."));
    }

    // Role label must be unique in the project
    let label_in_use = existing_roles.iter().any(|r| {
        let same_label = r.role_label.trim().to_lowercase() == dto.role_label.trim().to_lowercase();
        let is_self = exclude_id.map(|id| r.id == id).unwrap_or(false);
        same_label && !is_self
    });
    if label_in_use {
        errors.push(FieldError::new(
            "role_label",
            "DUPLICATE_LABEL",
            "This label is already in use. Each role must have a unique name.",
        ));
    }

    // Only one PI allowed
    if dto.role_type == RoleType::Pi {
        let pi_exists = existing_roles.iter().any(|r| {
            r.role_type == RoleType::Pi && exclude_id.map(|id| r.id != id).unwrap_or(true)
        });
        if pi_exists {
            errors.push(FieldError::entity(
                "DUPLICATE_PI",
                "Only one Principal Investigator (PI) may be registered per project.",
            ));
        }
    }

    // Salary must be > 0
    if dto.current_monthly_salary_try <= Decimal::ZERO {
        errors.push(FieldError::new(
            "current_monthly_salary_try",
            "INVALID_SALARY_TRY",
            "Monthly salary must be greater than zero.",
        ));
    }

    // FTE fraction must be in (0, 1]
    if dto.fte_fraction <= Decimal::ZERO || dto.fte_fraction > Decimal::ONE {
        errors.push(FieldError::new(
            "fte_fraction",
            "INVALID_FTE",
            "PM must be greater than 0 and at most 1.0 (100%).",
        ));
    }

    // Inflation rate must be in [0, 100]
    if dto.inflation_rate_pct < Decimal::ZERO || dto.inflation_rate_pct > Decimal::ONE_HUNDRED {
        errors.push(FieldError::new(
            "inflation_rate_pct",
            "INVALID_INFLATION_RATE",
            "Inflation rate must be between 0% and 100%.",
        ));
    }

    // Start month must not exceed end month
    if dto.start_month > dto.end_month {
        errors.push(FieldError::new(
            "start_month",
            "INVALID_MONTH_RANGE",
            "Start month must be on or before end month.",
        ));
    }

    // Both months must fall within the project duration
    let max_month = duration_years as u32 * 12;
    if dto.start_month < 1 || dto.start_month > max_month {
        errors.push(FieldError::new(
            "start_month",
            "MONTH_OUT_OF_RANGE",
            format!("Start month {} is outside the project duration of {max_month} months.", dto.start_month),
        ));
    }
    if dto.end_month < 1 || dto.end_month > max_month {
        errors.push(FieldError::new(
            "end_month",
            "MONTH_OUT_OF_RANGE",
            format!("End month {} is outside the project duration of {max_month} months.", dto.end_month),
        ));
    }

    errors.into_result()
}

// ─── Equipment Validation ─────────────────────────────────────────────────────

/// Validate an EquipmentItem input against all EQ-01 constraints.
pub fn validate_equipment_item(
    dto: &EquipmentItemInputDto,
    duration_years: u8,
    work_package_count: u8,
) -> Result<(), AppError> {
    let mut errors = ValidationErrors::default();

    if dto.name.trim().is_empty() {
        errors.push(FieldError::new("name", "REQUIRED", "Item name is required."));
    }

    if dto.purchase_cost_eur <= Decimal::ZERO {
        errors.push(FieldError::new(
            "purchase_cost_eur",
            "INVALID_PURCHASE_COST",
            "Purchase cost must be greater than zero.",
        ));
    }

    if dto.useful_lifetime_months < 1 {
        errors.push(FieldError::new(
            "useful_lifetime_months",
            "INVALID_LIFETIME",
            "Useful lifetime must be at least 1 month.",
        ));
    }

    if dto.grant_usage_pct <= Decimal::ZERO || dto.grant_usage_pct > Decimal::ONE_HUNDRED {
        errors.push(FieldError::new(
            "grant_usage_pct",
            "INVALID_USAGE_PCT",
            "Grant usage must be between 0% (exclusive) and 100% (inclusive).",
        ));
    }

    if dto.grant_usage_months < 1 {
        errors.push(FieldError::new(
            "grant_usage_months",
            "INVALID_USAGE_MONTHS",
            "Months used must be at least 1.",
        ));
    }

    let max_months = duration_years as u32 * 12 + 60; // grace period for late purchases
    if dto.grant_usage_months > max_months {
        errors.push(FieldError::new(
            "grant_usage_months",
            "USAGE_MONTHS_TOO_HIGH",
            format!("Months used cannot exceed {max_months} for a {duration_years}-year project."),
        ));
    }

    if dto.work_package_id < 1 || dto.work_package_id > work_package_count {
        errors.push(FieldError::new(
            "work_package_id",
            "WP_OUT_OF_RANGE",
            "Select a valid Work Package.",
        ));
    }

    errors.into_result()
}

// ─── Trip Validation ──────────────────────────────────────────────────────────

/// Validate a Trip input against all TR-01 constraints.
pub fn validate_trip(dto: &TripInputDto, work_package_count: u8) -> Result<(), AppError> {
    let mut errors = ValidationErrors::default();

    if dto.name.trim().is_empty() {
        errors.push(FieldError::new("name", "REQUIRED", "Trip name or purpose is required."));
    }

    if dto.work_package_ids.is_empty() {
        errors.push(FieldError::new(
            "work_package_ids",
            "NO_WORK_PACKAGE",
            "At least one Work Package must be selected.",
        ));
    }
    for &wp in &dto.work_package_ids {
        if wp < 1 || wp > work_package_count {
            errors.push(FieldError::new(
                "work_package_ids",
                "WP_OUT_OF_RANGE",
                "Select valid Work Packages.",
            ));
            break;
        }
    }

    if dto.number_of_instances < 1 {
        errors.push(FieldError::new(
            "number_of_instances",
            "INVALID_INSTANCES",
            "Number of trip instances must be at least 1.",
        ));
    }

    match &dto.trip_type {
        TripType::Itemized { number_of_nights, number_of_days, destination_country_code, .. } => {
            if destination_country_code.trim().is_empty() {
                errors.push(FieldError::new(
                    "destination_country_code",
                    "REQUIRED",
                    "Destination country is required for itemized trips.",
                ));
            }
            if *number_of_nights < 1 {
                errors.push(FieldError::new(
                    "number_of_nights",
                    "INVALID_NIGHTS",
                    "Number of nights must be at least 1.",
                ));
            }
            if *number_of_days < 1 {
                errors.push(FieldError::new(
                    "number_of_days",
                    "INVALID_DAYS",
                    "Number of days must be at least 1.",
                ));
            }
        }
        TripType::FlatAmount { flat_amount_per_instance_eur } => {
            if *flat_amount_per_instance_eur <= Decimal::ZERO {
                errors.push(FieldError::new(
                    "flat_amount_per_instance_eur",
                    "INVALID_FLAT_AMOUNT",
                    "Flat amount must be greater than zero.",
                ));
            }
        }
    }

    errors.into_result()
}

// ─── Other Cost Validation ────────────────────────────────────────────────────

/// Validate an OtherDirectCostItem input against OC-01 constraints.
pub fn validate_other_cost(
    dto: &OtherCostInputDto,
    work_package_count: u8,
    _existing_items: &[OtherDirectCostItem],
) -> Result<(), AppError> {
    let mut errors = ValidationErrors::default();

    if dto.name.trim().is_empty() {
        errors.push(FieldError::new("name", "REQUIRED", "Item name is required."));
    }

    if dto.amount_eur <= Decimal::ZERO {
        errors.push(FieldError::new(
            "amount_eur",
            "INVALID_C3_AMOUNT",
            "Amount must be greater than zero.",
        ));
    }

    if dto.work_package_ids.is_empty() {
        errors.push(FieldError::new(
            "work_package_ids",
            "NO_WORK_PACKAGE",
            "At least one Work Package must be selected.",
        ));
    }
    for &wp in &dto.work_package_ids {
        if wp < 1 || wp > work_package_count {
            errors.push(FieldError::new(
                "work_package_ids",
                "WP_OUT_OF_RANGE",
                "Select valid Work Packages.",
            ));
            break;
        }
    }

    // OC-01 items are not CFS items — they come through a different path.
    // The user cannot set is_cfs_item via the normal OC form.
    // (is_cfs_item is only set by the OC-02 auto-trigger flow, which bypasses
    // this validator entirely — it has its own inline Work Package validation
    // in commands::other_costs::add_cfs_item.)

    errors.into_result()
}

// ─── Project Config Validation ────────────────────────────────────────────────

/// Validate a ProjectConfig (PS-01 constraints).
pub fn validate_project_config(dto: &crate::domain::dto::ProjectConfigDto) -> Result<(), AppError> {
    let mut errors = ValidationErrors::default();

    if dto.duration_years < 1 || dto.duration_years > 7 {
        errors.push(FieldError::new(
            "duration_years",
            "INVALID_DURATION",
            "Project duration must be between 1 and 7 years.",
        ));
    }

    if dto.work_package_count < 1 || dto.work_package_count > 10 {
        errors.push(FieldError::new(
            "work_package_count",
            "INVALID_WP_COUNT",
            "Number of Work Packages must be between 1 and 10.",
        ));
    }

    if !dto.work_package_start_months.is_empty() || !dto.work_package_end_months.is_empty() {
        if dto.work_package_start_months.len() != dto.work_package_count as usize
            || dto.work_package_end_months.len() != dto.work_package_count as usize
        {
            errors.push(FieldError::new(
                "work_package_start_months",
                "INVALID_WP_DURATION",
                "Each Work Package must have a start and end month.",
            ));
        } else {
            let max_month = dto.duration_years as u32 * 12;
            let mut wp_ranges_valid = true;
            for (i, (&start, &end)) in dto.work_package_start_months.iter()
                .zip(dto.work_package_end_months.iter())
                .enumerate()
            {
                if start < 1 || end < 1 || start > max_month || end > max_month || start > end {
                    errors.push(FieldError::new(
                        "work_package_start_months",
                        "INVALID_WP_DURATION",
                        format!("WP{} duration (Month {start}–{end}) is invalid for a {max_month}-month project.", i + 1),
                    ));
                    wp_ranges_valid = false;
                    break;
                }
            }

            // Every project month must be covered by at least one Work Package,
            // otherwise personnel charged in an uncovered month would silently
            // disappear from every per-WP budget view (Review & Export, Excel,
            // WP Summary) while still counting toward the Category A total —
            // a reconciliation gap. Overlap between WPs is fine (split evenly).
            if wp_ranges_valid {
                let mut covered = vec![false; max_month as usize];
                for (&start, &end) in dto.work_package_start_months.iter()
                    .zip(dto.work_package_end_months.iter())
                {
                    for m in start..=end {
                        covered[(m - 1) as usize] = true;
                    }
                }
                if let Some(gap_idx) = covered.iter().position(|&c| !c) {
                    errors.push(FieldError::new(
                        "work_package_start_months",
                        "WP_COVERAGE_GAP",
                        format!(
                            "Work Packages must collectively cover the entire {max_month}-month project duration. Month {} is not covered by any Work Package.",
                            gap_idx + 1
                        ),
                    ));
                }
            }
        }
    }

    if dto.try_eur_rate <= Decimal::ZERO {
        errors.push(FieldError::new(
            "try_eur_rate",
            "INVALID_EXCHANGE_RATE",
            "Exchange rate must be a positive number greater than zero.",
        ));
    }

    if dto.default_inflation_rate_pct < Decimal::ZERO
        || dto.default_inflation_rate_pct > Decimal::ONE_HUNDRED
    {
        errors.push(FieldError::new(
            "default_inflation_rate_pct",
            "INVALID_INFLATION_RATE",
            "Default inflation rate must be between 0% and 100%.",
        ));
    }

    if dto.indirect_cost_rate_pct < Decimal::ZERO
        || dto.indirect_cost_rate_pct > Decimal::from(50u8)
    {
        errors.push(FieldError::new(
            "indirect_cost_rate_pct",
            "INVALID_INDIRECT_RATE",
            "Indirect cost rate must be between 0% and 50%.",
        ));
    }

    if dto.rate_version_id.trim().is_empty() {
        errors.push(FieldError::new(
            "rate_version_id",
            "REQUIRED",
            "A rate version must be selected.",
        ));
    }

    errors.into_result()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use uuid::Uuid;
    use crate::domain::entities::{PersonnelRole, RoleType, TripType};
    use crate::domain::dto::{ProjectConfigDto, PersonnelRoleInputDto, EquipmentItemInputDto, TripInputDto, OtherCostInputDto};

    // ── Helper builders ────────────────────────────────────────────────────────

    fn make_config_dto(duration: u8) -> ProjectConfigDto {
        let max_month = duration as u32 * 12;
        ProjectConfigDto {
            project_title: "Test Project".to_string(),
            pi_name: "Test PI".to_string(),
            call_reference: "ERC-2025-CoG".to_string(),
            duration_years: duration,
            work_package_count: 3,
            work_package_names: vec![None, None, None],
            work_package_start_months: vec![1, 1, 1],
            work_package_end_months: vec![max_month, max_month, max_month],
            default_inflation_rate_pct: dec!(20),
            try_eur_rate: dec!(50.62),
            indirect_cost_rate_pct: dec!(25),
            rate_version_id: "v_from_2025_05_13".to_string(),
            call_opening_date: None,
        }
    }

    fn make_role_dto(label: &str, role_type: RoleType, salary: &str, fte: &str, inflation: &str, start_month: u32, end_month: u32) -> PersonnelRoleInputDto {
        PersonnelRoleInputDto {
            role_label: label.to_string(),
            role_type,
            current_monthly_salary_try: salary.parse().unwrap(),
            fte_fraction: fte.parse().unwrap(),
            inflation_rate_pct: inflation.parse().unwrap(),
            start_month,
            end_month,
        }
    }

    fn make_existing_role(id: Uuid, label: &str, role_type: RoleType) -> PersonnelRole {
        PersonnelRole {
            id,
            role_label: label.to_string(),
            role_type,
            current_monthly_salary_try: dec!(100000),
            fte_fraction: dec!(1),
            inflation_rate_pct: dec!(20),
            start_month: 1,
            end_month: 60,
        }
    }

    fn make_equipment_dto(cost: &str, lifetime: u32, pct: &str, months: u32) -> EquipmentItemInputDto {
        EquipmentItemInputDto {
            name: "Test Item".to_string(),
            purchase_cost_eur: cost.parse().unwrap(),
            useful_lifetime_months: lifetime,
            grant_usage_pct: pct.parse().unwrap(),
            grant_usage_months: months,
            work_package_id: 1,
        }
    }

    fn make_itemized_trip_dto(country: &str) -> TripInputDto {
        TripInputDto {
            name: "Test Trip".to_string(),
            trip_type: TripType::Itemized {
                destination_country_code: country.to_string(),
                one_way_distance_km: 2500,
                number_of_nights: 4,
                number_of_days: 5,
                domestic_transport_per_instance_eur: dec!(0),
            },
            number_of_instances: 1,
            work_package_ids: vec![1],
        }
    }

    // ── validate_personnel_role tests ──────────────────────────────────────────

    #[test]
    fn test_val_pe_valid_pi_role() {
        let dto = make_role_dto("PI", RoleType::Pi, "227900", "0.70", "20", 1, 60);
        assert!(validate_personnel_role(&dto, &[], 5, None).is_ok());
    }

    #[test]
    fn test_val_pe_empty_label_returns_error() {
        let dto = make_role_dto("", RoleType::Expert, "100000", "1.0", "20", 1, 12);
        let result = validate_personnel_role(&dto, &[], 5, None);
        assert!(has_field_error(&result, "role_label", "REQUIRED"));
    }

    #[test]
    fn test_val_pe_duplicate_label_returns_error() {
        let existing = make_existing_role(Uuid::new_v4(), "PostDoc-1", RoleType::PostDoc);
        let dto = make_role_dto("PostDoc-1", RoleType::Expert, "100000", "1.0", "20", 1, 12);
        let result = validate_personnel_role(&dto, &[existing], 5, None);
        assert!(has_field_error(&result, "role_label", "DUPLICATE_LABEL"));
    }

    #[test]
    fn test_val_pe_duplicate_label_case_insensitive() {
        let existing = make_existing_role(Uuid::new_v4(), "postdoc-1", RoleType::PostDoc);
        let dto = make_role_dto("POSTDOC-1", RoleType::Expert, "100000", "1.0", "20", 1, 12);
        let result = validate_personnel_role(&dto, &[existing], 5, None);
        assert!(has_field_error(&result, "role_label", "DUPLICATE_LABEL"));
    }

    #[test]
    fn test_val_pe_update_excludes_self_from_duplicate_check() {
        let id = Uuid::new_v4();
        let existing = make_existing_role(id, "PostDoc-1", RoleType::PostDoc);
        let dto = make_role_dto("PostDoc-1", RoleType::PostDoc, "160000", "1.0", "20", 1, 24);
        let result = validate_personnel_role(&dto, &[existing], 5, Some(id));
        assert!(result.is_ok());
    }

    #[test]
    fn test_val_pe_second_pi_returns_error() {
        let existing = make_existing_role(Uuid::new_v4(), "PI", RoleType::Pi);
        let dto = make_role_dto("PI-2", RoleType::Pi, "200000", "0.5", "20", 1, 12);
        let result = validate_personnel_role(&dto, &[existing], 5, None);
        assert!(has_entity_error(&result, "DUPLICATE_PI"));
    }

    #[test]
    fn test_val_pe_update_pi_does_not_trigger_duplicate_pi() {
        let id = Uuid::new_v4();
        let existing = make_existing_role(id, "PI", RoleType::Pi);
        let dto = make_role_dto("PI", RoleType::Pi, "230000", "0.70", "20", 1, 60);
        let result = validate_personnel_role(&dto, &[existing], 5, Some(id));
        assert!(result.is_ok());
    }

    #[test]
    fn test_val_pe_zero_salary_returns_error() {
        let mut dto = make_role_dto("Expert-1", RoleType::Expert, "100000", "1.0", "20", 1, 12);
        dto.current_monthly_salary_try = dec!(0);
        let result = validate_personnel_role(&dto, &[], 5, None);
        assert!(has_field_error(&result, "current_monthly_salary_try", "INVALID_SALARY_TRY"));
    }

    #[test]
    fn test_val_pe_fte_zero_returns_error() {
        let mut dto = make_role_dto("Expert-1", RoleType::Expert, "100000", "1.0", "20", 1, 12);
        dto.fte_fraction = dec!(0);
        let result = validate_personnel_role(&dto, &[], 5, None);
        assert!(has_field_error(&result, "fte_fraction", "INVALID_FTE"));
    }

    #[test]
    fn test_val_pe_fte_over_one_returns_error() {
        let mut dto = make_role_dto("Expert-1", RoleType::Expert, "100000", "1.0", "20", 1, 12);
        dto.fte_fraction = dec!(1.1);
        let result = validate_personnel_role(&dto, &[], 5, None);
        assert!(has_field_error(&result, "fte_fraction", "INVALID_FTE"));
    }

    #[test]
    fn test_val_pe_inflation_over_100_returns_error() {
        let dto = make_role_dto("Expert-1", RoleType::Expert, "100000", "1.0", "101", 1, 12);
        let result = validate_personnel_role(&dto, &[], 5, None);
        assert!(has_field_error(&result, "inflation_rate_pct", "INVALID_INFLATION_RATE"));
    }

    #[test]
    fn test_val_pe_start_after_end_returns_error() {
        let dto = make_role_dto("Expert-1", RoleType::Expert, "100000", "1.0", "20", 12, 1);
        let result = validate_personnel_role(&dto, &[], 5, None);
        assert!(has_field_error(&result, "start_month", "INVALID_MONTH_RANGE"));
    }

    #[test]
    fn test_val_pe_month_out_of_range_returns_error() {
        // 5-year project: max month is 60.
        let dto = make_role_dto("Expert-1", RoleType::Expert, "100000", "1.0", "20", 1, 61);
        let result = validate_personnel_role(&dto, &[], 5, None);
        assert!(has_field_error(&result, "end_month", "MONTH_OUT_OF_RANGE"));
    }

    #[test]
    fn test_val_pe_multiple_errors_collected() {
        let mut dto = make_role_dto("", RoleType::Expert, "100000", "1.0", "20", 12, 1);
        dto.fte_fraction = dec!(0);
        let result = validate_personnel_role(&dto, &[], 5, None);
        if let Err(AppError::Validation(errs)) = result {
            assert!(errs.len() >= 3);
        } else {
            panic!("Expected Validation error with multiple fields");
        }
    }

    // ── validate_equipment_item tests ──────────────────────────────────────────

    #[test]
    fn test_val_eq_valid_laptop() {
        assert!(validate_equipment_item(&make_equipment_dto("2500", 48, "100", 55), 5, 3).is_ok());
    }

    #[test]
    fn test_val_eq_empty_name_returns_error() {
        let mut dto = make_equipment_dto("2500", 48, "100", 55);
        dto.name = "".to_string();
        assert!(has_field_error(&validate_equipment_item(&dto, 5, 3), "name", "REQUIRED"));
    }

    #[test]
    fn test_val_eq_zero_cost_returns_error() {
        let dto = make_equipment_dto("0", 48, "100", 55);
        assert!(has_field_error(&validate_equipment_item(&dto, 5, 3), "purchase_cost_eur", "INVALID_PURCHASE_COST"));
    }

    #[test]
    fn test_val_eq_zero_lifetime_returns_error() {
        let dto = make_equipment_dto("2500", 0, "100", 36);
        assert!(has_field_error(&validate_equipment_item(&dto, 5, 3), "useful_lifetime_months", "INVALID_LIFETIME"));
    }

    #[test]
    fn test_val_eq_zero_usage_pct_returns_error() {
        let dto = make_equipment_dto("2500", 48, "0", 36);
        assert!(has_field_error(&validate_equipment_item(&dto, 5, 3), "grant_usage_pct", "INVALID_USAGE_PCT"));
    }

    #[test]
    fn test_val_eq_usage_pct_over_100_returns_error() {
        let dto = make_equipment_dto("2500", 48, "101", 36);
        assert!(has_field_error(&validate_equipment_item(&dto, 5, 3), "grant_usage_pct", "INVALID_USAGE_PCT"));
    }

    #[test]
    fn test_val_eq_zero_usage_months_returns_error() {
        let dto = make_equipment_dto("2500", 48, "100", 0);
        assert!(has_field_error(&validate_equipment_item(&dto, 5, 3), "grant_usage_months", "INVALID_USAGE_MONTHS"));
    }

    #[test]
    fn test_val_eq_usage_months_too_high_returns_error() {
        // 5-year project: max = 5*12+60 = 120. 121 should fail.
        let dto = make_equipment_dto("2500", 48, "100", 121);
        assert!(has_field_error(&validate_equipment_item(&dto, 5, 3), "grant_usage_months", "USAGE_MONTHS_TOO_HIGH"));
    }

    #[test]
    fn test_val_eq_work_package_out_of_range_returns_error() {
        let mut dto = make_equipment_dto("2500", 48, "100", 36);
        dto.work_package_id = 6;
        assert!(has_field_error(&validate_equipment_item(&dto, 5, 3), "work_package_id", "WP_OUT_OF_RANGE"));
    }

    // ── validate_trip tests ────────────────────────────────────────────────────

    #[test]
    fn test_val_tr_valid_itemized_trip() {
        assert!(validate_trip(&make_itemized_trip_dto("IN"), 3).is_ok());
    }

    #[test]
    fn test_val_tr_empty_name_returns_error() {
        let mut dto = make_itemized_trip_dto("IN");
        dto.name = "".to_string();
        assert!(has_field_error(&validate_trip(&dto, 3), "name", "REQUIRED"));
    }

    #[test]
    fn test_val_tr_no_work_package_returns_error() {
        let mut dto = make_itemized_trip_dto("FR");
        dto.work_package_ids = vec![];
        assert!(has_field_error(&validate_trip(&dto, 3), "work_package_ids", "NO_WORK_PACKAGE"));
    }

    #[test]
    fn test_val_tr_work_package_out_of_range_returns_error() {
        let mut dto = make_itemized_trip_dto("FR");
        dto.work_package_ids = vec![9];
        assert!(has_field_error(&validate_trip(&dto, 3), "work_package_ids", "WP_OUT_OF_RANGE"));
    }

    #[test]
    fn test_val_tr_zero_instances_returns_error() {
        let mut dto = make_itemized_trip_dto("IN");
        dto.number_of_instances = 0;
        assert!(has_field_error(&validate_trip(&dto, 3), "number_of_instances", "INVALID_INSTANCES"));
    }

    #[test]
    fn test_val_tr_itemized_empty_country_returns_error() {
        assert!(has_field_error(&validate_trip(&make_itemized_trip_dto(""), 3), "destination_country_code", "REQUIRED"));
    }

    #[test]
    fn test_val_tr_itemized_zero_nights_returns_error() {
        let dto = TripInputDto {
            name: "Bad Trip".to_string(),
            trip_type: TripType::Itemized {
                destination_country_code: "IN".to_string(),
                one_way_distance_km: 5800,
                number_of_nights: 0,
                number_of_days: 5,
                domestic_transport_per_instance_eur: dec!(0),
            },
            number_of_instances: 1,
            work_package_ids: vec![1],
        };
        assert!(has_field_error(&validate_trip(&dto, 3), "number_of_nights", "INVALID_NIGHTS"));
    }

    #[test]
    fn test_val_tr_flat_zero_amount_returns_error() {
        let dto = TripInputDto {
            name: "Flat Trip".to_string(),
            trip_type: TripType::FlatAmount {
                flat_amount_per_instance_eur: dec!(0),
            },
            number_of_instances: 1,
            work_package_ids: vec![1],
        };
        assert!(has_field_error(&validate_trip(&dto, 3), "flat_amount_per_instance_eur", "INVALID_FLAT_AMOUNT"));
    }

    // ── validate_other_cost tests ──────────────────────────────────────────────

    #[test]
    fn test_val_oc_valid_item() {
        let dto = OtherCostInputDto {
            name: "MAXQDA License".to_string(),
            amount_eur: dec!(9870),
            notes: None,
            work_package_ids: vec![1],
        };
        assert!(validate_other_cost(&dto, 3, &[]).is_ok());
    }

    #[test]
    fn test_val_oc_empty_name_returns_error() {
        let dto = OtherCostInputDto {
            name: "".to_string(),
            amount_eur: dec!(500),
            notes: None,
            work_package_ids: vec![1],
        };
        assert!(has_field_error(&validate_other_cost(&dto, 3, &[]), "name", "REQUIRED"));
    }

    #[test]
    fn test_val_oc_zero_amount_returns_error() {
        let dto = OtherCostInputDto {
            name: "Item".to_string(),
            amount_eur: dec!(0),
            notes: None,
            work_package_ids: vec![1],
        };
        assert!(has_field_error(&validate_other_cost(&dto, 3, &[]), "amount_eur", "INVALID_C3_AMOUNT"));
    }

    #[test]
    fn test_val_oc_no_work_package_returns_error() {
        let dto = OtherCostInputDto {
            name: "Item".to_string(),
            amount_eur: dec!(1000),
            notes: None,
            work_package_ids: vec![],
        };
        assert!(has_field_error(&validate_other_cost(&dto, 3, &[]), "work_package_ids", "NO_WORK_PACKAGE"));
    }

    #[test]
    fn test_val_oc_work_package_out_of_range_returns_error() {
        let dto = OtherCostInputDto {
            name: "Item".to_string(),
            amount_eur: dec!(1000),
            notes: None,
            work_package_ids: vec![9],
        };
        assert!(has_field_error(&validate_other_cost(&dto, 3, &[]), "work_package_ids", "WP_OUT_OF_RANGE"));
    }

    // ── validate_project_config tests ──────────────────────────────────────────

    #[test]
    fn test_val_cfg_valid_5_year_project() {
        assert!(validate_project_config(&make_config_dto(5)).is_ok());
    }

    #[test]
    fn test_val_cfg_zero_duration_returns_error() {
        let mut dto = make_config_dto(5);
        dto.duration_years = 0;
        assert!(has_field_error(&validate_project_config(&dto), "duration_years", "INVALID_DURATION"));
    }

    #[test]
    fn test_val_cfg_8_year_duration_returns_error() {
        assert!(has_field_error(&validate_project_config(&make_config_dto(8)), "duration_years", "INVALID_DURATION"));
    }

    #[test]
    fn test_val_cfg_zero_exchange_rate_returns_error() {
        let mut dto = make_config_dto(5);
        dto.try_eur_rate = dec!(0);
        assert!(has_field_error(&validate_project_config(&dto), "try_eur_rate", "INVALID_EXCHANGE_RATE"));
    }

    #[test]
    fn test_val_cfg_indirect_rate_over_50_returns_error() {
        let mut dto = make_config_dto(5);
        dto.indirect_cost_rate_pct = dec!(51);
        assert!(has_field_error(&validate_project_config(&dto), "indirect_cost_rate_pct", "INVALID_INDIRECT_RATE"));
    }

    #[test]
    fn test_val_cfg_empty_rate_version_returns_error() {
        let mut dto = make_config_dto(5);
        dto.rate_version_id = "".to_string();
        assert!(has_field_error(&validate_project_config(&dto), "rate_version_id", "REQUIRED"));
    }

    #[test]
    fn test_val_cfg_wp_count_11_returns_error() {
        let mut dto = make_config_dto(5);
        dto.work_package_count = 11;
        assert!(has_field_error(&validate_project_config(&dto), "work_package_count", "INVALID_WP_COUNT"));
    }

    #[test]
    fn test_val_cfg_wp_duration_valid_passes() {
        let dto = make_config_dto(5);
        assert!(validate_project_config(&dto).is_ok());
    }

    #[test]
    fn test_val_cfg_wp_duration_empty_arrays_skip_validation() {
        // Backward compatibility: files saved before this field existed have empty arrays.
        let mut dto = make_config_dto(5);
        dto.work_package_start_months = vec![];
        dto.work_package_end_months = vec![];
        assert!(validate_project_config(&dto).is_ok());
    }

    #[test]
    fn test_val_cfg_wp_duration_length_mismatch_returns_error() {
        let mut dto = make_config_dto(5);
        dto.work_package_start_months = vec![1, 1]; // only 2, but work_package_count is 3
        assert!(has_field_error(&validate_project_config(&dto), "work_package_start_months", "INVALID_WP_DURATION"));
    }

    #[test]
    fn test_val_cfg_wp_duration_start_after_end_returns_error() {
        let mut dto = make_config_dto(5);
        dto.work_package_start_months = vec![30, 1, 1];
        dto.work_package_end_months = vec![20, 60, 60]; // WP1: start 30 > end 20
        assert!(has_field_error(&validate_project_config(&dto), "work_package_start_months", "INVALID_WP_DURATION"));
    }

    #[test]
    fn test_val_cfg_wp_duration_month_out_of_range_returns_error() {
        let mut dto = make_config_dto(5);
        dto.work_package_end_months = vec![60, 60, 61]; // 61 exceeds 60-month (5-year) duration
        assert!(has_field_error(&validate_project_config(&dto), "work_package_start_months", "INVALID_WP_DURATION"));
    }

    #[test]
    fn test_val_cfg_wp_coverage_gap_returns_error() {
        let mut dto = make_config_dto(1);
        dto.work_package_count = 1;
        dto.work_package_names = vec![None];
        // WP1 only covers months 1-8; months 9-12 are uncovered.
        dto.work_package_start_months = vec![1];
        dto.work_package_end_months = vec![8];
        assert!(has_field_error(&validate_project_config(&dto), "work_package_start_months", "WP_COVERAGE_GAP"));
    }

    #[test]
    fn test_val_cfg_wp_sequential_full_coverage_is_ok() {
        let mut dto = make_config_dto(1);
        dto.duration_years = 2;
        dto.work_package_count = 2;
        dto.work_package_names = vec![None, None];
        dto.work_package_start_months = vec![1, 13];
        dto.work_package_end_months = vec![12, 24];
        assert!(validate_project_config(&dto).is_ok());
    }

    #[test]
    fn test_val_cfg_wp_overlapping_full_coverage_is_ok() {
        let mut dto = make_config_dto(1);
        dto.duration_years = 1;
        dto.work_package_count = 2;
        dto.work_package_names = vec![None, None];
        // Both WPs span the full year — full coverage, overlapping (allowed).
        dto.work_package_start_months = vec![1, 1];
        dto.work_package_end_months = vec![12, 12];
        assert!(validate_project_config(&dto).is_ok());
    }

    // ── Assertion helpers ──────────────────────────────────────────────────────

    fn has_field_error(result: &Result<(), AppError>, field: &str, code: &str) -> bool {
        match result {
            Err(AppError::Validation(errs)) => errs.iter().any(|e| {
                e.field.as_deref() == Some(field) && e.code == code
            }),
            _ => false,
        }
    }

    fn has_entity_error(result: &Result<(), AppError>, code: &str) -> bool {
        match result {
            Err(AppError::Validation(errs)) => errs.iter().any(|e| e.field.is_none() && e.code == code),
            _ => false,
        }
    }
}
