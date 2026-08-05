//! Execution-specific DTOs returned over IPC. Sprint E1 built enough to
//! render the dashboard shell (project header + planned budget). Sprint E2
//! adds Personnel/Person-Month Tracking (M-03), Work Package Management
//! (M-04), Milestone Tracking (M-06), and Amendment Management (from-scratch
//! design — see `enums::AmendmentType` doc comment). Later sprints extend
//! `ExecutionProjectSummaryDto` further with `actuals`/`warnings` per
//! `docs/executer/execution-architecture.md` §11.

use super::enums::{
    AmendmentStatus, AmendmentType, DeliverableStatus, DeliverableType, DisseminationLevel,
    EntryStatus, IssueStatus, Level, MilestoneStatus, NavigationTarget, ReportingPeriodStatus,
    RiskStatus, WarningSeverity, WpStatus,
};
use erc_core::domain::dto::{BudgetSummaryDto, CfsStatus};
use erc_core::domain::entities::RoleType;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Read-only administrative facts about the project, sourced from the
/// Budget App's `ProjectConfig` (never edited from the Execution App).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfoDto {
    pub project_title: String,
    pub pi_name: String,
    pub call_reference: String,
    pub duration_years: u8,
    pub work_package_count: u8,
}

/// A lightweight projection of a Budget App `PersonnelRole`, just enough for
/// the Execution App's "link to role" / "assign leader" dropdowns. Not a
/// business-logic type — a UI convenience view over read-only Budget App data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonnelRoleSummaryDto {
    pub id: Uuid,
    pub role_label: String,
    pub role_type: RoleType,
}

/// The Execution App's master summary DTO, returned by every command that
/// opens or mutates the project (same full-recalculation pattern as the
/// Budget App's `BudgetSummaryDto`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionProjectSummaryDto {
    pub project_info: ProjectInfoDto,
    pub planned: BudgetSummaryDto,
    pub current_project_month: u32,
    pub personnel_roles: Vec<PersonnelRoleSummaryDto>,
    pub planned_trips: Vec<PlannedTripSummaryDto>,
    pub planned_equipment: Vec<PlannedEquipmentSummaryDto>,
    pub planned_other_costs: Vec<PlannedOtherCostSummaryDto>,
    pub persons: Vec<PersonDetailDto>,
    pub person_months: Vec<PersonMonthDetailDto>,
    pub work_packages: Vec<WorkPackageExecutionDetailDto>,
    pub milestones: Vec<MilestoneDetailDto>,
    pub amendments: Vec<AmendmentDetailDto>,
    pub actuals: ActualFinancialsDto,
    pub trip_executions: Vec<TripExecutionDetailDto>,
    pub equipment_procurements: Vec<EquipmentProcurementDetailDto>,
    pub actual_cost_entries: Vec<ActualCostEntryDetailDto>,
    pub subcontracting_lines: Vec<SubcontractingLineDetailDto>,
    pub deliverables: Vec<DeliverableDetailDto>,
    pub reporting_periods: Vec<ReportingPeriodDetailDto>,
    pub reporting_period_coverage: ReportingPeriodCoverageDto,
    pub risks: Vec<RiskEntryDetailDto>,
    pub issues: Vec<IssueEntryDetailDto>,
    pub warnings: Vec<WarningDto>,
}

