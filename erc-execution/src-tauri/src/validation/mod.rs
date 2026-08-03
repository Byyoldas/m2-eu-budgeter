//! Execution-specific validation. Imports the shared field-level validators
//! from `erc-core` where applicable; everything here enforces business rules
//! specific to M-03/M-04/M-06/M-08/M-09/M-10/M-11 and the from-scratch
//! Amendment Management design (see `domain::enums::AmendmentType` doc comment).

use crate::domain::dto::{
    ActualCostEntryInputDto, AmendmentInputDto, DeliverableInputDto, EquipmentProcurementInputDto,
    IssueEntryInputDto, MilestoneInputDto, PersonInputDto, PersonMonthRecordInputDto,
    ReportingPeriodInputDto, RiskEntryInputDto, SubcontractingLineInputDto, TripExecutionInputDto,
    WorkPackageExecutionInputDto,
};
use crate::domain::enums::{DeliverableStatus, IssueStatus, Level, MilestoneStatus, RiskStatus};
use crate::domain::execution_entities::{
    Deliverable, Person, ReportingPeriod, RiskEntry, SubcontractingLine, TripExecution,
};
use crate::engines::risk_engine::{derive_risk_priority, risk_score};
use crate::error::{AppError, FieldError, ValidationErrors};
use erc_core::domain::entities::{EquipmentItem, OtherDirectCostItem, PersonnelRole, Trip};
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
    deliverables: &[Deliverable],
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

    for deliverable_id in &dto.linked_deliverable_ids {
        if !deliverables.iter().any(|d| d.id == *deliverable_id) {
            errors.push(FieldError::new(
                "linked_deliverable_ids",
                "NOT_FOUND",
                "Linked deliverable does not exist in this project.",
            ));
            break;
        }
    }

    errors.into_result()
}

/// BR-MS-02: a milestone can only be marked `Completed` once every linked
/// deliverable is `Accepted`. Called explicitly by `commands::milestones`
/// whenever a mutation would set `status` to `Completed` — not folded into
/// `validate_milestone` itself, since that runs on every save regardless of
/// the target status.
pub fn validate_milestone_completion(
    linked_deliverable_ids: &[Uuid],
    deliverables: &[Deliverable],
) -> Result<(), AppError> {
    let mut errors = ValidationErrors::default();

    let all_accepted = linked_deliverable_ids.iter().all(|id| {
        deliverables
            .iter()
            .find(|d| d.id == *id)
            .is_some_and(|d| d.status == DeliverableStatus::Accepted)
    });
    if !all_accepted {
        errors.push(FieldError::new(
            "status",
            "DELIVERABLES_NOT_ACCEPTED",
            "All linked deliverables must be Accepted before this milestone can be completed.",
        ));
    }

    errors.into_result()
}

// ─── M-05: Deliverable Tracking ────────────────────────────────────────────────

pub fn validate_deliverable(
    dto: &DeliverableInputDto,
    work_package_count: u8,
    max_month: u32,
    roles: &[PersonnelRole],
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

    if !roles.iter().any(|r| r.id == dto.responsible_role_id) {
        errors.push(FieldError::new(
            "responsible_role_id",
            "NOT_FOUND",
            "Responsible role does not exist in this project's budget.",
        ));
    }

    if let Some(date) = &dto.actual_submission_date {
        if parse_iso_date(date).is_none() {
            errors.push(FieldError::new(
                "actual_submission_date",
                "INVALID_DATE",
                "Actual submission date must be a valid date (YYYY-MM-DD).",
            ));
        }
    }

    // BR-DEL-03: Rejected deliverables must have a revision note and a
    // revised planned month.
    if dto.status == DeliverableStatus::Rejected {
        if dto.revision_note.as_deref().unwrap_or("").trim().is_empty() {
            errors.push(FieldError::new(
                "revision_note",
                "REQUIRED",
                "A revision note is required for rejected deliverables.",
            ));
        }
        if dto.revised_planned_month.is_none() {
            errors.push(FieldError::new(
                "revised_planned_month",
                "REQUIRED",
                "A revised planned month is required for rejected deliverables.",
            ));
        }
    }

    if let Some(month) = dto.revised_planned_month {
        if month == 0 || month > max_month {
            errors.push(FieldError::new(
                "revised_planned_month",
                "OUT_OF_RANGE",
                format!("Revised planned month must be between 1 and {max_month}."),
            ));
        }
    }

    errors.into_result()
}

// ─── M-14: Reporting Period Management ─────────────────────────────────────────

