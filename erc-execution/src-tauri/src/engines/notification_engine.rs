//! Notifications & Warnings engine (M-21). `evaluate_warnings` re-derives
//! every warning (W-01 through W-12) from data already assembled elsewhere
//! in `build_summary` — it introduces no second source of truth, only new
//! aggregation over already-derived fields (`is_overdue`, `overspend_warning`,
//! `is_stale_warning`, category overrun flags, ...) plus a handful of checks
//! (W-02, W-03/04, W-08, W-10, W-12) that have no existing derived field.

use crate::domain::dto::{
    ActualFinancialsDto, DeliverableDetailDto, IssueEntryDetailDto, MilestoneDetailDto,
    ReportingPeriodDetailDto, RiskEntryDetailDto, TripExecutionDetailDto, WarningDto,
    WorkPackageExecutionDetailDto,
};
use crate::domain::enums::{Level, NavigationTarget, RiskStatus, WarningSeverity};
use crate::domain::execution_entities::ExecutionData;
use chrono::NaiveDate;
use erc_core::domain::dto::CfsStatus;
use erc_core::domain::entities::Project;

/// BR-RP-03/M-21: a reporting period's deadline is "within 60 days" (W-03,
/// Warning) or "within 14 days" (W-04, Error, supersedes W-03 for the same
/// period). A deadline already in the past also counts as within 14 days.
const REPORTING_DEADLINE_WARNING_DAYS: i64 = 60;
const REPORTING_DEADLINE_ERROR_DAYS: i64 = 14;

pub struct WarningContext<'a> {
    pub project: &'a Project,
    pub exec: &'a ExecutionData,
    pub actuals: &'a ActualFinancialsDto,
    pub deliverables: &'a [DeliverableDetailDto],
    pub milestones: &'a [MilestoneDetailDto],
    pub work_packages: &'a [WorkPackageExecutionDetailDto],
    pub reporting_periods: &'a [ReportingPeriodDetailDto],
    pub risks: &'a [RiskEntryDetailDto],
    pub issues: &'a [IssueEntryDetailDto],
    pub trip_executions: &'a [TripExecutionDetailDto],
    pub current_project_month: u32,
    pub today: NaiveDate,
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

/// The project's last calendar day, when `call_opening_date` is known (same
/// "skip when the calendar anchor is unset" precedent used elsewhere, e.g.
/// BR-PM-06 in `validation::validate_person`).
fn project_end_date(project: &Project) -> Option<NaiveDate> {
    let call_opening_date = project.config.call_opening_date.as_deref()?;
    let base = parse_date(call_opening_date)?;
    let duration_months = project.config.duration_years as u32 * 12;
    base.checked_add_months(chrono::Months::new(duration_months.saturating_sub(1)))
}