/// UI-convenience projections of read-only Budget App entities, just enough
/// for the Execution App's "link to planned item" dropdowns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedTripSummaryDto {
    pub id: Uuid,
    pub name: String,
    pub number_of_instances: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedEquipmentSummaryDto {
    pub id: Uuid,
    pub name: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub planned_cost_eur: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedOtherCostSummaryDto {
    pub id: Uuid,
    pub name: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount_eur: Decimal,
}

// ─── Personnel & Person-Month Tracking (M-03) ─────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonInputDto {
    pub full_name: String,
    pub email: Option<String>,
    pub institution: Option<String>,
    pub orcid: Option<String>,
    pub linked_role_id: Uuid,
    pub actual_start_date: String,
    pub actual_end_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonDetailDto {
    pub id: Uuid,
    pub full_name: String,
    pub email: Option<String>,
    pub institution: Option<String>,
    pub orcid: Option<String>,
    pub linked_role_id: Uuid,
    pub linked_role_label: String,
    pub actual_start_date: String,
    pub actual_end_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonMonthRecordInputDto {
    pub person_id: Uuid,
    pub project_month: u32,
    #[serde(with = "rust_decimal::serde::str")]
    pub reported_months: Decimal,
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub approved_months: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonMonthDetailDto {
    pub id: Uuid,
    pub person_id: Uuid,
    pub project_month: u32,
    #[serde(with = "rust_decimal::serde::str")]
    pub reported_months: Decimal,
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub approved_months: Option<Decimal>,
    /// BR-PM-07: `approved_months × inflation-adjusted monthly salary (EUR)`
    /// for the project year `project_month` falls in. `None` until the
    /// record has an `approved_months` value.
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub salary_cost_estimate_eur: Option<Decimal>,
    /// Derived from `project_month` + `ProjectConfig.call_opening_date`.
    /// `None` when the call opening date is unset (see
    /// `progress_engine::project_month_to_calendar`).
    pub calendar_year: Option<i32>,
    /// 1-12. See `calendar_year`.
    pub calendar_month: Option<u32>,
}

// ─── Work Package Management (M-04) ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkPackageExecutionInputDto {
    pub leader_role_id: Option<Uuid>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkPackageExecutionDetailDto {
    pub work_package_id: u8,
    pub work_package_name: Option<String>,
    pub leader_role_id: Option<Uuid>,
    pub leader_role_label: Option<String>,
    pub notes: Option<String>,
    pub status: WpStatus,
    #[serde(with = "rust_decimal::serde::str")]
    pub planned_eur: Decimal,
    /// Sprint E2 scope: only allocated personnel actuals (BR-PM-07 sums,
    /// allocated across WPs by `erc_core::calculation::personnel_cost::allocate_personnel_cost_by_wp`).
    /// Expands to include C1/C2/C3/B actuals once those tracking modules
    /// (Sprint E3) exist — see `progress_engine::calculate_wp_actual_eur`.
    #[serde(with = "rust_decimal::serde::str")]
    pub actual_eur: Decimal,
    /// BR-WP-03: `actual_eur > planned_eur × 1.05`.
    pub overspend_warning: bool,
}

// ─── Milestone Tracking (M-06) ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneInputDto {
    pub title: String,
    pub work_package_id: u8,
    pub planned_month: u32,
    pub status: MilestoneStatus,
    pub actual_completion_month: Option<u32>,
    /// References `Deliverable.id` (M-05). BR-MS-02.
    #[serde(default)]
    pub linked_deliverable_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneDetailDto {
    pub id: Uuid,
    pub title: String,
    pub work_package_id: u8,
    pub planned_month: u32,
    /// The stored status, exactly as last set by the user.
    pub status: MilestoneStatus,
    /// BR-MS-01's `AtRisk` overlay applied on top of `status` — see
    /// `progress_engine::derive_milestone_status`. Use this for display.
    pub effective_status: MilestoneStatus,
    pub actual_completion_month: Option<u32>,
    pub linked_deliverable_ids: Vec<Uuid>,
}

// ─── Amendment Management (from-scratch design) ───────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmendmentInputDto {
    pub amendment_type: AmendmentType,
    pub title: String,
    pub description: String,
    pub requested_date: String,
    pub decision_date: Option<String>,
    pub status: AmendmentStatus,
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub financial_impact_eur: Option<Decimal>,
    #[serde(default)]
    pub affected_work_package_ids: Vec<u8>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmendmentDetailDto {
    pub id: Uuid,
    pub amendment_number: String,
    pub amendment_type: AmendmentType,
    pub title: String,
    pub description: String,
    pub requested_date: String,
    pub decision_date: Option<String>,
    pub status: AmendmentStatus,
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub financial_impact_eur: Option<Decimal>,
    pub affected_work_package_ids: Vec<u8>,
    pub notes: Option<String>,
}

// ─── M-08: Travel Tracking ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TripExecutionInputDto {
    pub trip_id: Uuid,
    pub instance_number: u32,
    pub traveller_person_id: Uuid,
    pub actual_travel_date: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub actual_cost_eur: Decimal,
    pub status: EntryStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TripExecutionDetailDto {
    pub id: Uuid,
    pub trip_id: Uuid,
    pub trip_name: String,
    pub instance_number: u32,
    pub traveller_person_id: Uuid,
    pub traveller_name: String,
    pub actual_travel_date: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub actual_cost_eur: Decimal,
    pub status: EntryStatus,
    /// The per-instance planned cost (trip's total planned cost ÷ instances),
    /// used for BR-TR-04's >20% overspend check.
    #[serde(with = "rust_decimal::serde::str")]
    pub planned_cost_per_instance_eur: Decimal,
    /// BR-TR-04: `actual_cost_eur > planned_cost_per_instance_eur × 1.20`.
    pub overspend_warning: bool,
}

// ─── M-09: Equipment Tracking ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentProcurementInputDto {
    pub equipment_item_id: Uuid,
    #[serde(with = "rust_decimal::serde::str")]
    pub actual_purchase_cost_eur: Decimal,
    pub purchase_date: String,
    pub delivery_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentProcurementDetailDto {
    pub id: Uuid,
    pub equipment_item_id: Uuid,
    pub equipment_item_name: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub actual_purchase_cost_eur: Decimal,
    pub purchase_date: String,
    pub delivery_confirmed: bool,
    /// BR-EQ-01: CALC-05 recomputed with the actual purchase cost. `None`
    /// until delivery is confirmed (BR-EQ-04).
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub actual_eligible_depreciation_eur: Option<Decimal>,
    /// BR-EQ-02: `actual_purchase_cost_eur > planned_cost_eur × 1.10`.
    pub overspend_warning: bool,
}

// ─── M-10: Other Costs Tracking ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActualCostEntryInputDto {
    pub linked_entity_id: Option<Uuid>,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount_eur: Decimal,
    pub description: String,
    pub incurred_date: String,
    pub status: EntryStatus,
    pub justification: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActualCostEntryDetailDto {
    pub id: Uuid,
    pub linked_entity_id: Option<Uuid>,
    pub linked_entity_name: Option<String>,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount_eur: Decimal,
    pub description: String,
    pub incurred_date: String,
    pub status: EntryStatus,
    pub justification: Option<String>,
    /// BR-OC-02: this item's approved actual total (across all its entries)
    /// exceeds its planned amount × 1.10. `false` for unbudgeted entries.
    pub overspend_warning: bool,
}

// ─── M-11: Subcontracting Tracking ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubcontractingLineInputDto {
    pub vendor: String,
    pub contract_reference: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount_eur: Decimal,
    pub work_package_id: u8,
    pub status: EntryStatus,
    pub vendor_is_host_institution: bool,
    pub payment_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubcontractingLineDetailDto {
    pub id: Uuid,
    pub vendor: String,
    pub contract_reference: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount_eur: Decimal,
    pub work_package_id: u8,
    pub status: EntryStatus,
    pub vendor_is_host_institution: bool,
    pub payment_date: Option<String>,
    /// BR-SC-03 advisory: `amount_eur > €200,000`.
    pub competitive_tender_warning: bool,
    /// BR-SC-04 advisory: `vendor_is_host_institution`.
    pub host_institution_warning: bool,
}

// ─── M-07: Financial Reporting (Planned vs. Actual) ─────────────────────────────

/// The actuals-side counterpart to `BudgetSummaryDto`, reusing the same
/// erc-core aggregation functions (`calculate_indirect_costs`,
/// `calculate_total_direct_costs`, `calculate_total_eligible_costs`,
/// `calculate_requested_contribution`, `check_cfs_threshold`) with actual
/// instead of planned category totals — see `engines::financial_engine`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActualFinancialsDto {
    #[serde(with = "rust_decimal::serde::str")]
    pub a_actual: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub b_actual: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub c1_actual: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub c2_actual: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub c3_actual: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub e_actual: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub total_direct_actual: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub total_eligible_actual: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub requested_eu_contribution_actual: Decimal,
    pub cfs_status_actual: CfsStatus,
    /// BR-FIN-04: actual exceeds planned by more than 15%, per category.
    pub category_a_overrun: bool,
    pub category_b_overrun: bool,
    pub category_c1_overrun: bool,
    pub category_c2_overrun: bool,
    pub category_c3_overrun: bool,
}

// ─── M-05: Deliverable Tracking ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliverableInputDto {
    pub title: String,
    pub deliverable_type: DeliverableType,
    pub work_package_id: u8,
    pub planned_month: u32,
    pub responsible_role_id: Uuid,
    pub dissemination_level: DisseminationLevel,
    pub status: DeliverableStatus,
    pub actual_submission_date: Option<String>,
    pub revision_note: Option<String>,
    pub revised_planned_month: Option<u32>,
    pub cordis_registered: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliverableDetailDto {
    pub id: Uuid,
    pub deliverable_number: String,
    pub title: String,
    pub deliverable_type: DeliverableType,
    pub work_package_id: u8,
    pub planned_month: u32,
    pub responsible_role_id: Uuid,
    pub responsible_role_label: String,
    pub dissemination_level: DisseminationLevel,
    pub status: DeliverableStatus,
    pub actual_submission_date: Option<String>,
    pub revision_note: Option<String>,
    pub revised_planned_month: Option<u32>,
    pub cordis_registered: bool,
    pub notes: Option<String>,
    /// BR-DEL-01: derived, never stored.
    pub is_overdue: bool,
    /// BR-DEL-04 advisory: `Public` dissemination not yet registered in CORDIS.
    pub cordis_warning: bool,
    /// BR-DEL-05: the reporting period whose month range contains this
    /// deliverable's effective planned month (`revised_planned_month` if
    /// set, else `planned_month`), if any periods exist yet.
    pub reporting_period_number: Option<u32>,
}

// ─── M-14: Reporting Period Management ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingPeriodInputDto {
    pub start_month: u32,
    pub end_month: u32,
    pub submission_deadline: Option<String>,
    pub technical_report_submitted: bool,
    pub financial_report_submitted: bool,
    pub status: ReportingPeriodStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingPeriodDetailDto {
    pub id: Uuid,
    pub period_number: u32,
    pub start_month: u32,
    pub end_month: u32,
    pub submission_deadline: Option<String>,
    pub technical_report_submitted: bool,
    pub financial_report_submitted: bool,
    pub status: ReportingPeriodStatus,
    /// Number of deliverables whose effective planned month falls in this
    /// period (BR-DEL-05), and how many of those are already Submitted or
    /// beyond.
    pub deliverables_due: u32,
    pub deliverables_submitted: u32,
}

/// BR-RP-01/02 advisory coverage check across the whole `reporting_periods`
/// list — see `validation::validate_reporting_period`'s doc comment for why
/// this isn't a hard-blocking validator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingPeriodCoverageDto {
    pub gaps_detected: bool,
    pub final_period_covers_project_end: bool,
}

// ─── M-12: Risk Register ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskEntryInputDto {
    pub title: String,
    pub description: String,
    pub work_package_id: Option<u8>,
    pub probability: Level,
    pub impact: Level,
    pub mitigation: Option<String>,
    pub status: RiskStatus,
    pub owner_role_id: Option<Uuid>,
    pub identified_date: String,
    pub review_date: Option<String>,
    pub closed_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskEntryDetailDto {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub work_package_id: Option<u8>,
    pub probability: Level,
    pub impact: Level,
    pub mitigation: Option<String>,
    pub status: RiskStatus,
    pub owner_role_id: Option<Uuid>,
    pub owner_role_label: Option<String>,
    pub identified_date: String,
    pub review_date: Option<String>,
    pub closed_date: Option<String>,
    /// BR-RK-01: `probability × impact`, range 1–9.
    pub risk_score: u8,
    /// BR-RK-02: derived from `risk_score` (≥6 High, 3–5 Medium, 1–2 Low).
    pub priority: Level,
}

// ─── M-13: Issue Log ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueEntryInputDto {
    pub description: String,
    pub work_package_id: Option<u8>,
    pub raised_date: String,
    pub priority: Level,
    pub owner_role_id: Option<Uuid>,
    pub status: IssueStatus,
    pub resolution: Option<String>,
    pub linked_risk_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueEntryDetailDto {
    pub id: Uuid,
    pub description: String,
    pub work_package_id: Option<u8>,
    pub raised_date: String,
    pub priority: Level,
    pub owner_role_id: Option<Uuid>,
    pub owner_role_label: Option<String>,
    pub status: IssueStatus,
    pub resolution: Option<String>,
    pub linked_risk_id: Option<Uuid>,
    /// BR-IS-02: `High` priority, still `Open`, raised more than 14 days ago.
    pub is_stale_warning: bool,
}

// ─── M-21: Notifications & Warnings ─────────────────────────────────────────────

/// One entry in the persistent notification tray (M-21, codes W-01 through
/// W-12). Assembled by `engines::notification_engine::evaluate_warnings`,
/// which re-derives every warning from data already on `ExecutionProjectSummaryDto`
/// rather than introducing a second source of truth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarningDto {
    pub code: String,
    pub severity: WarningSeverity,
    pub message: String,
    pub navigation_target: NavigationTarget,
    pub entity_id: Option<Uuid>,
}
