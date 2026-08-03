//! Execution-specific validation. Imports the shared field-level validators
//! from `erc-core` where applicable; everything here enforces business rules
//! specific to M-03/M-04/M-06 and the from-scratch Amendment Management
//! design (see `domain::enums::AmendmentType` doc comment).

use crate::domain::dto::{
    AmendmentInputDto, MilestoneInputDto, PersonInputDto, PersonMonthRecordInputDto,
    WorkPackageExecutionInputDto,
};
use crate::domain::enums::MilestoneStatus;
use crate::domain::execution_entities::Person;
use crate::error::{AppError, FieldError, ValidationErrors};
use erc_core::domain::entities::PersonnelRole;
use rust_decimal::Decimal;
use uuid::Uuid;

fn parse_iso_date(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

// ─── M-03: Personnel & Person-Month Tracking ──────────────────────────────────

/// BR-PM-01/06 and required-field checks for a `Person`.
///
/// # Arguments
/// * `exclude_id` — the person's own id, when validating an update (excluded
///   from the BR-PM-01 "at most one Person per role" check).
/// * `call_opening_date` — `ProjectConfig.call_opening_date`, used to convert
///   the linked role's `start_month` into a calendar date for BR-PM-06. That
///   field is optional in the Budget App, so BR-PM-06 is simply skipped
///   (not an error) when it hasn't been set.
pub fn validate_person(
    dto: &PersonInputDto,
    existing_persons: &[Person],
    roles: &[PersonnelRole],
    exclude_id: Option<Uuid>,
    call_opening_date: Option<&str>,
) -> Result<(), AppError> {
    let mut errors = ValidationErrors::default();

    if dto.full_name.trim().is_empty() {
        errors.push(FieldError::new(
            "full_name",
            "REQUIRED",
            "Full name is required.",
        ));
    }

    let linked_role = roles.iter().find(|r| r.id == dto.linked_role_id);
    match linked_role {
        None => errors.push(FieldError::new(
            "linked_role_id",
            "NOT_FOUND",
            "Linked role does not exist in this project's budget.",
        )),
        Some(role) => {
            // BR-PM-01: at most one Person per role at a time.
            let role_in_use = existing_persons.iter().any(|p| {
                let same_role = p.linked_role_id == dto.linked_role_id;
                let is_self = exclude_id.map(|id| p.id == id).unwrap_or(false);
                same_role && !is_self
            });
            if role_in_use {
                errors.push(FieldError::new(
                    "linked_role_id",
                    "ROLE_ALREADY_LINKED",
                    "This role is already linked to another person.",
                ));
            }

            // BR-PM-06: actual start date must be <= the role's start month,
            // when we can derive a calendar date (call_opening_date known).
            if let Some(start_date) = parse_iso_date(&dto.actual_start_date) {
                if let Some(call_opening_date) = call_opening_date {
                    if let Some(role_start_date) =
                        month_to_calendar_date(call_opening_date, role.start_month)
                    {
                        if start_date > role_start_date {
                            errors.push(FieldError::new(
                                "actual_start_date",
                                "STARTS_AFTER_ROLE",
                                "Actual start date must be on or before the linked role's planned start month.",
                            ));
                        }
                    }
                }
            } else {
                errors.push(FieldError::new(
                    "actual_start_date",
                    "INVALID_DATE",
                    "Actual start date must be a valid date (YYYY-MM-DD).",
                ));
            }
        }
    }

    if let Some(end) = &dto.actual_end_date {
        if parse_iso_date(end).is_none() {
            errors.push(FieldError::new(
                "actual_end_date",
                "INVALID_DATE",
                "Actual end date must be a valid date (YYYY-MM-DD).",
            ));
        }
    }

    errors.into_result()
}

fn month_to_calendar_date(call_opening_date: &str, month: u32) -> Option<chrono::NaiveDate> {
    let base = chrono::NaiveDate::parse_from_str(call_opening_date, "%Y-%m-%d").ok()?;
    base.checked_add_months(chrono::Months::new(month.saturating_sub(1)))
}

/// BR-PM-03/04/05 for one calendar-month `PersonMonthRecord`. See the
/// module-level scoping note in `domain::execution_entities` for why this is
/// per-month rather than per-period.
///
/// # Arguments
/// * `planned_fte_months_this_month` — sum of `fte_fraction` across every
///   role active in `dto.project_month` (BR-PM-05's 10% tolerance baseline).
pub fn validate_person_month_record(
    dto: &PersonMonthRecordInputDto,
    persons: &[Person],
    planned_fte_months_this_month: Decimal,
    approved_total_this_month_excluding_self: Decimal,
) -> Result<(), AppError> {
    let mut errors = ValidationErrors::default();

    if !persons.iter().any(|p| p.id == dto.person_id) {
        errors.push(FieldError::new(
            "person_id",
            "NOT_FOUND",
            "Person does not exist in this project.",
        ));
    }

    if dto.reported_months < Decimal::ZERO {
        errors.push(FieldError::new(
            "reported_months",
            "NEGATIVE",
            "Reported months cannot be negative.",
        ));
    }
    // BR-PM-03: <= 1.0 FTE-month cap.
    if dto.reported_months > Decimal::ONE {
        errors.push(FieldError::new(
            "reported_months",
            "EXCEEDS_FTE_CAP",
            "Reported months cannot exceed 1.0 (full-time) for a single calendar month.",
        ));
    }

    if let Some(approved) = dto.approved_months {
        if approved < Decimal::ZERO {
            errors.push(FieldError::new(
                "approved_months",
                "NEGATIVE",
                "Approved months cannot be negative.",
            ));
        }
        // BR-PM-04: approved <= reported.
        if approved > dto.reported_months {
            errors.push(FieldError::new(
                "approved_months",
                "EXCEEDS_REPORTED",
                "Approved months cannot exceed reported months.",
            ));
        }

        // BR-PM-05: sum of approved months this month must not exceed planned
        // by more than 10%.
        let total_with_this = approved_total_this_month_excluding_self + approved;
        // 1.10 — BR-PM-05's 10% tolerance multiplier.
        let tolerance = planned_fte_months_this_month * Decimal::new(110, 2);
        if planned_fte_months_this_month > Decimal::ZERO && total_with_this > tolerance {
            errors.push(FieldError::new(
                "approved_months",
                "EXCEEDS_PLANNED_TOLERANCE",
                "Total approved person-months this month exceed the planned amount by more than 10%.",
            ));
        }
    }

    errors.into_result()
}

// ─── M-04: Work Package Management ────────────────────────────────────────────

pub fn validate_work_package_execution(
    dto: &WorkPackageExecutionInputDto,
    roles: &[PersonnelRole],
) -> Result<(), AppError> {
    let mut errors = ValidationErrors::default();

    if let Some(leader_id) = dto.leader_role_id {
        if !roles.iter().any(|r| r.id == leader_id) {
            errors.push(FieldError::new(
                "leader_role_id",
                "NOT_FOUND",
                "Leader role does not exist in this project's budget.",
            ));
        }
    }

    errors.into_result()
}

// ─── M-06: Milestone Tracking ──────────────────────────────────────────────────

pub fn validate_milestone(
    dto: &MilestoneInputDto,
    work_package_count: u8,
    max_month: u32,
) -> Result<(), AppError> {
    let mut errors = ValidationErrors::default();

    if dto.title.trim().is_empty() {
        errors.push(FieldError::new("title", "REQUIRED", "Title is required."));
    }

    if dto.work_package_id == 0 || dto.work_package_id > work_package_count {
        errors.push(FieldError::new(
            "work_package_id",
            "OUT_OF_RANGE",
            "Work package does not exist in this project.",
        ));
    }

    if dto.planned_month == 0 || dto.planned_month > max_month {
        errors.push(FieldError::new(
            "planned_month",
            "OUT_OF_RANGE",
            format!("Planned month must be between 1 and {max_month}."),
        ));
    }

    // AtRisk is a derived overlay (BR-MS-01), never a direct input — see
    // `domain::enums::MilestoneStatus` doc comment.
    if dto.status == MilestoneStatus::AtRisk {
        errors.push(FieldError::new(
            "status",
            "DERIVED_STATUS",
            "'At Risk' is calculated automatically and cannot be set directly.",
        ));
    }

    if let Some(month) = dto.actual_completion_month {
        if month == 0 || month > max_month {
            errors.push(FieldError::new(
                "actual_completion_month",
                "OUT_OF_RANGE",
                format!("Actual completion month must be between 1 and {max_month}."),
            ));
        }
    }

    errors.into_result()
}

// ─── Amendment Management (from-scratch design) ───────────────────────────────

pub fn validate_amendment(dto: &AmendmentInputDto, work_package_count: u8) -> Result<(), AppError> {
    let mut errors = ValidationErrors::default();

    if dto.title.trim().is_empty() {
        errors.push(FieldError::new("title", "REQUIRED", "Title is required."));
    }
    if dto.description.trim().is_empty() {
        errors.push(FieldError::new(
            "description",
            "REQUIRED",
            "Description is required.",
        ));
    }

    let requested = parse_iso_date(&dto.requested_date);
    if requested.is_none() {
        errors.push(FieldError::new(
            "requested_date",
            "INVALID_DATE",
            "Requested date must be a valid date (YYYY-MM-DD).",
        ));
    }

    if let Some(decision_str) = &dto.decision_date {
        match parse_iso_date(decision_str) {
            None => errors.push(FieldError::new(
                "decision_date",
                "INVALID_DATE",
                "Decision date must be a valid date (YYYY-MM-DD).",
            )),
            Some(decision) => {
                if let Some(req) = requested {
                    if decision < req {
                        errors.push(FieldError::new(
                            "decision_date",
                            "BEFORE_REQUESTED",
                            "Decision date cannot be before the requested date.",
                        ));
                    }
                }
            }
        }
    }

    for wp_id in &dto.affected_work_package_ids {
        if *wp_id == 0 || *wp_id > work_package_count {
            errors.push(FieldError::new(
                "affected_work_package_ids",
                "OUT_OF_RANGE",
                "Affected work package does not exist in this project.",
            ));
            break;
        }
    }

    errors.into_result()
}

#[cfg(test)]
mod tests {
    use super::*;
    use erc_core::domain::entities::RoleType;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    fn make_role(id: Uuid, start_month: u32, end_month: u32) -> PersonnelRole {
        PersonnelRole {
            id,
            role_label: "PostDoc-1".to_string(),
            role_type: RoleType::PostDoc,
            current_monthly_salary_try: dec!(50000),
            fte_fraction: dec!(1),
            inflation_rate_pct: dec!(10),
            start_month,
            end_month,
        }
    }

    // ─── validate_person ───────────────────────────────────────────────

    #[test]
    fn test_val_person_valid() {
        let role_id = Uuid::new_v4();
        let roles = vec![make_role(role_id, 1, 12)];
        let dto = PersonInputDto {
            full_name: "Ada Lovelace".to_string(),
            email: None,
            institution: None,
            orcid: None,
            linked_role_id: role_id,
            actual_start_date: "2026-01-01".to_string(),
            actual_end_date: None,
        };
        assert!(validate_person(&dto, &[], &roles, None, None).is_ok());
    }

    #[test]
    fn test_val_person_empty_name_returns_error() {
        let role_id = Uuid::new_v4();
        let roles = vec![make_role(role_id, 1, 12)];
        let dto = PersonInputDto {
            full_name: "  ".to_string(),
            email: None,
            institution: None,
            orcid: None,
            linked_role_id: role_id,
            actual_start_date: "2026-01-01".to_string(),
            actual_end_date: None,
        };
        assert!(validate_person(&dto, &[], &roles, None, None).is_err());
    }

    #[test]
    fn test_val_person_unknown_role_returns_error() {
        let dto = PersonInputDto {
            full_name: "Ada Lovelace".to_string(),
            email: None,
            institution: None,
            orcid: None,
            linked_role_id: Uuid::new_v4(),
            actual_start_date: "2026-01-01".to_string(),
            actual_end_date: None,
        };
        assert!(validate_person(&dto, &[], &[], None, None).is_err());
    }

    #[test]
    fn test_val_person_role_already_linked_returns_error() {
        let role_id = Uuid::new_v4();
        let roles = vec![make_role(role_id, 1, 12)];
        let existing = Person {
            id: Uuid::new_v4(),
            full_name: "Existing Person".to_string(),
            email: None,
            institution: None,
            orcid: None,
            linked_role_id: role_id,
            actual_start_date: "2026-01-01".to_string(),
            actual_end_date: None,
        };
        let dto = PersonInputDto {
            full_name: "Ada Lovelace".to_string(),
            email: None,
            institution: None,
            orcid: None,
            linked_role_id: role_id,
            actual_start_date: "2026-01-01".to_string(),
            actual_end_date: None,
        };
        assert!(validate_person(&dto, &[existing], &roles, None, None).is_err());
    }

    #[test]
    fn test_val_person_update_excludes_self_from_role_link_check() {
        let role_id = Uuid::new_v4();
        let roles = vec![make_role(role_id, 1, 12)];
        let self_id = Uuid::new_v4();
        let existing = Person {
            id: self_id,
            full_name: "Ada Lovelace".to_string(),
            email: None,
            institution: None,
            orcid: None,
            linked_role_id: role_id,
            actual_start_date: "2026-01-01".to_string(),
            actual_end_date: None,
        };
        let dto = PersonInputDto {
            full_name: "Ada Lovelace (updated)".to_string(),
            email: None,
            institution: None,
            orcid: None,
            linked_role_id: role_id,
            actual_start_date: "2026-01-01".to_string(),
            actual_end_date: None,
        };
        assert!(validate_person(&dto, &[existing], &roles, Some(self_id), None).is_ok());
    }

    #[test]
    fn test_val_person_invalid_date_returns_error() {
        let role_id = Uuid::new_v4();
        let roles = vec![make_role(role_id, 1, 12)];
        let dto = PersonInputDto {
            full_name: "Ada Lovelace".to_string(),
            email: None,
            institution: None,
            orcid: None,
            linked_role_id: role_id,
            actual_start_date: "not-a-date".to_string(),
            actual_end_date: None,
        };
        assert!(validate_person(&dto, &[], &roles, None, None).is_err());
    }

    #[test]
    fn test_val_person_br_pm_06_starts_after_role_returns_error() {
        let role_id = Uuid::new_v4();
        // Role starts at project month 1 == 2026-01-01.
        let roles = vec![make_role(role_id, 1, 12)];
        let dto = PersonInputDto {
            full_name: "Ada Lovelace".to_string(),
            email: None,
            institution: None,
            orcid: None,
            linked_role_id: role_id,
            actual_start_date: "2026-02-01".to_string(),
            actual_end_date: None,
        };
        assert!(validate_person(&dto, &[], &roles, None, Some("2026-01-01")).is_err());
    }

    #[test]
    fn test_val_person_br_pm_06_starts_on_or_before_role_is_ok() {
        let role_id = Uuid::new_v4();
        let roles = vec![make_role(role_id, 2, 12)];
        let dto = PersonInputDto {
            full_name: "Ada Lovelace".to_string(),
            email: None,
            institution: None,
            orcid: None,
            linked_role_id: role_id,
            actual_start_date: "2026-01-15".to_string(),
            actual_end_date: None,
        };
        // Role starts at project month 2 == 2026-02-01; actual start is earlier.
        assert!(validate_person(&dto, &[], &roles, None, Some("2026-01-01")).is_ok());
    }

    #[test]
    fn test_val_person_br_pm_06_skipped_when_call_opening_date_absent() {
        let role_id = Uuid::new_v4();
        let roles = vec![make_role(role_id, 1, 12)];
        let dto = PersonInputDto {
            full_name: "Ada Lovelace".to_string(),
            email: None,
            institution: None,
            orcid: None,
            linked_role_id: role_id,
            actual_start_date: "2026-06-01".to_string(),
            actual_end_date: None,
        };
        assert!(validate_person(&dto, &[], &roles, None, None).is_ok());
    }

    // ─── validate_person_month_record ──────────────────────────────────

    fn pm_dto(reported: Decimal, approved: Option<Decimal>) -> PersonMonthRecordInputDto {
        PersonMonthRecordInputDto {
            person_id: Uuid::new_v4(),
            project_month: 1,
            reported_months: reported,
            approved_months: approved,
        }
    }

    #[test]
    fn test_val_pmr_valid() {
        let person_id = Uuid::new_v4();
        let mut dto = pm_dto(dec!(1), Some(dec!(1)));
        dto.person_id = person_id;
        let persons = vec![Person {
            id: person_id,
            full_name: "P".to_string(),
            email: None,
            institution: None,
            orcid: None,
            linked_role_id: Uuid::new_v4(),
            actual_start_date: "2026-01-01".to_string(),
            actual_end_date: None,
        }];
        assert!(validate_person_month_record(&dto, &persons, dec!(1), dec!(0)).is_ok());
    }

    #[test]
    fn test_val_pmr_unknown_person_returns_error() {
        let dto = pm_dto(dec!(1), None);
        assert!(validate_person_month_record(&dto, &[], dec!(1), dec!(0)).is_err());
    }

    #[test]
    fn test_val_pmr_reported_over_one_returns_error() {
        let dto = pm_dto(dec!(1.5), None);
        assert!(validate_person_month_record(&dto, &[], dec!(1), dec!(0)).is_err());
    }

    #[test]
    fn test_val_pmr_approved_exceeds_reported_returns_error() {
        let dto = pm_dto(dec!(0.5), Some(dec!(0.8)));
        assert!(validate_person_month_record(&dto, &[], dec!(1), dec!(0)).is_err());
    }

    #[test]
    fn test_val_pmr_approved_exceeds_planned_tolerance_returns_error() {
        // Planned 1.0 FTE-month, tolerance is 1.10; 0.5 already approved for
        // other roles + this record's 0.7 = 1.2 > 1.10.
        let dto = pm_dto(dec!(0.7), Some(dec!(0.7)));
        assert!(validate_person_month_record(&dto, &[], dec!(1), dec!(0.5)).is_err());
    }

    #[test]
    fn test_val_pmr_within_tolerance_is_ok() {
        let person_id = Uuid::new_v4();
        let mut dto = pm_dto(dec!(0.6), Some(dec!(0.6)));
        dto.person_id = person_id;
        let persons = vec![Person {
            id: person_id,
            full_name: "P".to_string(),
            email: None,
            institution: None,
            orcid: None,
            linked_role_id: Uuid::new_v4(),
            actual_start_date: "2026-01-01".to_string(),
            actual_end_date: None,
        }];
        // 0.5 + 0.6 = 1.1 == 1.10 tolerance boundary, not exceeding.
        assert!(validate_person_month_record(&dto, &persons, dec!(1), dec!(0.5)).is_ok());
    }

    // ─── validate_work_package_execution ───────────────────────────────

    #[test]
    fn test_val_wpe_no_leader_is_ok() {
        let dto = WorkPackageExecutionInputDto {
            leader_role_id: None,
            notes: None,
        };
        assert!(validate_work_package_execution(&dto, &[]).is_ok());
    }

    #[test]
    fn test_val_wpe_unknown_leader_returns_error() {
        let dto = WorkPackageExecutionInputDto {
            leader_role_id: Some(Uuid::new_v4()),
            notes: None,
        };
        assert!(validate_work_package_execution(&dto, &[]).is_err());
    }

    #[test]
    fn test_val_wpe_valid_leader_is_ok() {
        let role_id = Uuid::new_v4();
        let roles = vec![make_role(role_id, 1, 12)];
        let dto = WorkPackageExecutionInputDto {
            leader_role_id: Some(role_id),
            notes: Some("Leads WP1".to_string()),
        };
        assert!(validate_work_package_execution(&dto, &roles).is_ok());
    }

    // ─── validate_milestone ─────────────────────────────────────────────

    fn milestone_dto(work_package_id: u8, planned_month: u32) -> MilestoneInputDto {
        MilestoneInputDto {
            title: "Prototype ready".to_string(),
            work_package_id,
            planned_month,
            status: MilestoneStatus::NotStarted,
            actual_completion_month: None,
        }
    }

    #[test]
    fn test_val_ms_valid() {
        assert!(validate_milestone(&milestone_dto(1, 6), 3, 36).is_ok());
    }

    #[test]
    fn test_val_ms_empty_title_returns_error() {
        let mut dto = milestone_dto(1, 6);
        dto.title = "".to_string();
        assert!(validate_milestone(&dto, 3, 36).is_err());
    }

    #[test]
    fn test_val_ms_wp_out_of_range_returns_error() {
        assert!(validate_milestone(&milestone_dto(9, 6), 3, 36).is_err());
    }

    #[test]
    fn test_val_ms_month_out_of_range_returns_error() {
        assert!(validate_milestone(&milestone_dto(1, 99), 3, 36).is_err());
    }

    #[test]
    fn test_val_ms_direct_at_risk_status_returns_error() {
        let mut dto = milestone_dto(1, 6);
        dto.status = MilestoneStatus::AtRisk;
        assert!(validate_milestone(&dto, 3, 36).is_err());
    }

    #[test]
    fn test_val_ms_actual_completion_out_of_range_returns_error() {
        let mut dto = milestone_dto(1, 6);
        dto.actual_completion_month = Some(99);
        assert!(validate_milestone(&dto, 3, 36).is_err());
    }

    // ─── validate_amendment ─────────────────────────────────────────────

    fn amendment_dto() -> AmendmentInputDto {
        AmendmentInputDto {
            amendment_type: crate::domain::enums::AmendmentType::DurationExtension,
            title: "6-month no-cost extension".to_string(),
            description: "Requesting extension due to equipment delivery delays.".to_string(),
            requested_date: "2026-01-01".to_string(),
            decision_date: None,
            status: crate::domain::enums::AmendmentStatus::Requested,
            financial_impact_eur: None,
            affected_work_package_ids: vec![],
            notes: None,
        }
    }

    #[test]
    fn test_val_amd_valid() {
        assert!(validate_amendment(&amendment_dto(), 3).is_ok());
    }

    #[test]
    fn test_val_amd_empty_title_returns_error() {
        let mut dto = amendment_dto();
        dto.title = "".to_string();
        assert!(validate_amendment(&dto, 3).is_err());
    }

    #[test]
    fn test_val_amd_empty_description_returns_error() {
        let mut dto = amendment_dto();
        dto.description = "".to_string();
        assert!(validate_amendment(&dto, 3).is_err());
    }

    #[test]
    fn test_val_amd_invalid_requested_date_returns_error() {
        let mut dto = amendment_dto();
        dto.requested_date = "not-a-date".to_string();
        assert!(validate_amendment(&dto, 3).is_err());
    }

    #[test]
    fn test_val_amd_decision_before_requested_returns_error() {
        let mut dto = amendment_dto();
        dto.requested_date = "2026-06-01".to_string();
        dto.decision_date = Some("2026-01-01".to_string());
        assert!(validate_amendment(&dto, 3).is_err());
    }

    #[test]
    fn test_val_amd_decision_on_or_after_requested_is_ok() {
        let mut dto = amendment_dto();
        dto.decision_date = Some("2026-01-01".to_string());
        dto.status = crate::domain::enums::AmendmentStatus::Approved;
        assert!(validate_amendment(&dto, 3).is_ok());
    }

    #[test]
    fn test_val_amd_wp_out_of_range_returns_error() {
        let mut dto = amendment_dto();
        dto.affected_work_package_ids = vec![9];
        assert!(validate_amendment(&dto, 3).is_err());
    }
}