/// # Arguments
/// * `exclude_id` — this period's own id, when validating an update
///   (excluded from the BR-RP overlap check).
///
/// BR-RP-01/02 (full, gapless coverage of the project duration, and the
/// final period ending exactly at `max_month`) are surfaced as advisory
/// warnings on the periods list (`build_summary`) rather than enforced here
/// — blocking single-record edits on a collective, whole-list invariant
/// would make editing one period at a time impossible. Only the per-record
/// range/overlap checks and BR-RP-03 are hard-enforced.
pub fn validate_reporting_period(
    dto: &ReportingPeriodInputDto,
    existing_periods: &[ReportingPeriod],
    max_month: u32,
    exclude_id: Option<Uuid>,
) -> Result<(), AppError> {
    let mut errors = ValidationErrors::default();

    if dto.start_month == 0 || dto.start_month > max_month {
        errors.push(FieldError::new(
            "start_month",
            "OUT_OF_RANGE",
            format!("Start month must be between 1 and {max_month}."),
        ));
    }
    if dto.end_month == 0 || dto.end_month > max_month {
        errors.push(FieldError::new(
            "end_month",
            "OUT_OF_RANGE",
            format!("End month must be between 1 and {max_month}."),
        ));
    }
    if dto.start_month > dto.end_month {
        errors.push(FieldError::new(
            "end_month",
            "BEFORE_START",
            "End month cannot be before start month.",
        ));
    }

    let overlaps = existing_periods.iter().any(|p| {
        Some(p.id) != exclude_id && dto.start_month <= p.end_month && p.start_month <= dto.end_month
    });
    if overlaps {
        errors.push(FieldError::new(
            "start_month",
            "OVERLAPS_EXISTING",
            "This period overlaps an existing reporting period.",
        ));
    }

    if let Some(deadline) = &dto.submission_deadline {
        if parse_iso_date(deadline).is_none() {
            errors.push(FieldError::new(
                "submission_deadline",
                "INVALID_DATE",
                "Submission deadline must be a valid date (YYYY-MM-DD).",
            ));
        }
    }

    // BR-RP-03: cannot move to Submitted without both report flags set.
    if dto.status == crate::domain::enums::ReportingPeriodStatus::Submitted
        && !(dto.technical_report_submitted && dto.financial_report_submitted)
    {
        errors.push(FieldError::new(
            "status",
            "REPORTS_INCOMPLETE",
            "Both the technical and financial report must be submitted before this period can be marked Submitted.",
        ));
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

// ─── M-08: Travel Tracking ──────────────────────────────────────────────────────

/// # Arguments
/// * `exclude_id` — this record's own id, when validating an update (excluded
///   from the "one execution per instance" uniqueness check).
pub fn validate_trip_execution(
    dto: &TripExecutionInputDto,
    trips: &[Trip],
    persons: &[Person],
    existing_executions: &[TripExecution],
    exclude_id: Option<Uuid>,
) -> Result<(), AppError> {
    let mut errors = ValidationErrors::default();

    match trips.iter().find(|t| t.id == dto.trip_id) {
        None => errors.push(FieldError::new(
            "trip_id",
            "NOT_FOUND",
            "Trip does not exist in this project's budget.",
        )),
        Some(trip) => {
            if dto.instance_number == 0 || dto.instance_number > trip.number_of_instances {
                errors.push(FieldError::new(
                    "instance_number",
                    "OUT_OF_RANGE",
                    format!(
                        "Instance number must be between 1 and {}.",
                        trip.number_of_instances
                    ),
                ));
            }
            let duplicate = existing_executions.iter().any(|e| {
                e.trip_id == dto.trip_id
                    && e.instance_number == dto.instance_number
                    && Some(e.id) != exclude_id
            });
            if duplicate {
                errors.push(FieldError::new(
                    "instance_number",
                    "DUPLICATE_INSTANCE",
                    "This trip instance already has an execution record.",
                ));
            }
        }
    }

    if !persons.iter().any(|p| p.id == dto.traveller_person_id) {
        errors.push(FieldError::new(
            "traveller_person_id",
            "NOT_FOUND",
            "Traveller does not exist in this project.",
        ));
    }

    if parse_iso_date(&dto.actual_travel_date).is_none() {
        errors.push(FieldError::new(
            "actual_travel_date",
            "INVALID_DATE",
            "Actual travel date must be a valid date (YYYY-MM-DD).",
        ));
    }

    if dto.actual_cost_eur <= Decimal::ZERO {
        errors.push(FieldError::new(
            "actual_cost_eur",
            "NOT_POSITIVE",
            "Actual cost must be greater than zero.",
        ));
    }

    errors.into_result()
}

// ─── M-09: Equipment Tracking ───────────────────────────────────────────────────

pub fn validate_equipment_procurement(
    dto: &EquipmentProcurementInputDto,
    equipment_items: &[EquipmentItem],
) -> Result<(), AppError> {
    let mut errors = ValidationErrors::default();

    if !equipment_items
        .iter()
        .any(|e| e.id == dto.equipment_item_id)
    {
        errors.push(FieldError::new(
            "equipment_item_id",
            "NOT_FOUND",
            "Equipment item does not exist in this project's budget.",
        ));
    }

    if dto.actual_purchase_cost_eur <= Decimal::ZERO {
        errors.push(FieldError::new(
            "actual_purchase_cost_eur",
            "NOT_POSITIVE",
            "Actual purchase cost must be greater than zero.",
        ));
    }

    if parse_iso_date(&dto.purchase_date).is_none() {
        errors.push(FieldError::new(
            "purchase_date",
            "INVALID_DATE",
            "Purchase date must be a valid date (YYYY-MM-DD).",
        ));
    }

    errors.into_result()
}

// ─── M-10: Other Costs Tracking ─────────────────────────────────────────────────

pub fn validate_actual_cost_entry(
    dto: &ActualCostEntryInputDto,
    other_cost_items: &[OtherDirectCostItem],
) -> Result<(), AppError> {
    let mut errors = ValidationErrors::default();

    match dto.linked_entity_id {
        Some(linked_id) => {
            if !other_cost_items.iter().any(|i| i.id == linked_id) {
                errors.push(FieldError::new(
                    "linked_entity_id",
                    "NOT_FOUND",
                    "Linked other cost item does not exist in this project's budget.",
                ));
            }
        }
        // BR-OC-03: unbudgeted entries require justification.
        None => {
            if dto.justification.as_deref().unwrap_or("").trim().is_empty() {
                errors.push(FieldError::new(
                    "justification",
                    "REQUIRED",
                    "Justification is required for unbudgeted cost entries.",
                ));
            }
        }
    }

    if dto.amount_eur <= Decimal::ZERO {
        errors.push(FieldError::new(
            "amount_eur",
            "NOT_POSITIVE",
            "Amount must be greater than zero.",
        ));
    }

    if dto.description.trim().is_empty() {
        errors.push(FieldError::new(
            "description",
            "REQUIRED",
            "Description is required.",
        ));
    }

    if parse_iso_date(&dto.incurred_date).is_none() {
        errors.push(FieldError::new(
            "incurred_date",
            "INVALID_DATE",
            "Incurred date must be a valid date (YYYY-MM-DD).",
        ));
    }

    errors.into_result()
}

// ─── M-11: Subcontracting Tracking ──────────────────────────────────────────────

/// # Arguments
/// * `exclude_id` — this line's own id, when validating an update (excluded
///   from the BR-SC-01 cap check).
pub fn validate_subcontracting_line(
    dto: &SubcontractingLineInputDto,
    existing_lines: &[SubcontractingLine],
    planned_amount_eur: Decimal,
    work_package_count: u8,
    exclude_id: Option<Uuid>,
) -> Result<(), AppError> {
    let mut errors = ValidationErrors::default();

    if dto.vendor.trim().is_empty() {
        errors.push(FieldError::new("vendor", "REQUIRED", "Vendor is required."));
    }
    if dto.contract_reference.trim().is_empty() {
        errors.push(FieldError::new(
            "contract_reference",
            "REQUIRED",
            "Contract reference is required.",
        ));
    }
    if dto.amount_eur <= Decimal::ZERO {
        errors.push(FieldError::new(
            "amount_eur",
            "NOT_POSITIVE",
            "Amount must be greater than zero.",
        ));
    }
    if dto.work_package_id == 0 || dto.work_package_id > work_package_count {
        errors.push(FieldError::new(
            "work_package_id",
            "OUT_OF_RANGE",
            "Work package does not exist in this project.",
        ));
    }
    if let Some(date) = &dto.payment_date {
        if parse_iso_date(date).is_none() {
            errors.push(FieldError::new(
                "payment_date",
                "INVALID_DATE",
                "Payment date must be a valid date (YYYY-MM-DD).",
            ));
        }
    }

    // BR-SC-01: total of all lines must not exceed the planned lump sum.
    let existing_total: Decimal = existing_lines
        .iter()
        .filter(|l| Some(l.id) != exclude_id)
        .map(|l| l.amount_eur)
        .sum();
    if existing_total + dto.amount_eur > planned_amount_eur {
        errors.push(FieldError::new(
            "amount_eur",
            "EXCEEDS_PLANNED_SUBCONTRACTING",
            "Total subcontracting lines cannot exceed the planned subcontracting amount.",
        ));
    }

    errors.into_result()
}

// ─── M-12: Risk Register ────────────────────────────────────────────────────────

/// # Arguments
/// * `existing_status` — the risk's current stored status, when validating
///   an update (`None` on create). BR-RK-04: once `Closed`, terminal.
/// * `today` — real calendar date (not project month) for BR-RK-03's 30-day
///   review-date check.
pub fn validate_risk_entry(
    dto: &RiskEntryInputDto,
    roles: &[PersonnelRole],
    work_package_count: u8,
    existing_status: Option<RiskStatus>,
    today: chrono::NaiveDate,
) -> Result<(), AppError> {
    let mut errors = ValidationErrors::default();

    if dto.title.trim().is_empty() {
        errors.push(FieldError::new("title", "REQUIRED", "Title is required."));
    }

    if let Some(wp_id) = dto.work_package_id {
        if wp_id == 0 || wp_id > work_package_count {
            errors.push(FieldError::new(
                "work_package_id",
                "OUT_OF_RANGE",
                "Work package does not exist in this project.",
            ));
        }
    }

    if let Some(owner_id) = dto.owner_role_id {
        if !roles.iter().any(|r| r.id == owner_id) {
            errors.push(FieldError::new(
                "owner_role_id",
                "NOT_FOUND",
                "Owner role does not exist in this project's budget.",
            ));
        }
    }

    if parse_iso_date(&dto.identified_date).is_none() {
        errors.push(FieldError::new(
            "identified_date",
            "INVALID_DATE",
            "Identified date must be a valid date (YYYY-MM-DD).",
        ));
    }

    // BR-RK-03: High-priority risks require a review date within 30 days.
    if derive_risk_priority(risk_score(dto.probability, dto.impact)) == Level::High {
        match dto.review_date.as_deref().and_then(parse_iso_date) {
            None => errors.push(FieldError::new(
                "review_date",
                "REQUIRED",
                "High-priority risks require a review date.",
            )),
            Some(review_date) => {
                if review_date > today + chrono::Duration::days(30) {
                    errors.push(FieldError::new(
                        "review_date",
                        "TOO_FAR_OUT",
                        "High-priority risks require a review date within 30 days.",
                    ));
                }
            }
        }
    } else if let Some(date) = &dto.review_date {
        if parse_iso_date(date).is_none() {
            errors.push(FieldError::new(
                "review_date",
                "INVALID_DATE",
                "Review date must be a valid date (YYYY-MM-DD).",
            ));
        }
    }

    if let Some(date) = &dto.closed_date {
        if parse_iso_date(date).is_none() {
            errors.push(FieldError::new(
                "closed_date",
                "INVALID_DATE",
                "Closed date must be a valid date (YYYY-MM-DD).",
            ));
        }
    }

    // BR-RK-04: a Closed risk cannot be re-opened.
    if existing_status == Some(RiskStatus::Closed) && dto.status != RiskStatus::Closed {
        errors.push(FieldError::new(
            "status",
            "CANNOT_REOPEN_CLOSED_RISK",
            "A closed risk cannot be re-opened; create a new entry instead.",
        ));
    }

    errors.into_result()
}

// ─── M-13: Issue Log ─────────────────────────────────────────────────────────────

pub fn validate_issue_entry(
    dto: &IssueEntryInputDto,
    roles: &[PersonnelRole],
    work_package_count: u8,
    existing_risks: &[RiskEntry],
    today: chrono::NaiveDate,
) -> Result<(), AppError> {
    let mut errors = ValidationErrors::default();

    if dto.description.trim().is_empty() {
        errors.push(FieldError::new(
            "description",
            "REQUIRED",
            "Description is required.",
        ));
    }

    if let Some(wp_id) = dto.work_package_id {
        if wp_id == 0 || wp_id > work_package_count {
            errors.push(FieldError::new(
                "work_package_id",
                "OUT_OF_RANGE",
                "Work package does not exist in this project.",
            ));
        }
    }

    match parse_iso_date(&dto.raised_date) {
        None => errors.push(FieldError::new(
            "raised_date",
            "INVALID_DATE",
            "Raised date must be a valid date (YYYY-MM-DD).",
        )),
        Some(raised) => {
            if raised > today {
                errors.push(FieldError::new(
                    "raised_date",
                    "IN_FUTURE",
                    "Raised date cannot be in the future.",
                ));
            }
        }
    }

    if let Some(owner_id) = dto.owner_role_id {
        if !roles.iter().any(|r| r.id == owner_id) {
            errors.push(FieldError::new(
                "owner_role_id",
                "NOT_FOUND",
                "Owner role does not exist in this project's budget.",
            ));
        }
    }

    if let Some(risk_id) = dto.linked_risk_id {
        if !existing_risks.iter().any(|r| r.id == risk_id) {
            errors.push(FieldError::new(
                "linked_risk_id",
                "NOT_FOUND",
                "Linked risk does not exist in this project.",
            ));
        }
    }

    // BR-IS-01: an issue without a resolution cannot be marked Closed.
    if dto.status == IssueStatus::Closed
        && dto.resolution.as_deref().unwrap_or("").trim().is_empty()
    {
        errors.push(FieldError::new(
            "resolution",
            "REQUIRED",
            "A resolution is required to close an issue.",
        ));
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
            linked_deliverable_ids: vec![],
        }
    }

    #[test]
    fn test_val_ms_valid() {
        assert!(validate_milestone(&milestone_dto(1, 6), 3, 36, &[]).is_ok());
    }

    #[test]
    fn test_val_ms_empty_title_returns_error() {
        let mut dto = milestone_dto(1, 6);
        dto.title = "".to_string();
        assert!(validate_milestone(&dto, 3, 36, &[]).is_err());
    }

    #[test]
    fn test_val_ms_wp_out_of_range_returns_error() {
        assert!(validate_milestone(&milestone_dto(9, 6), 3, 36, &[]).is_err());
    }

    #[test]
    fn test_val_ms_month_out_of_range_returns_error() {
        assert!(validate_milestone(&milestone_dto(1, 99), 3, 36, &[]).is_err());
    }

    #[test]
    fn test_val_ms_direct_at_risk_status_returns_error() {
        let mut dto = milestone_dto(1, 6);
        dto.status = MilestoneStatus::AtRisk;
        assert!(validate_milestone(&dto, 3, 36, &[]).is_err());
    }

    #[test]
    fn test_val_ms_actual_completion_out_of_range_returns_error() {
        let mut dto = milestone_dto(1, 6);
        dto.actual_completion_month = Some(99);
        assert!(validate_milestone(&dto, 3, 36, &[]).is_err());
    }

    #[test]
    fn test_val_ms_unknown_linked_deliverable_returns_error() {
        let mut dto = milestone_dto(1, 6);
        dto.linked_deliverable_ids = vec![Uuid::new_v4()];
        assert!(validate_milestone(&dto, 3, 36, &[]).is_err());
    }

    #[test]
    fn test_val_ms_known_linked_deliverable_is_ok() {
        let deliverable = make_deliverable(DeliverableStatus::NotStarted);
        let mut dto = milestone_dto(1, 6);
        dto.linked_deliverable_ids = vec![deliverable.id];
        assert!(validate_milestone(&dto, 3, 36, &[deliverable]).is_ok());
    }

    // ─── validate_milestone_completion ──────────────────────────────────

    #[test]
    fn test_val_ms_completion_all_accepted_is_ok() {
        let deliverable = make_deliverable(DeliverableStatus::Accepted);
        let id = deliverable.id;
        assert!(validate_milestone_completion(&[id], &[deliverable]).is_ok());
    }

    #[test]
    fn test_val_ms_completion_no_links_is_ok() {
        assert!(validate_milestone_completion(&[], &[]).is_ok());
    }

    #[test]
    fn test_val_ms_completion_unaccepted_deliverable_returns_error() {
        let deliverable = make_deliverable(DeliverableStatus::Submitted);
        let id = deliverable.id;
        assert!(validate_milestone_completion(&[id], &[deliverable]).is_err());
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

    // ─── validate_trip_execution ────────────────────────────────────────

    use erc_core::domain::entities::TripType;

    fn make_trip(id: Uuid, instances: u32) -> Trip {
        Trip {
            id,
            name: "Conference".to_string(),
            trip_type: TripType::FlatAmount {
                flat_amount_per_instance_eur: dec!(500),
            },
            number_of_instances: instances,
            work_package_ids: vec![1],
        }
    }

    fn make_person(id: Uuid) -> Person {
        Person {
            id,
            full_name: "Ada Lovelace".to_string(),
            email: None,
            institution: None,
            orcid: None,
            linked_role_id: Uuid::new_v4(),
            actual_start_date: "2026-01-01".to_string(),
            actual_end_date: None,
        }
    }

    fn trip_execution_dto(trip_id: Uuid, person_id: Uuid, instance: u32) -> TripExecutionInputDto {
        TripExecutionInputDto {
            trip_id,
            instance_number: instance,
            traveller_person_id: person_id,
            actual_travel_date: "2026-03-01".to_string(),
            actual_cost_eur: dec!(500),
            status: crate::domain::enums::EntryStatus::Approved,
        }
    }

    #[test]
    fn test_val_te_valid() {
        let trip_id = Uuid::new_v4();
        let person_id = Uuid::new_v4();
        let trips = vec![make_trip(trip_id, 2)];
        let persons = vec![make_person(person_id)];
        let dto = trip_execution_dto(trip_id, person_id, 1);
        assert!(validate_trip_execution(&dto, &trips, &persons, &[], None).is_ok());
    }

    #[test]
    fn test_val_te_unknown_trip_returns_error() {
        let person_id = Uuid::new_v4();
        let persons = vec![make_person(person_id)];
        let dto = trip_execution_dto(Uuid::new_v4(), person_id, 1);
        assert!(validate_trip_execution(&dto, &[], &persons, &[], None).is_err());
    }

    #[test]
    fn test_val_te_instance_out_of_range_returns_error() {
        let trip_id = Uuid::new_v4();
        let person_id = Uuid::new_v4();
        let trips = vec![make_trip(trip_id, 2)];
        let persons = vec![make_person(person_id)];
        let dto = trip_execution_dto(trip_id, person_id, 5);
        assert!(validate_trip_execution(&dto, &trips, &persons, &[], None).is_err());
    }

    #[test]
    fn test_val_te_unknown_traveller_returns_error() {
        let trip_id = Uuid::new_v4();
        let trips = vec![make_trip(trip_id, 2)];
        let dto = trip_execution_dto(trip_id, Uuid::new_v4(), 1);
        assert!(validate_trip_execution(&dto, &trips, &[], &[], None).is_err());
    }

    #[test]
    fn test_val_te_duplicate_instance_returns_error() {
        let trip_id = Uuid::new_v4();
        let person_id = Uuid::new_v4();
        let trips = vec![make_trip(trip_id, 2)];
        let persons = vec![make_person(person_id)];
        let existing = TripExecution {
            id: Uuid::new_v4(),
            trip_id,
            instance_number: 1,
            traveller_person_id: person_id,
            actual_travel_date: "2026-03-01".to_string(),
            actual_cost_eur: dec!(500),
            status: crate::domain::enums::EntryStatus::Approved,
        };
        let dto = trip_execution_dto(trip_id, person_id, 1);
        assert!(validate_trip_execution(&dto, &trips, &persons, &[existing], None).is_err());
    }

    #[test]
    fn test_val_te_update_excludes_self_from_duplicate_check() {
        let trip_id = Uuid::new_v4();
        let person_id = Uuid::new_v4();
        let self_id = Uuid::new_v4();
        let trips = vec![make_trip(trip_id, 2)];
        let persons = vec![make_person(person_id)];
        let existing = TripExecution {
            id: self_id,
            trip_id,
            instance_number: 1,
            traveller_person_id: person_id,
            actual_travel_date: "2026-03-01".to_string(),
            actual_cost_eur: dec!(500),
            status: crate::domain::enums::EntryStatus::Approved,
        };
        let dto = trip_execution_dto(trip_id, person_id, 1);
        assert!(
            validate_trip_execution(&dto, &trips, &persons, &[existing], Some(self_id)).is_ok()
        );
    }

    #[test]
    fn test_val_te_zero_cost_returns_error() {
        let trip_id = Uuid::new_v4();
        let person_id = Uuid::new_v4();
        let trips = vec![make_trip(trip_id, 2)];
        let persons = vec![make_person(person_id)];
        let mut dto = trip_execution_dto(trip_id, person_id, 1);
        dto.actual_cost_eur = dec!(0);
        assert!(validate_trip_execution(&dto, &trips, &persons, &[], None).is_err());
    }

    // ─── validate_equipment_procurement ─────────────────────────────────

    fn make_equipment_item(id: Uuid) -> EquipmentItem {
        EquipmentItem {
            id,
            name: "Laptop".to_string(),
            purchase_cost_eur: dec!(2000),
            useful_lifetime_months: 36,
            grant_usage_pct: dec!(100),
            grant_usage_months: 36,
            work_package_id: 1,
        }
    }

    fn equipment_procurement_dto(equipment_item_id: Uuid) -> EquipmentProcurementInputDto {
        EquipmentProcurementInputDto {
            equipment_item_id,
            actual_purchase_cost_eur: dec!(2000),
            purchase_date: "2026-02-01".to_string(),
            delivery_confirmed: true,
        }
    }

    #[test]
    fn test_val_ep_valid() {
        let item_id = Uuid::new_v4();
        let items = vec![make_equipment_item(item_id)];
        assert!(
            validate_equipment_procurement(&equipment_procurement_dto(item_id), &items).is_ok()
        );
    }

    #[test]
    fn test_val_ep_unknown_item_returns_error() {
        assert!(
            validate_equipment_procurement(&equipment_procurement_dto(Uuid::new_v4()), &[])
                .is_err()
        );
    }

    #[test]
    fn test_val_ep_zero_cost_returns_error() {
        let item_id = Uuid::new_v4();
        let items = vec![make_equipment_item(item_id)];
        let mut dto = equipment_procurement_dto(item_id);
        dto.actual_purchase_cost_eur = dec!(0);
        assert!(validate_equipment_procurement(&dto, &items).is_err());
    }

    #[test]
    fn test_val_ep_invalid_date_returns_error() {
        let item_id = Uuid::new_v4();
        let items = vec![make_equipment_item(item_id)];
        let mut dto = equipment_procurement_dto(item_id);
        dto.purchase_date = "not-a-date".to_string();
        assert!(validate_equipment_procurement(&dto, &items).is_err());
    }

    // ─── validate_actual_cost_entry ─────────────────────────────────────

    fn make_other_cost_item(id: Uuid) -> OtherDirectCostItem {
        OtherDirectCostItem {
            id,
            name: "Publication fees".to_string(),
            amount_eur: dec!(1000),
            is_cfs_item: false,
            notes: None,
            work_package_ids: vec![1],
        }
    }

    fn actual_cost_entry_dto(linked_entity_id: Option<Uuid>) -> ActualCostEntryInputDto {
        ActualCostEntryInputDto {
            linked_entity_id,
            amount_eur: dec!(500),
            description: "Open-access fee".to_string(),
            incurred_date: "2026-02-01".to_string(),
            status: crate::domain::enums::EntryStatus::Approved,
            justification: None,
        }
    }

    #[test]
    fn test_val_ace_valid_linked() {
        let item_id = Uuid::new_v4();
        let items = vec![make_other_cost_item(item_id)];
        assert!(validate_actual_cost_entry(&actual_cost_entry_dto(Some(item_id)), &items).is_ok());
    }

    #[test]
    fn test_val_ace_unknown_linked_item_returns_error() {
        assert!(
            validate_actual_cost_entry(&actual_cost_entry_dto(Some(Uuid::new_v4())), &[]).is_err()
        );
    }

    #[test]
    fn test_val_ace_unbudgeted_without_justification_returns_error() {
        assert!(validate_actual_cost_entry(&actual_cost_entry_dto(None), &[]).is_err());
    }

    #[test]
    fn test_val_ace_unbudgeted_with_justification_is_ok() {
        let mut dto = actual_cost_entry_dto(None);
        dto.justification = Some("Unplanned translation service.".to_string());
        assert!(validate_actual_cost_entry(&dto, &[]).is_ok());
    }

    #[test]
    fn test_val_ace_zero_amount_returns_error() {
        let mut dto = actual_cost_entry_dto(None);
        dto.justification = Some("Justified".to_string());
        dto.amount_eur = dec!(0);
        assert!(validate_actual_cost_entry(&dto, &[]).is_err());
    }

    // ─── validate_subcontracting_line ───────────────────────────────────

    fn subcontracting_line_dto(amount: Decimal) -> SubcontractingLineInputDto {
        SubcontractingLineInputDto {
            vendor: "Acme Labs".to_string(),
            contract_reference: "CTR-001".to_string(),
            amount_eur: amount,
            work_package_id: 1,
            status: crate::domain::enums::EntryStatus::Approved,
            vendor_is_host_institution: false,
            payment_date: None,
        }
    }

    #[test]
    fn test_val_sl_valid() {
        let dto = subcontracting_line_dto(dec!(1000));
        assert!(validate_subcontracting_line(&dto, &[], dec!(5000), 3, None).is_ok());
    }

    #[test]
    fn test_val_sl_empty_vendor_returns_error() {
        let mut dto = subcontracting_line_dto(dec!(1000));
        dto.vendor = "".to_string();
        assert!(validate_subcontracting_line(&dto, &[], dec!(5000), 3, None).is_err());
    }

    #[test]
    fn test_val_sl_exceeds_planned_cap_returns_error() {
        let dto = subcontracting_line_dto(dec!(6000));
        assert!(validate_subcontracting_line(&dto, &[], dec!(5000), 3, None).is_err());
    }

    #[test]
    fn test_val_sl_cap_check_sums_existing_lines() {
        let existing = SubcontractingLine {
            id: Uuid::new_v4(),
            vendor: "Existing Vendor".to_string(),
            contract_reference: "CTR-000".to_string(),
            amount_eur: dec!(4000),
            work_package_id: 1,
            status: crate::domain::enums::EntryStatus::Approved,
            vendor_is_host_institution: false,
            payment_date: None,
        };
        let dto = subcontracting_line_dto(dec!(1500));
        // 4000 existing + 1500 new = 5500 > 5000 planned.
        assert!(validate_subcontracting_line(&dto, &[existing], dec!(5000), 3, None).is_err());
    }

    #[test]
    fn test_val_sl_update_excludes_self_from_cap_check() {
        let self_id = Uuid::new_v4();
        let existing = SubcontractingLine {
            id: self_id,
            vendor: "Existing Vendor".to_string(),
            contract_reference: "CTR-000".to_string(),
            amount_eur: dec!(4000),
            work_package_id: 1,
            status: crate::domain::enums::EntryStatus::Approved,
            vendor_is_host_institution: false,
            payment_date: None,
        };
        let dto = subcontracting_line_dto(dec!(4500));
        assert!(
            validate_subcontracting_line(&dto, &[existing], dec!(5000), 3, Some(self_id)).is_ok()
        );
    }

    #[test]
    fn test_val_sl_wp_out_of_range_returns_error() {
        let mut dto = subcontracting_line_dto(dec!(1000));
        dto.work_package_id = 9;
        assert!(validate_subcontracting_line(&dto, &[], dec!(5000), 3, None).is_err());
    }

    // ─── validate_deliverable ────────────────────────────────────────────

    fn make_deliverable(status: DeliverableStatus) -> Deliverable {
        Deliverable {
            id: Uuid::new_v4(),
            deliverable_number: "D1.1".to_string(),
            title: "Data Collection Protocol".to_string(),
            deliverable_type: crate::domain::enums::DeliverableType::Dataset,
            work_package_id: 1,
            planned_month: 6,
            responsible_role_id: Uuid::new_v4(),
            dissemination_level: crate::domain::enums::DisseminationLevel::Public,
            status,
            actual_submission_date: None,
            revision_note: None,
            revised_planned_month: None,
            cordis_registered: false,
            notes: None,
        }
    }

    fn deliverable_dto(role_id: Uuid) -> DeliverableInputDto {
        DeliverableInputDto {
            title: "Data Collection Protocol".to_string(),
            deliverable_type: crate::domain::enums::DeliverableType::Dataset,
            work_package_id: 1,
            planned_month: 6,
            responsible_role_id: role_id,
            dissemination_level: crate::domain::enums::DisseminationLevel::Public,
            status: DeliverableStatus::NotStarted,
            actual_submission_date: None,
            revision_note: None,
            revised_planned_month: None,
            cordis_registered: false,
            notes: None,
        }
    }

    #[test]
    fn test_val_del_valid() {
        let role_id = Uuid::new_v4();
        let roles = vec![make_role(role_id, 1, 12)];
        assert!(validate_deliverable(&deliverable_dto(role_id), 3, 36, &roles).is_ok());
    }

    #[test]
    fn test_val_del_empty_title_returns_error() {
        let role_id = Uuid::new_v4();
        let roles = vec![make_role(role_id, 1, 12)];
        let mut dto = deliverable_dto(role_id);
        dto.title = "".to_string();
        assert!(validate_deliverable(&dto, 3, 36, &roles).is_err());
    }

    #[test]
    fn test_val_del_wp_out_of_range_returns_error() {
        let role_id = Uuid::new_v4();
        let roles = vec![make_role(role_id, 1, 12)];
        let mut dto = deliverable_dto(role_id);
        dto.work_package_id = 9;
        assert!(validate_deliverable(&dto, 3, 36, &roles).is_err());
    }

    #[test]
    fn test_val_del_month_out_of_range_returns_error() {
        let role_id = Uuid::new_v4();
        let roles = vec![make_role(role_id, 1, 12)];
        let mut dto = deliverable_dto(role_id);
        dto.planned_month = 99;
        assert!(validate_deliverable(&dto, 3, 36, &roles).is_err());
    }

    #[test]
    fn test_val_del_unknown_responsible_role_returns_error() {
        assert!(validate_deliverable(&deliverable_dto(Uuid::new_v4()), 3, 36, &[]).is_err());
    }

    #[test]
    fn test_val_del_invalid_submission_date_returns_error() {
        let role_id = Uuid::new_v4();
        let roles = vec![make_role(role_id, 1, 12)];
        let mut dto = deliverable_dto(role_id);
        dto.actual_submission_date = Some("not-a-date".to_string());
        assert!(validate_deliverable(&dto, 3, 36, &roles).is_err());
    }

    #[test]
    fn test_val_del_rejected_without_revision_note_returns_error() {
        let role_id = Uuid::new_v4();
        let roles = vec![make_role(role_id, 1, 12)];
        let mut dto = deliverable_dto(role_id);
        dto.status = DeliverableStatus::Rejected;
        assert!(validate_deliverable(&dto, 3, 36, &roles).is_err());
    }

    #[test]
    fn test_val_del_rejected_with_revision_note_and_month_is_ok() {
        let role_id = Uuid::new_v4();
        let roles = vec![make_role(role_id, 1, 12)];
        let mut dto = deliverable_dto(role_id);
        dto.status = DeliverableStatus::Rejected;
        dto.revision_note = Some("Needs more detail on methodology.".to_string());
        dto.revised_planned_month = Some(9);
        assert!(validate_deliverable(&dto, 3, 36, &roles).is_ok());
    }

    #[test]
    fn test_val_del_revised_month_out_of_range_returns_error() {
        let role_id = Uuid::new_v4();
        let roles = vec![make_role(role_id, 1, 12)];
        let mut dto = deliverable_dto(role_id);
        dto.status = DeliverableStatus::Rejected;
        dto.revision_note = Some("Needs rework.".to_string());
        dto.revised_planned_month = Some(99);
        assert!(validate_deliverable(&dto, 3, 36, &roles).is_err());
    }

    // ─── validate_reporting_period ──────────────────────────────────────

    fn reporting_period_dto(start: u32, end: u32) -> ReportingPeriodInputDto {
        ReportingPeriodInputDto {
            start_month: start,
            end_month: end,
            submission_deadline: None,
            technical_report_submitted: false,
            financial_report_submitted: false,
            status: crate::domain::enums::ReportingPeriodStatus::Open,
        }
    }

    fn make_reporting_period(start: u32, end: u32) -> ReportingPeriod {
        ReportingPeriod {
            id: Uuid::new_v4(),
            period_number: 1,
            start_month: start,
            end_month: end,
            submission_deadline: None,
            technical_report_submitted: false,
            financial_report_submitted: false,
            status: crate::domain::enums::ReportingPeriodStatus::Open,
        }
    }

    #[test]
    fn test_val_rp_valid() {
        assert!(validate_reporting_period(&reporting_period_dto(1, 18), &[], 36, None).is_ok());
    }

    #[test]
    fn test_val_rp_start_out_of_range_returns_error() {
        assert!(validate_reporting_period(&reporting_period_dto(0, 18), &[], 36, None).is_err());
    }

    #[test]
    fn test_val_rp_end_before_start_returns_error() {
        assert!(validate_reporting_period(&reporting_period_dto(18, 10), &[], 36, None).is_err());
    }

    #[test]
    fn test_val_rp_overlaps_existing_returns_error() {
        let existing = make_reporting_period(1, 18);
        assert!(
            validate_reporting_period(&reporting_period_dto(15, 24), &[existing], 36, None)
                .is_err()
        );
    }

    #[test]
    fn test_val_rp_update_excludes_self_from_overlap_check() {
        let self_id = Uuid::new_v4();
        let mut existing = make_reporting_period(1, 18);
        existing.id = self_id;
        assert!(validate_reporting_period(
            &reporting_period_dto(1, 20),
            &[existing],
            36,
            Some(self_id)
        )
        .is_ok());
    }

    #[test]
    fn test_val_rp_invalid_deadline_returns_error() {
        let mut dto = reporting_period_dto(1, 18);
        dto.submission_deadline = Some("not-a-date".to_string());
        assert!(validate_reporting_period(&dto, &[], 36, None).is_err());
    }

    #[test]
    fn test_val_rp_submitted_without_both_flags_returns_error() {
        let mut dto = reporting_period_dto(1, 18);
        dto.status = crate::domain::enums::ReportingPeriodStatus::Submitted;
        dto.technical_report_submitted = true;
        assert!(validate_reporting_period(&dto, &[], 36, None).is_err());
    }

    #[test]
    fn test_val_rp_submitted_with_both_flags_is_ok() {
        let mut dto = reporting_period_dto(1, 18);
        dto.status = crate::domain::enums::ReportingPeriodStatus::Submitted;
        dto.technical_report_submitted = true;
        dto.financial_report_submitted = true;
        assert!(validate_reporting_period(&dto, &[], 36, None).is_ok());
    }

    // ─── validate_risk_entry ─────────────────────────────────────────────

    fn today() -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()
    }

    fn risk_dto(probability: Level, impact: Level) -> RiskEntryInputDto {
        RiskEntryInputDto {
            title: "Key researcher departure".to_string(),
            description: "PostDoc may leave for industry position.".to_string(),
            work_package_id: Some(1),
            probability,
            impact,
            mitigation: None,
            status: RiskStatus::Open,
            owner_role_id: None,
            identified_date: "2026-01-01".to_string(),
            review_date: None,
            closed_date: None,
        }
    }

    #[test]
    fn test_val_risk_low_priority_no_review_date_is_ok() {
        let dto = risk_dto(Level::Low, Level::Low);
        assert!(validate_risk_entry(&dto, &[], 3, None, today()).is_ok());
    }

    #[test]
    fn test_val_risk_empty_title_returns_error() {
        let mut dto = risk_dto(Level::Low, Level::Low);
        dto.title = "".to_string();
        assert!(validate_risk_entry(&dto, &[], 3, None, today()).is_err());
    }

    #[test]
    fn test_val_risk_wp_out_of_range_returns_error() {
        let mut dto = risk_dto(Level::Low, Level::Low);
        dto.work_package_id = Some(9);
        assert!(validate_risk_entry(&dto, &[], 3, None, today()).is_err());
    }

    #[test]
    fn test_val_risk_unknown_owner_returns_error() {
        let mut dto = risk_dto(Level::Low, Level::Low);
        dto.owner_role_id = Some(Uuid::new_v4());
        assert!(validate_risk_entry(&dto, &[], 3, None, today()).is_err());
    }

    #[test]
    fn test_val_risk_high_priority_without_review_date_returns_error() {
        // Medium × High = 6 → High priority (BR-RK-02).
        let dto = risk_dto(Level::Medium, Level::High);
        assert!(validate_risk_entry(&dto, &[], 3, None, today()).is_err());
    }

    #[test]
    fn test_val_risk_high_priority_with_near_review_date_is_ok() {
        let mut dto = risk_dto(Level::High, Level::High);
        dto.review_date = Some("2026-06-15".to_string());
        assert!(validate_risk_entry(&dto, &[], 3, None, today()).is_ok());
    }

    #[test]
    fn test_val_risk_high_priority_review_date_too_far_out_returns_error() {
        let mut dto = risk_dto(Level::High, Level::High);
        dto.review_date = Some("2026-12-01".to_string());
        assert!(validate_risk_entry(&dto, &[], 3, None, today()).is_err());
    }

    #[test]
    fn test_val_risk_cannot_reopen_closed_risk_returns_error() {
        let mut dto = risk_dto(Level::Low, Level::Low);
        dto.status = RiskStatus::Open;
        assert!(validate_risk_entry(&dto, &[], 3, Some(RiskStatus::Closed), today()).is_err());
    }

    #[test]
    fn test_val_risk_keeping_closed_risk_closed_is_ok() {
        let mut dto = risk_dto(Level::Low, Level::Low);
        dto.status = RiskStatus::Closed;
        assert!(validate_risk_entry(&dto, &[], 3, Some(RiskStatus::Closed), today()).is_ok());
    }

    // ─── validate_issue_entry ────────────────────────────────────────────

    fn issue_dto() -> IssueEntryInputDto {
        IssueEntryInputDto {
            description: "Equipment delivery delayed".to_string(),
            work_package_id: Some(1),
            raised_date: "2026-05-01".to_string(),
            priority: Level::Medium,
            owner_role_id: None,
            status: IssueStatus::Open,
            resolution: None,
            linked_risk_id: None,
        }
    }

    #[test]
    fn test_val_issue_valid() {
        assert!(validate_issue_entry(&issue_dto(), &[], 3, &[], today()).is_ok());
    }

    #[test]
    fn test_val_issue_empty_description_returns_error() {
        let mut dto = issue_dto();
        dto.description = "".to_string();
        assert!(validate_issue_entry(&dto, &[], 3, &[], today()).is_err());
    }

    #[test]
    fn test_val_issue_wp_out_of_range_returns_error() {
        let mut dto = issue_dto();
        dto.work_package_id = Some(9);
        assert!(validate_issue_entry(&dto, &[], 3, &[], today()).is_err());
    }

    #[test]
    fn test_val_issue_raised_date_in_future_returns_error() {
        let mut dto = issue_dto();
        dto.raised_date = "2026-12-01".to_string();
        assert!(validate_issue_entry(&dto, &[], 3, &[], today()).is_err());
    }

    #[test]
    fn test_val_issue_unknown_linked_risk_returns_error() {
        let mut dto = issue_dto();
        dto.linked_risk_id = Some(Uuid::new_v4());
        assert!(validate_issue_entry(&dto, &[], 3, &[], today()).is_err());
    }

    #[test]
    fn test_val_issue_known_linked_risk_is_ok() {
        let risk = RiskEntry {
            id: Uuid::new_v4(),
            title: "Risk".to_string(),
            description: "D".to_string(),
            work_package_id: None,
            probability: Level::Low,
            impact: Level::Low,
            mitigation: None,
            status: RiskStatus::Open,
            owner_role_id: None,
            identified_date: "2026-01-01".to_string(),
            review_date: None,
            closed_date: None,
        };
        let mut dto = issue_dto();
        dto.linked_risk_id = Some(risk.id);
        assert!(validate_issue_entry(&dto, &[], 3, &[risk], today()).is_ok());
    }

    #[test]
    fn test_val_issue_closed_without_resolution_returns_error() {
        let mut dto = issue_dto();
        dto.status = IssueStatus::Closed;
        assert!(validate_issue_entry(&dto, &[], 3, &[], today()).is_err());
    }

    #[test]
    fn test_val_issue_closed_with_resolution_is_ok() {
        let mut dto = issue_dto();
        dto.status = IssueStatus::Closed;
        dto.resolution = Some("Delivery rescheduled and confirmed.".to_string());
        assert!(validate_issue_entry(&dto, &[], 3, &[], today()).is_ok());
    }
}