pub fn evaluate_warnings(ctx: &WarningContext) -> Vec<WarningDto> {
    let mut warnings = Vec::new();

    // W-01: overdue deliverables.
    for d in ctx.deliverables {
        if d.is_overdue {
            warnings.push(WarningDto {
                code: "W-01".to_string(),
                severity: WarningSeverity::Error,
                message: format!(
                    "{} '{}' is overdue (due M{})",
                    d.deliverable_number, d.title, d.planned_month
                ),
                navigation_target: NavigationTarget::Deliverables,
                entity_id: Some(d.id),
            });
        }
    }

    // W-02: milestone planned month passed with no completion recorded.
    for m in ctx.milestones {
        if m.actual_completion_month.is_none() && m.planned_month < ctx.current_project_month {
            warnings.push(WarningDto {
                code: "W-02".to_string(),
                severity: WarningSeverity::Warning,
                message: format!(
                    "Milestone '{}' passed its planned month (M{}) with no completion recorded",
                    m.title, m.planned_month
                ),
                navigation_target: NavigationTarget::Milestones,
                entity_id: Some(m.id),
            });
        }
    }

    // W-03/W-04: reporting period deadline approaching or overdue.
    for p in ctx.reporting_periods {
        if p.status == crate::domain::enums::ReportingPeriodStatus::Submitted {
            continue;
        }
        let Some(deadline) = p.submission_deadline.as_deref().and_then(parse_date) else {
            continue;
        };
        let days_until = (deadline - ctx.today).num_days();
        if days_until <= REPORTING_DEADLINE_ERROR_DAYS {
            warnings.push(WarningDto {
                code: "W-04".to_string(),
                severity: WarningSeverity::Error,
                message: format!(
                    "Reporting period P{}'s deadline ({}) is within 14 days",
                    p.period_number, deadline
                ),
                navigation_target: NavigationTarget::ReportingPeriods,
                entity_id: Some(p.id),
            });
        } else if days_until <= REPORTING_DEADLINE_WARNING_DAYS {
            warnings.push(WarningDto {
                code: "W-03".to_string(),
                severity: WarningSeverity::Warning,
                message: format!(
                    "Reporting period P{}'s deadline ({}) is within 60 days",
                    p.period_number, deadline
                ),
                navigation_target: NavigationTarget::ReportingPeriods,
                entity_id: Some(p.id),
            });
        }
    }

    // W-05: budget category overrun > 15%.
    for (overrun, label) in [
        (ctx.actuals.category_a_overrun, "Category A (Personnel)"),
        (
            ctx.actuals.category_b_overrun,
            "Category B (Subcontracting)",
        ),
        (ctx.actuals.category_c1_overrun, "Category C1 (Travel)"),
        (ctx.actuals.category_c2_overrun, "Category C2 (Equipment)"),
        (
            ctx.actuals.category_c3_overrun,
            "Category C3 (Other Direct Costs)",
        ),
    ] {
        if overrun {
            warnings.push(WarningDto {
                code: "W-05".to_string(),
                severity: WarningSeverity::Warning,
                message: format!("{label} actual cost exceeds planned by more than 15%"),
                navigation_target: NavigationTarget::Dashboard,
                entity_id: None,
            });
        }
    }

    // W-06: total EU contribution actual exceeds the CFS threshold, unaddressed.
    if ctx.actuals.cfs_status_actual == CfsStatus::RequiredAndUnaddressed {
        warnings.push(WarningDto {
            code: "W-06".to_string(),
            severity: WarningSeverity::Error,
            message: "Total actual EU contribution exceeds €430,000 and CFS is not addressed"
                .to_string(),
            navigation_target: NavigationTarget::Dashboard,
            entity_id: None,
        });
    }

    // W-07: WP budget overrun > 5%.
    for wp in ctx.work_packages {
        if wp.overspend_warning {
            let label = wp
                .work_package_name
                .clone()
                .unwrap_or_else(|| format!("WP{}", wp.work_package_id));
            warnings.push(WarningDto {
                code: "W-07".to_string(),
                severity: WarningSeverity::Warning,
                message: format!("{label}'s actual cost exceeds planned by more than 5%"),
                navigation_target: NavigationTarget::WorkPackages,
                entity_id: None,
            });
        }
    }

    // W-08: high-priority risk review overdue.
    for r in ctx.risks {
        if r.priority == Level::High && r.status != RiskStatus::Closed {
            if let Some(review_date) = r.review_date.as_deref().and_then(parse_date) {
                if review_date < ctx.today {
                    warnings.push(WarningDto {
                        code: "W-08".to_string(),
                        severity: WarningSeverity::Warning,
                        message: format!("High-priority risk '{}' review is overdue", r.title),
                        navigation_target: NavigationTarget::RiskRegister,
                        entity_id: Some(r.id),
                    });
                }
            }
        }
    }

    // W-09: high-priority issue unresolved > 14 days.
    for i in ctx.issues {
        if i.is_stale_warning {
            warnings.push(WarningDto {
                code: "W-09".to_string(),
                severity: WarningSeverity::Warning,
                message: format!(
                    "High-priority issue '{}' has been unresolved for more than 14 days",
                    i.description
                ),
                navigation_target: NavigationTarget::IssueLog,
                entity_id: Some(i.id),
            });
        }
    }

    // W-10: PersonnelRole with no linked Person.
    for role in &ctx.project.personnel_roles {
        if !ctx.exec.persons.iter().any(|p| p.linked_role_id == role.id) {
            warnings.push(WarningDto {
                code: "W-10".to_string(),
                severity: WarningSeverity::Info,
                message: format!("Role '{}' has no linked person", role.role_label),
                navigation_target: NavigationTarget::Personnel,
                entity_id: Some(role.id),
            });
        }
    }

    // W-11: travel instance actual cost > 120% of planned.
    for te in ctx.trip_executions {
        if te.overspend_warning {
            warnings.push(WarningDto {
                code: "W-11".to_string(),
                severity: WarningSeverity::Warning,
                message: format!(
                    "Trip '{}' instance {} actual cost exceeds 120% of planned",
                    te.trip_name, te.instance_number
                ),
                navigation_target: NavigationTarget::Travel,
                entity_id: Some(te.id),
            });
        }
    }

    // W-12: equipment purchase date after project end.
    if let Some(end_date) = project_end_date(ctx.project) {
        for ep in &ctx.exec.equipment_procurements {
            if let Some(purchase_date) = parse_date(&ep.purchase_date) {
                if purchase_date > end_date {
                    warnings.push(WarningDto {
                        code: "W-12".to_string(),
                        severity: WarningSeverity::Error,
                        message: format!(
                            "Equipment purchase on {purchase_date} is after the project's end date"
                        ),
                        navigation_target: NavigationTarget::Equipment,
                        entity_id: Some(ep.id),
                    });
                }
            }
        }
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::enums::{
        DeliverableStatus, DeliverableType, DisseminationLevel, EntryStatus, IssueStatus,
        MilestoneStatus, ReportingPeriodStatus, WpStatus,
    };
    use crate::domain::execution_entities::{EquipmentProcurement, Person};
    use erc_core::domain::entities::{PersonnelRole, Project, ProjectConfig, RoleType};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    fn make_project(call_opening_date: Option<String>) -> Project {
        let config = ProjectConfig {
            project_title: "Test".to_string(),
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
            call_opening_date,
        };
        Project::new(config)
    }

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()
    }

    fn empty_actuals() -> ActualFinancialsDto {
        ActualFinancialsDto {
            a_actual: Decimal::ZERO,
            b_actual: Decimal::ZERO,
            c1_actual: Decimal::ZERO,
            c2_actual: Decimal::ZERO,
            c3_actual: Decimal::ZERO,
            e_actual: Decimal::ZERO,
            total_direct_actual: Decimal::ZERO,
            total_eligible_actual: Decimal::ZERO,
            requested_eu_contribution_actual: Decimal::ZERO,
            cfs_status_actual: CfsStatus::NotRequired,
            category_a_overrun: false,
            category_b_overrun: false,
            category_c1_overrun: false,
            category_c2_overrun: false,
            category_c3_overrun: false,
        }
    }

    fn base_ctx<'a>(
        project: &'a Project,
        exec: &'a ExecutionData,
        actuals: &'a ActualFinancialsDto,
    ) -> WarningContext<'a> {
        WarningContext {
            project,
            exec,
            actuals,
            deliverables: &[],
            milestones: &[],
            work_packages: &[],
            reporting_periods: &[],
            risks: &[],
            issues: &[],
            trip_executions: &[],
            current_project_month: 6,
            today: today(),
        }
    }

    #[test]
    fn test_no_warnings_for_empty_project() {
        let project = make_project(None);
        let exec = ExecutionData::default();
        let actuals = empty_actuals();
        let ctx = base_ctx(&project, &exec, &actuals);
        assert!(evaluate_warnings(&ctx).is_empty());
    }

    fn make_deliverable(is_overdue: bool) -> DeliverableDetailDto {
        DeliverableDetailDto {
            id: Uuid::new_v4(),
            deliverable_number: "D1.1".to_string(),
            title: "Protocol".to_string(),
            deliverable_type: DeliverableType::Report,
            work_package_id: 1,
            planned_month: 3,
            responsible_role_id: Uuid::new_v4(),
            responsible_role_label: "Role".to_string(),
            dissemination_level: DisseminationLevel::Public,
            status: DeliverableStatus::InProgress,
            actual_submission_date: None,
            revision_note: None,
            revised_planned_month: None,
            cordis_registered: false,
            notes: None,
            is_overdue,
            cordis_warning: false,
            reporting_period_number: None,
        }
    }

    #[test]
    fn test_w01_overdue_deliverable() {
        let project = make_project(None);
        let exec = ExecutionData::default();
        let actuals = empty_actuals();
        let deliverables = vec![make_deliverable(true)];
        let mut ctx = base_ctx(&project, &exec, &actuals);
        ctx.deliverables = &deliverables;
        let warnings = evaluate_warnings(&ctx);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W-01");
        assert_eq!(warnings[0].severity, WarningSeverity::Error);
    }

    #[test]
    fn test_w01_not_overdue_deliverable_no_warning() {
        let project = make_project(None);
        let exec = ExecutionData::default();
        let actuals = empty_actuals();
        let deliverables = vec![make_deliverable(false)];
        let mut ctx = base_ctx(&project, &exec, &actuals);
        ctx.deliverables = &deliverables;
        assert!(evaluate_warnings(&ctx).is_empty());
    }

    fn make_milestone(
        planned_month: u32,
        actual_completion_month: Option<u32>,
    ) -> MilestoneDetailDto {
        MilestoneDetailDto {
            id: Uuid::new_v4(),
            title: "Prototype ready".to_string(),
            work_package_id: 1,
            planned_month,
            status: MilestoneStatus::OnTrack,
            effective_status: MilestoneStatus::OnTrack,
            actual_completion_month,
            linked_deliverable_ids: vec![],
        }
    }

    #[test]
    fn test_w02_milestone_overdue_without_completion() {
        let project = make_project(None);
        let exec = ExecutionData::default();
        let actuals = empty_actuals();
        let milestones = vec![make_milestone(3, None)];
        let mut ctx = base_ctx(&project, &exec, &actuals);
        ctx.milestones = &milestones;
        let warnings = evaluate_warnings(&ctx);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W-02");
    }

    #[test]
    fn test_w02_milestone_completed_no_warning() {
        let project = make_project(None);
        let exec = ExecutionData::default();
        let actuals = empty_actuals();
        let milestones = vec![make_milestone(3, Some(4))];
        let mut ctx = base_ctx(&project, &exec, &actuals);
        ctx.milestones = &milestones;
        assert!(evaluate_warnings(&ctx).is_empty());
    }

    fn make_period(
        status: ReportingPeriodStatus,
        deadline: Option<&str>,
    ) -> ReportingPeriodDetailDto {
        ReportingPeriodDetailDto {
            id: Uuid::new_v4(),
            period_number: 1,
            start_month: 1,
            end_month: 18,
            submission_deadline: deadline.map(|s| s.to_string()),
            technical_report_submitted: false,
            financial_report_submitted: false,
            status,
            deliverables_due: 0,
            deliverables_submitted: 0,
        }
    }

    #[test]
    fn test_w03_deadline_within_60_days() {
        let project = make_project(None);
        let exec = ExecutionData::default();
        let actuals = empty_actuals();
        // today = 2026-06-01; deadline 30 days out.
        let periods = vec![make_period(ReportingPeriodStatus::Open, Some("2026-07-01"))];
        let mut ctx = base_ctx(&project, &exec, &actuals);
        ctx.reporting_periods = &periods;
        let warnings = evaluate_warnings(&ctx);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W-03");
    }

    #[test]
    fn test_w04_deadline_within_14_days() {
        let project = make_project(None);
        let exec = ExecutionData::default();
        let actuals = empty_actuals();
        let periods = vec![make_period(ReportingPeriodStatus::Open, Some("2026-06-10"))];
        let mut ctx = base_ctx(&project, &exec, &actuals);
        ctx.reporting_periods = &periods;
        let warnings = evaluate_warnings(&ctx);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W-04");
    }

    #[test]
    fn test_w04_deadline_already_passed() {
        let project = make_project(None);
        let exec = ExecutionData::default();
        let actuals = empty_actuals();
        let periods = vec![make_period(ReportingPeriodStatus::Open, Some("2026-05-01"))];
        let mut ctx = base_ctx(&project, &exec, &actuals);
        ctx.reporting_periods = &periods;
        let warnings = evaluate_warnings(&ctx);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W-04");
    }

    #[test]
    fn test_no_period_warning_when_submitted() {
        let project = make_project(None);
        let exec = ExecutionData::default();
        let actuals = empty_actuals();
        let periods = vec![make_period(
            ReportingPeriodStatus::Submitted,
            Some("2026-06-05"),
        )];
        let mut ctx = base_ctx(&project, &exec, &actuals);
        ctx.reporting_periods = &periods;
        assert!(evaluate_warnings(&ctx).is_empty());
    }

    #[test]
    fn test_w05_category_overrun() {
        let project = make_project(None);
        let exec = ExecutionData::default();
        let mut actuals = empty_actuals();
        actuals.category_a_overrun = true;
        let ctx = base_ctx(&project, &exec, &actuals);
        let warnings = evaluate_warnings(&ctx);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W-05");
    }

    #[test]
    fn test_w06_cfs_unaddressed() {
        let project = make_project(None);
        let exec = ExecutionData::default();
        let mut actuals = empty_actuals();
        actuals.cfs_status_actual = CfsStatus::RequiredAndUnaddressed;
        let ctx = base_ctx(&project, &exec, &actuals);
        let warnings = evaluate_warnings(&ctx);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W-06");
    }

    #[test]
    fn test_w06_cfs_dismissed_no_warning() {
        let project = make_project(None);
        let exec = ExecutionData::default();
        let mut actuals = empty_actuals();
        actuals.cfs_status_actual = CfsStatus::RequiredButDismissed;
        let ctx = base_ctx(&project, &exec, &actuals);
        assert!(evaluate_warnings(&ctx).is_empty());
    }

    fn make_wp(overspend: bool) -> WorkPackageExecutionDetailDto {
        WorkPackageExecutionDetailDto {
            work_package_id: 1,
            work_package_name: None,
            leader_role_id: None,
            leader_role_label: None,
            notes: None,
            status: WpStatus::OnTrack,
            planned_eur: dec!(1000),
            actual_eur: dec!(1200),
            overspend_warning: overspend,
        }
    }

    #[test]
    fn test_w07_wp_overspend() {
        let project = make_project(None);
        let exec = ExecutionData::default();
        let actuals = empty_actuals();
        let wps = vec![make_wp(true)];
        let mut ctx = base_ctx(&project, &exec, &actuals);
        ctx.work_packages = &wps;
        let warnings = evaluate_warnings(&ctx);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W-07");
    }

    fn make_risk(
        priority: Level,
        status: RiskStatus,
        review_date: Option<&str>,
    ) -> RiskEntryDetailDto {
        RiskEntryDetailDto {
            id: Uuid::new_v4(),
            title: "Key researcher departure".to_string(),
            description: "D".to_string(),
            work_package_id: None,
            probability: priority,
            impact: priority,
            mitigation: None,
            status,
            owner_role_id: None,
            owner_role_label: None,
            identified_date: "2026-01-01".to_string(),
            review_date: review_date.map(|s| s.to_string()),
            closed_date: None,
            risk_score: 9,
            priority,
        }
    }

    #[test]
    fn test_w08_high_risk_review_overdue() {
        let project = make_project(None);
        let exec = ExecutionData::default();
        let actuals = empty_actuals();
        let risks = vec![make_risk(Level::High, RiskStatus::Open, Some("2026-05-01"))];
        let mut ctx = base_ctx(&project, &exec, &actuals);
        ctx.risks = &risks;
        let warnings = evaluate_warnings(&ctx);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W-08");
    }

    #[test]
    fn test_w08_closed_risk_no_warning() {
        let project = make_project(None);
        let exec = ExecutionData::default();
        let actuals = empty_actuals();
        let risks = vec![make_risk(
            Level::High,
            RiskStatus::Closed,
            Some("2026-05-01"),
        )];
        let mut ctx = base_ctx(&project, &exec, &actuals);
        ctx.risks = &risks;
        assert!(evaluate_warnings(&ctx).is_empty());
    }

    #[test]
    fn test_w08_future_review_date_no_warning() {
        let project = make_project(None);
        let exec = ExecutionData::default();
        let actuals = empty_actuals();
        let risks = vec![make_risk(Level::High, RiskStatus::Open, Some("2026-06-15"))];
        let mut ctx = base_ctx(&project, &exec, &actuals);
        ctx.risks = &risks;
        assert!(evaluate_warnings(&ctx).is_empty());
    }

    fn make_issue(is_stale: bool) -> IssueEntryDetailDto {
        IssueEntryDetailDto {
            id: Uuid::new_v4(),
            description: "Delivery delay".to_string(),
            work_package_id: None,
            raised_date: "2026-01-01".to_string(),
            priority: Level::High,
            owner_role_id: None,
            owner_role_label: None,
            status: IssueStatus::Open,
            resolution: None,
            linked_risk_id: None,
            is_stale_warning: is_stale,
        }
    }

    #[test]
    fn test_w09_stale_issue() {
        let project = make_project(None);
        let exec = ExecutionData::default();
        let actuals = empty_actuals();
        let issues = vec![make_issue(true)];
        let mut ctx = base_ctx(&project, &exec, &actuals);
        ctx.issues = &issues;
        let warnings = evaluate_warnings(&ctx);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W-09");
    }

    #[test]
    fn test_w10_role_without_linked_person() {
        let mut project = make_project(None);
        let role_id = Uuid::new_v4();
        project.personnel_roles.push(PersonnelRole {
            id: role_id,
            role_label: "PostDoc-1".to_string(),
            role_type: RoleType::PostDoc,
            current_monthly_salary_try: dec!(50000),
            fte_fraction: dec!(1),
            inflation_rate_pct: dec!(0),
            start_month: 1,
            end_month: 12,
        });
        let exec = ExecutionData::default();
        let actuals = empty_actuals();
        let ctx = base_ctx(&project, &exec, &actuals);
        let warnings = evaluate_warnings(&ctx);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W-10");
    }

    #[test]
    fn test_w10_role_with_linked_person_no_warning() {
        let mut project = make_project(None);
        let role_id = Uuid::new_v4();
        project.personnel_roles.push(PersonnelRole {
            id: role_id,
            role_label: "PostDoc-1".to_string(),
            role_type: RoleType::PostDoc,
            current_monthly_salary_try: dec!(50000),
            fte_fraction: dec!(1),
            inflation_rate_pct: dec!(0),
            start_month: 1,
            end_month: 12,
        });
        let mut exec = ExecutionData::default();
        exec.persons.push(Person {
            id: Uuid::new_v4(),
            full_name: "Ada".to_string(),
            email: None,
            institution: None,
            orcid: None,
            linked_role_id: role_id,
            actual_start_date: "2026-01-01".to_string(),
            actual_end_date: None,
        });
        let actuals = empty_actuals();
        let ctx = base_ctx(&project, &exec, &actuals);
        assert!(evaluate_warnings(&ctx).is_empty());
    }

    fn make_trip_execution(overspend: bool) -> TripExecutionDetailDto {
        TripExecutionDetailDto {
            id: Uuid::new_v4(),
            trip_id: Uuid::new_v4(),
            trip_name: "Conference".to_string(),
            instance_number: 1,
            traveller_person_id: Uuid::new_v4(),
            traveller_name: "Ada".to_string(),
            actual_travel_date: "2026-03-01".to_string(),
            actual_cost_eur: dec!(700),
            status: EntryStatus::Approved,
            planned_cost_per_instance_eur: dec!(500),
            overspend_warning: overspend,
        }
    }

    #[test]
    fn test_w11_trip_overspend() {
        let project = make_project(None);
        let exec = ExecutionData::default();
        let actuals = empty_actuals();
        let trips = vec![make_trip_execution(true)];
        let mut ctx = base_ctx(&project, &exec, &actuals);
        ctx.trip_executions = &trips;
        let warnings = evaluate_warnings(&ctx);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W-11");
    }

    #[test]
    fn test_w12_equipment_purchase_after_project_end() {
        // duration_years=1, call_opening_date=2026-01-01 → project end = 2026-12-01.
        let project = make_project(Some("2026-01-01".to_string()));
        let mut exec = ExecutionData::default();
        exec.equipment_procurements.push(EquipmentProcurement {
            id: Uuid::new_v4(),
            equipment_item_id: Uuid::new_v4(),
            actual_purchase_cost_eur: dec!(2000),
            purchase_date: "2027-01-15".to_string(),
            delivery_confirmed: true,
        });
        let actuals = empty_actuals();
        let ctx = base_ctx(&project, &exec, &actuals);
        let warnings = evaluate_warnings(&ctx);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W-12");
    }

    #[test]
    fn test_w12_skipped_without_call_opening_date() {
        let project = make_project(None);
        let mut exec = ExecutionData::default();
        exec.equipment_procurements.push(EquipmentProcurement {
            id: Uuid::new_v4(),
            equipment_item_id: Uuid::new_v4(),
            actual_purchase_cost_eur: dec!(2000),
            purchase_date: "2099-01-15".to_string(),
            delivery_confirmed: true,
        });
        let actuals = empty_actuals();
        let ctx = base_ctx(&project, &exec, &actuals);
        assert!(evaluate_warnings(&ctx).is_empty());
    }

    #[test]
    fn test_w12_purchase_within_project_duration_no_warning() {
        let project = make_project(Some("2026-01-01".to_string()));
        let mut exec = ExecutionData::default();
        exec.equipment_procurements.push(EquipmentProcurement {
            id: Uuid::new_v4(),
            equipment_item_id: Uuid::new_v4(),
            actual_purchase_cost_eur: dec!(2000),
            purchase_date: "2026-06-01".to_string(),
            delivery_confirmed: true,
        });
        let actuals = empty_actuals();
        let ctx = base_ctx(&project, &exec, &actuals);
        assert!(evaluate_warnings(&ctx).is_empty());
    }
}
