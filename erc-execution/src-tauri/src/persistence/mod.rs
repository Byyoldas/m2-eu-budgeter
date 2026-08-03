//! Persistence for the Execution Application. Reads/writes the same
//! `.ercbudget` file the Budget Application produces, but — unlike
//! `erc_core::persistence` — deserialises `execution_data` into the typed
//! `ExecutionData` struct instead of leaving it opaque (see
//! `docs/executer/execution-architecture.md` §8). Always writes format
//! version "1.1" since saving from this app always populates the block.

use crate::domain::execution_entities::ExecutionData;
use erc_core::domain::entities::Project;
use erc_core::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const EXECUTION_FORMAT_VERSION: &str = "1.1";

#[derive(Debug, Serialize, Deserialize)]
struct ExecutionProjectFile {
    format_version: String,
    created_at: String,
    updated_at: String,
    project: Project,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    execution_data: Option<ExecutionData>,
}

/// Load a project + its execution data from a `.ercbudget` file.
/// Files with no `execution_data` block (format "1.0", or "1.1" written by
/// something that omitted it) get a fresh `ExecutionData::default()` —
/// this is the "creates an execution_data block" upgrade path from BR-IO-03.
pub fn load_execution(path: &Path) -> Result<(Project, ExecutionData), AppError> {
    let json = std::fs::read_to_string(path).map_err(|e| {
        AppError::Persistence(format!("Failed to read file {}: {e}", path.display()))
    })?;

    let file: ExecutionProjectFile = serde_json::from_str(&json).map_err(|e| {
        AppError::Persistence(format!(
            "Failed to parse project file (is this a valid .ercbudget file?): {e}"
        ))
    })?;

    match file.format_version.as_str() {
        "1.0" | "1.1" => {}
        v => {
            return Err(AppError::Persistence(format!(
                "Unsupported format version: {v}"
            )))
        }
    }

    let exec = file.execution_data.unwrap_or_default();
    Ok((file.project, exec))
}

/// Save a project + its execution data back to a `.ercbudget` file.
pub fn save_execution(
    project: &Project,
    exec: &ExecutionData,
    path: &Path,
) -> Result<(), AppError> {
    let now = chrono::Utc::now().to_rfc3339();
    let created_at = if path.exists() {
        read_created_at(path).unwrap_or_else(|_| now.clone())
    } else {
        now.clone()
    };

    let file = ExecutionProjectFile {
        format_version: EXECUTION_FORMAT_VERSION.to_string(),
        created_at,
        updated_at: now,
        project: project.clone(),
        execution_data: Some(exec.clone()),
    };

    let json = serde_json::to_string_pretty(&file)
        .map_err(|e| AppError::Persistence(format!("Failed to serialise project: {e}")))?;

    std::fs::write(path, json.as_bytes()).map_err(|e| {
        AppError::Persistence(format!("Failed to write file {}: {e}", path.display()))
    })?;

    Ok(())
}

/// Auto-save to a `.ercbudget.autosave` sibling (called after every mutation).
pub fn auto_save(project: &Project, exec: &ExecutionData, path: &Path) -> Result<(), AppError> {
    let auto_path = path.with_extension("ercbudget.autosave");
    save_execution(project, exec, &auto_path)
}

