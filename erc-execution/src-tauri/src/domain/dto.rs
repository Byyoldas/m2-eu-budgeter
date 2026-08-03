//! Execution-specific DTOs returned over IPC. Sprint E1 built enough to
//! render the dashboard shell (project header + planned budget). Sprint E2
//! adds Personnel/Person-Month Tracking (M-03), Work Package Management
//! (M-04), Milestone Tracking (M-06), and Amendment Management (from-scratch
//! design — see `enums::AmendmentType` doc comment). Later sprints extend
//! `ExecutionProjectSummaryDto` further with `actuals`/`warnings` per
//! `docs/executer/execution-architecture.md` §11.

use super::enums::{AmendmentStatus, AmendmentType, MilestoneStatus, WpStatus};
use erc_core::domain::dto::BudgetSummaryDto;
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
    pub persons: Vec<PersonDetailDto>,
    pub person_months: Vec<PersonMonthDetailDto>,
    pub work_packages: Vec<WorkPackageExecutionDetailDto>,
    pub milestones: Vec<MilestoneDetailDto>,
    pub amendments: Vec<AmendmentDetailDto>,
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
