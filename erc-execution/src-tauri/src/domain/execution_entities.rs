//! Execution-specific domain entities, stored opaquely (as JSON) inside the
//! shared `.ercbudget` file's `execution_data` block (see
//! `erc_core::persistence::ProjectFile`). Grows with each Sprint E2–E5 module.
//!
//! Sprint E2 scope note: `PersonMonthRecord` tracks by calendar `project_month`
//! rather than by "reporting period" (`docs/executer/execution-requirements.md`
//! BR-PM-02/03/05 are written in terms of periods) because Reporting Period
//! Management (M-14) has not been built yet. The per-month tolerance checks in
//! `validation::validate_person_month_record` apply the same BR-PM-03/04/05
//! numeric rules per calendar month instead; this can be re-expressed in
//! terms of periods once M-14 exists without changing the stored data shape.

use super::enums::{AmendmentStatus, AmendmentType, MilestoneStatus};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// All execution-tracking state for one project. `Default` is used both for
/// brand-new (never-opened-in-Execution-App) projects and as the schema
/// anchor for future migrations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionData {
    pub schema_version: String,
    #[serde(default)]
    pub persons: Vec<Person>,
    #[serde(default)]
    pub person_month_records: Vec<PersonMonthRecord>,
    #[serde(default)]
    pub work_package_executions: Vec<WorkPackageExecution>,
    #[serde(default)]
    pub milestones: Vec<Milestone>,
    #[serde(default)]
    pub amendments: Vec<Amendment>,
}

impl Default for ExecutionData {
    fn default() -> Self {
        Self {
            schema_version: "1.0".to_string(),
            persons: Vec::new(),
            person_month_records: Vec::new(),
            work_package_executions: Vec::new(),
            milestones: Vec::new(),
            amendments: Vec::new(),
        }
    }
}

/// A named individual linked to a planned `PersonnelRole` (M-03).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Person {
    pub id: Uuid,
    pub full_name: String,
    pub email: Option<String>,
    pub institution: Option<String>,
    pub orcid: Option<String>,
    /// References `erc_core::domain::entities::PersonnelRole.id`. BR-PM-01:
    /// at most one `Person` may link to a given role at a time — enforced by
    /// `validation::validate_person`.
    pub linked_role_id: Uuid,
    pub actual_start_date: String,
    pub actual_end_date: Option<String>,
}

/// One calendar project-month's reported/approved FTE-months for a person
/// (M-03). See the module-level note on the period-vs-month scoping decision.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PersonMonthRecord {
    pub id: Uuid,
    pub person_id: Uuid,
    pub project_month: u32,
    /// BR-PM-03: must be ≤ 1.0 (full-time-equivalent cap for one calendar month).
    #[serde(with = "rust_decimal::serde::str")]
    pub reported_months: Decimal,
    /// BR-PM-04: must be ≤ `reported_months`.
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub approved_months: Option<Decimal>,
}

/// Execution-side data attached to a Budget App work package. The WP's
/// identity, dates, and planned budget are read-only, sourced from
/// `ProjectConfig`/`BudgetSummaryDto`; this only carries what the Execution
/// App itself adds (M-04).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkPackageExecution {
    pub work_package_id: u8,
    /// References `erc_core::domain::entities::PersonnelRole.id`.
    pub leader_role_id: Option<Uuid>,
    pub notes: Option<String>,
}

/// A project milestone (M-06). `status` is user-settable and never stores
/// `AtRisk` directly — see `enums::MilestoneStatus` and
/// `progress_engine::derive_milestone_status`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Milestone {
    pub id: Uuid,
    pub title: String,
    pub work_package_id: u8,
    pub planned_month: u32,
    pub status: MilestoneStatus,
    pub actual_completion_month: Option<u32>,
}

/// A formal Horizon Europe grant amendment (scope, budget, duration, or
/// personnel change). See `enums::AmendmentType`/`AmendmentStatus` doc
/// comments for why this module was designed from scratch.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Amendment {
    pub id: Uuid,
    /// Sequential, immutable once assigned: "AMD-1", "AMD-2", ... — see
    /// `commands::amendments::record_amendment`.
    pub amendment_number: String,
    pub amendment_type: AmendmentType,
    pub title: String,
    pub description: String,
    pub requested_date: String,
    pub decision_date: Option<String>,
    pub status: AmendmentStatus,
    /// Informational only — logged for audit purposes. Never modifies
    /// `BudgetSummaryDto`; the Budget App remains the sole source of truth
    /// for planned figures (see `execution-architecture.md` §1 "Independence").
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub financial_impact_eur: Option<Decimal>,
    #[serde(default)]
    pub affected_work_package_ids: Vec<u8>,
    pub notes: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_sets_schema_version_and_empty_collections() {
        let data = ExecutionData::default();
        assert_eq!(data.schema_version, "1.0");
        assert!(data.persons.is_empty());
        assert!(data.person_month_records.is_empty());
        assert!(data.work_package_executions.is_empty());
        assert!(data.milestones.is_empty());
        assert!(data.amendments.is_empty());
    }

    #[test]
    fn test_serde_roundtrip() {
        let data = ExecutionData::default();
        let json = serde_json::to_string(&data).unwrap();
        let reloaded: ExecutionData = serde_json::from_str(&json).unwrap();
        assert_eq!(data, reloaded);
    }

    #[test]
    fn test_sprint_e1_era_json_without_new_fields_still_deserialises() {
        // A file saved by the Sprint E1 build of erc-execution only ever had
        // `schema_version` — confirms the new `#[serde(default)]` fields
        // don't break loading those files.
        let json = r#"{"schema_version":"1.0"}"#;
        let data: ExecutionData = serde_json::from_str(json).unwrap();
        assert_eq!(data, ExecutionData::default());
    }
}