fn read_created_at(path: &Path) -> Result<String, AppError> {
    let json = std::fs::read_to_string(path).map_err(|e| AppError::Persistence(e.to_string()))?;
    let value: serde_json::Value =
        serde_json::from_str(&json).map_err(|e| AppError::Persistence(e.to_string()))?;
    let ts = value
        .get("created_at")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if ts.is_empty() {
        Err(AppError::Persistence("No created_at in file".to_string()))
    } else {
        Ok(ts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use erc_core::domain::entities::ProjectConfig;
    use rust_decimal_macros::dec;

    fn make_project() -> Project {
        let config = ProjectConfig {
            project_title: "Execution Persistence Test".to_string(),
            pi_name: "PI".to_string(),
            call_reference: "ERC-2025-CoG".to_string(),
            duration_years: 1,
            work_package_count: 1,
            work_package_names: vec![None],
            work_package_start_months: vec![1],
            work_package_end_months: vec![12],
            default_inflation_rate_pct: dec!(0),
            try_eur_rate: dec!(50),
            indirect_cost_rate_pct: dec!(25),
            rate_version_id: "from_2025_05_13".to_string(),
            call_opening_date: None,
        };
        Project::new(config)
    }

    fn temp_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "erc-execution-persistence-test-{name}-{}.ercbudget",
            std::process::id()
        ))
    }

    #[test]
    fn test_load_v1_0_file_without_execution_data_defaults_it() {
        let json = r#"{
            "format_version": "1.0",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z",
            "project": {
                "id": "11111111-1111-4111-8111-111111111111",
                "config": {
                    "project_title": "Legacy",
                    "pi_name": "PI",
                    "call_reference": "ERC-2025-CoG",
                    "duration_years": 1,
                    "work_package_count": 1,
                    "work_package_names": [null],
                    "work_package_start_months": [1],
                    "work_package_end_months": [12],
                    "default_inflation_rate_pct": "0",
                    "try_eur_rate": "50",
                    "indirect_cost_rate_pct": "25",
                    "rate_version_id": "from_2025_05_13",
                    "call_opening_date": null
                },
                "personnel_roles": [],
                "equipment_items": [],
                "trips": [],
                "other_cost_items": [],
                "subcontracting": { "amount_eur": "0", "work_package_id": 1 },
                "cfs_warning_dismissed": false
            }
        }"#;
        let path = temp_path("load-v1-0");
        std::fs::write(&path, json).unwrap();
        let (project, exec) = load_execution(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(project.config.project_title, "Legacy");
        assert_eq!(exec, ExecutionData::default());
    }

    #[test]
    fn test_save_and_load_roundtrip_preserves_execution_data() {
        let project = make_project();
        let exec = ExecutionData::default();
        let path = temp_path("roundtrip");

        save_execution(&project, &exec, &path).unwrap();
        let (reloaded_project, reloaded_exec) = load_execution(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(project.id, reloaded_project.id);
        assert_eq!(exec, reloaded_exec);
    }

    #[test]
    fn test_save_and_load_roundtrip_preserves_e2_entities() {
        use crate::domain::enums::{AmendmentStatus, AmendmentType, MilestoneStatus};
        use crate::domain::execution_entities::{
            Amendment, Milestone, Person, PersonMonthRecord, WorkPackageExecution,
        };

        let project = make_project();
        let person_id = uuid::Uuid::new_v4();
        let mut exec = ExecutionData::default();
        exec.persons.push(Person {
            id: person_id,
            full_name: "Ada Lovelace".to_string(),
            email: Some("ada@example.com".to_string()),
            institution: None,
            orcid: None,
            linked_role_id: uuid::Uuid::new_v4(),
            actual_start_date: "2026-01-01".to_string(),
            actual_end_date: None,
        });
        exec.person_month_records.push(PersonMonthRecord {
            id: uuid::Uuid::new_v4(),
            person_id,
            project_month: 1,
            reported_months: dec!(1),
            approved_months: Some(dec!(1)),
        });
        exec.work_package_executions.push(WorkPackageExecution {
            work_package_id: 1,
            leader_role_id: None,
            notes: Some("Leads WP1".to_string()),
        });
        exec.milestones.push(Milestone {
            id: uuid::Uuid::new_v4(),
            title: "Prototype ready".to_string(),
            work_package_id: 1,
            planned_month: 6,
            status: MilestoneStatus::OnTrack,
            actual_completion_month: None,
            linked_deliverable_ids: vec![],
        });
        exec.amendments.push(Amendment {
            id: uuid::Uuid::new_v4(),
            amendment_number: "AMD-1".to_string(),
            amendment_type: AmendmentType::DurationExtension,
            title: "No-cost extension".to_string(),
            description: "Delays due to equipment lead time.".to_string(),
            requested_date: "2026-06-01".to_string(),
            decision_date: None,
            status: AmendmentStatus::Requested,
            financial_impact_eur: None,
            affected_work_package_ids: vec![1],
            notes: None,
        });

        let path = temp_path("e2-entities-roundtrip");
        save_execution(&project, &exec, &path).unwrap();
        let (_, reloaded_exec) = load_execution(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(exec, reloaded_exec);
    }

    #[test]
    fn test_save_and_load_roundtrip_preserves_e3_entities() {
        use crate::domain::enums::EntryStatus;
        use crate::domain::execution_entities::{
            ActualCostEntry, EquipmentProcurement, SubcontractingLine, TripExecution,
        };

        let project = make_project();
        let mut exec = ExecutionData::default();
        exec.trip_executions.push(TripExecution {
            id: uuid::Uuid::new_v4(),
            trip_id: uuid::Uuid::new_v4(),
            instance_number: 1,
            traveller_person_id: uuid::Uuid::new_v4(),
            actual_travel_date: "2026-03-01".to_string(),
            actual_cost_eur: dec!(600),
            status: EntryStatus::Approved,
        });
        exec.equipment_procurements.push(EquipmentProcurement {
            id: uuid::Uuid::new_v4(),
            equipment_item_id: uuid::Uuid::new_v4(),
            actual_purchase_cost_eur: dec!(2000),
            purchase_date: "2026-02-01".to_string(),
            delivery_confirmed: true,
        });
        exec.actual_cost_entries.push(ActualCostEntry {
            id: uuid::Uuid::new_v4(),
            linked_entity_id: None,
            amount_eur: dec!(300),
            description: "Open-access fee".to_string(),
            incurred_date: "2026-02-15".to_string(),
            status: EntryStatus::Approved,
            justification: Some("Unplanned publication.".to_string()),
        });
        exec.subcontracting_lines.push(SubcontractingLine {
            id: uuid::Uuid::new_v4(),
            vendor: "Acme Labs".to_string(),
            contract_reference: "CTR-001".to_string(),
            amount_eur: dec!(5000),
            work_package_id: 1,
            status: EntryStatus::Approved,
            vendor_is_host_institution: false,
            payment_date: Some("2026-03-01".to_string()),
        });

        let path = temp_path("e3-entities-roundtrip");
        save_execution(&project, &exec, &path).unwrap();
        let (_, reloaded_exec) = load_execution(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(exec, reloaded_exec);
    }

    #[test]
    fn test_save_and_load_roundtrip_preserves_e4_entities() {
        use crate::domain::enums::{
            DeliverableStatus, DeliverableType, DisseminationLevel, ReportingPeriodStatus,
        };
        use crate::domain::execution_entities::{Deliverable, ReportingPeriod};

        let project = make_project();
        let mut exec = ExecutionData::default();
        exec.deliverables.push(Deliverable {
            id: uuid::Uuid::new_v4(),
            deliverable_number: "D1.1".to_string(),
            title: "Data Collection Protocol".to_string(),
            deliverable_type: DeliverableType::Dataset,
            work_package_id: 1,
            planned_month: 6,
            responsible_role_id: uuid::Uuid::new_v4(),
            dissemination_level: DisseminationLevel::Public,
            status: DeliverableStatus::Accepted,
            actual_submission_date: Some("2026-06-15".to_string()),
            revision_note: None,
            revised_planned_month: None,
            cordis_registered: true,
            notes: None,
        });
        exec.reporting_periods.push(ReportingPeriod {
            id: uuid::Uuid::new_v4(),
            period_number: 1,
            start_month: 1,
            end_month: 12,
            submission_deadline: Some("2027-02-01".to_string()),
            technical_report_submitted: false,
            financial_report_submitted: false,
            status: ReportingPeriodStatus::Open,
        });

        let path = temp_path("e4-entities-roundtrip");
        save_execution(&project, &exec, &path).unwrap();
        let (_, reloaded_exec) = load_execution(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(exec, reloaded_exec);
    }

    #[test]
    fn test_save_and_load_roundtrip_preserves_e5_entities() {
        use crate::domain::enums::{IssueStatus, Level, RiskStatus};
        use crate::domain::execution_entities::{IssueEntry, RiskEntry};

        let project = make_project();
        let mut exec = ExecutionData::default();
        let risk_id = uuid::Uuid::new_v4();
        exec.risks.push(RiskEntry {
            id: risk_id,
            title: "Key researcher departure".to_string(),
            description: "PostDoc may leave for industry position.".to_string(),
            work_package_id: Some(1),
            probability: Level::Medium,
            impact: Level::High,
            mitigation: Some("Cross-train a second team member.".to_string()),
            status: RiskStatus::Open,
            owner_role_id: None,
            identified_date: "2026-01-01".to_string(),
            review_date: Some("2026-06-15".to_string()),
            closed_date: None,
        });
        exec.issues.push(IssueEntry {
            id: uuid::Uuid::new_v4(),
            description: "Equipment delivery delayed".to_string(),
            work_package_id: Some(2),
            raised_date: "2026-05-01".to_string(),
            priority: Level::High,
            owner_role_id: None,
            status: IssueStatus::Open,
            resolution: None,
            linked_risk_id: Some(risk_id),
        });

        let path = temp_path("e5-entities-roundtrip");
        save_execution(&project, &exec, &path).unwrap();
        let (_, reloaded_exec) = load_execution(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(exec, reloaded_exec);
    }

    #[test]
    fn test_save_writes_format_version_1_1() {
        let project = make_project();
        let exec = ExecutionData::default();
        let path = temp_path("format-version");

        save_execution(&project, &exec, &path).unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        std::fs::remove_file(&path).ok();

        let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(value["format_version"], "1.1");
    }

    #[test]
    fn test_save_preserves_created_at_on_resave() {
        let project = make_project();
        let exec = ExecutionData::default();
        let path = temp_path("preserve-created-at");

        save_execution(&project, &exec, &path).unwrap();
        let first_created_at = read_created_at(&path).unwrap();

        save_execution(&project, &exec, &path).unwrap();
        let second_created_at = read_created_at(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(first_created_at, second_created_at);
    }

    #[test]
    fn test_auto_save_writes_autosave_sibling() {
        let project = make_project();
        let exec = ExecutionData::default();
        let path = temp_path("autosave-base");
        let autosave_path = path.with_extension("ercbudget.autosave");
        std::fs::remove_file(&autosave_path).ok();

        auto_save(&project, &exec, &path).unwrap();
        assert!(autosave_path.exists());
        std::fs::remove_file(&autosave_path).ok();
    }
}
