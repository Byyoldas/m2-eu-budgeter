//! Status/type enums for Sprint E2 execution-tracking entities.

use serde::{Deserialize, Serialize};

/// Derived (never stored) status of a Work Package. See
/// `docs/executer/execution-requirements.md` BR-WP-04.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WpStatus {
    NotStarted,
    OnTrack,
    AtRisk,
    Delayed,
    Completed,
}

/// User-settable milestone status (BR-MS §M-06). `AtRisk` is never stored —
/// it is a derived overlay applied by `progress_engine::derive_milestone_status`
/// on top of a stored `NotStarted` status (BR-MS-01), so it is rejected by
/// `validate_milestone` as a direct input value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MilestoneStatus {
    NotStarted,
    OnTrack,
    AtRisk,
    Delayed,
    Completed,
    Cancelled,
}

/// Amendment Management is not specified anywhere in
/// `docs/executer/execution-requirements.md` — `development-roadmap.md`'s
/// Sprint E2 table names it (with an `Amendment` entity and a
/// `record_amendment` command) but no business rules, DTOs, or UX exist for
/// it. This type catalogue is a from-scratch design covering the kinds of
/// formal Horizon Europe grant amendment a PI would need to log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmendmentType {
    BudgetReallocation,
    DurationExtension,
    WorkPackageScopeChange,
    PersonnelChange,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmendmentStatus {
    Requested,
    Approved,
    Rejected,
}

/// Shared approval-state enum for Sprint E3's actual-cost tracking entities
/// (`TripExecution`, `EquipmentProcurement`, `ActualCostEntry`,
/// `SubcontractingLine`). Only `Approved` entries count toward actuals
/// (BR-TR-05, BR-EQ-04's `delivery_confirmed` plays the equivalent role for
/// equipment, BR-FIN-01/02).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryStatus {
    Pending,
    Approved,
    Rejected,
}

/// M-05 deliverable type catalogue (`execution-requirements.md` §M-05).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliverableType {
    Report,
    Dataset,
    Software,
    Prototype,
    Dem,
    Ethics,
    Other,
}

/// M-05 dissemination level (`execution-requirements.md` §M-05).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisseminationLevel {
    Public,
    RestrictedToProgramme,
    Confidential,
}

/// M-05 status flow: `NotStarted` → `InProgress` → `Submitted` →
/// `Accepted` | `Rejected` → `Revised` → `Submitted`. Not enforced as a
/// strict state machine (same looseness as `WpStatus`/`AmendmentStatus`) —
/// only BR-DEL-03 (`Rejected` requires a revision note) is validated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliverableStatus {
    NotStarted,
    InProgress,
    Submitted,
    Accepted,
    Rejected,
    Revised,
}

/// M-14 reporting period status. BR-RP-03: a period cannot move to
/// `Submitted` unless both report-submission flags are set — see
/// `validation::validate_reporting_period`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportingPeriodStatus {
    Open,
    Submitted,
}
