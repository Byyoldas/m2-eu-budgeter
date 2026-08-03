//! Progress Engine — Work Package status (M-04, BR-WP-04) and Milestone
//! status (M-06, BR-MS-01) derivation, plus the WP "actual cost" rollup this
//! sprint can support (personnel only — see `calculate_wp_actual_personnel_eur`).
//!
//! Sprint E2 scope notes:
//! - BR-WP-04's `Completed` case drops the "AND all deliverables Accepted"
//!   clause, and `AtRisk` only considers milestones — Deliverable Tracking
//!   (M-05) doesn't exist yet. Both tighten once it does.
//! - `current_project_month` auto-detects from `ProjectConfig.call_opening_date`
//!   (falling back to month 1 when unset) — a reasonable default for the
//!   architecture doc's own open question on this (§17, Q2).

use crate::domain::enums::{MilestoneStatus, WpStatus};
use crate::domain::execution_entities::{Milestone, Person, PersonMonthRecord};
use chrono::Datelike;
use erc_core::calculation::personnel_cost::allocate_personnel_cost_by_wp;
use erc_core::calculation::salary_projection::{convert_try_to_eur, project_salary_chain};
use erc_core::domain::entities::{PersonnelRole, Project};
use erc_core::error::AppError;
use rust_decimal::Decimal;
use std::collections::BTreeMap;

pub fn derive_current_project_month(project: &Project) -> u32 {
    derive_current_project_month_at(project, chrono::Utc::now().date_naive())
}

fn derive_current_project_month_at(project: &Project, today: chrono::NaiveDate) -> u32 {
    let max_month = project.config.duration_years as u32 * 12;
    let Some(call_opening_date) = &project.config.call_opening_date else {
        return 1;
    };
    let Ok(base) = chrono::NaiveDate::parse_from_str(call_opening_date, "%Y-%m-%d") else {
        return 1;
    };
    if today <= base {
        return 1;
    }
    let months_elapsed =
        (today.year() - base.year()) * 12 + (today.month() as i32 - base.month() as i32);
    (months_elapsed + 1).clamp(1, max_month as i32) as u32
}

/// BR-MS-01. Pure display-time overlay — never mutates the stored `status`.
pub fn derive_milestone_status(
    milestone: &Milestone,
    current_project_month: u32,
) -> MilestoneStatus {
    if milestone.status == MilestoneStatus::NotStarted
        && milestone.planned_month < current_project_month
    {
        MilestoneStatus::AtRisk
    } else {
        milestone.status
    }
}

/// BR-WP-04.
pub fn derive_wp_status(
    wp_start_month: u32,
    wp_end_month: u32,
    wp_milestones: &[&Milestone],
    current_project_month: u32,
) -> WpStatus {
    if current_project_month < wp_start_month {
        return WpStatus::NotStarted;
    }
    if current_project_month > wp_end_month {
        return WpStatus::Completed;
    }
    let at_risk = wp_milestones.iter().any(|m| {
        matches!(
            derive_milestone_status(m, current_project_month),
            MilestoneStatus::AtRisk | MilestoneStatus::Delayed
        )
    });
    if at_risk {
        WpStatus::AtRisk
    } else {
        WpStatus::OnTrack
    }
}

/// BR-PM-05's "planned person-months for that period" baseline, computed
/// directly from `PersonnelRole` (erc-core has no ready-made "planned
/// FTE-months for a given month" helper — this is new execution-specific
/// arithmetic over existing domain data, not a duplicate of any calculation).
pub fn calculate_planned_fte_months_for_month(roles: &[PersonnelRole], month: u32) -> Decimal {
    roles
        .iter()
        .filter(|r| month >= r.start_month && month <= r.end_month)
        .map(|r| r.fte_fraction)
        .sum()
}

/// BR-PM-07: `approved_months × inflation-adjusted monthly salary (EUR)` for
/// the project year `record.project_month` falls in. Reuses erc-core's
/// CALC-01/02 (currency conversion + compounding inflation) rather than
/// reimplementing them.
pub fn calculate_person_month_salary_estimate(
    record: &PersonMonthRecord,
    role: &PersonnelRole,
    project: &Project,
) -> Result<Option<Decimal>, AppError> {
    let Some(approved) = record.approved_months else {
        return Ok(None);
    };
    let base_eur =
        convert_try_to_eur(role.current_monthly_salary_try, project.config.try_eur_rate)?;
    let projections = project_salary_chain(
        base_eur,
        role.inflation_rate_pct,
        project.config.duration_years,
    )?;
    let year = ((record.project_month - 1) / 12 + 1) as u8;
    let monthly_eur = projections
        .iter()
        .find(|p| p.year == year)
        .map(|p| p.projected_monthly_eur)
        .unwrap_or(Decimal::ZERO);
    Ok(Some(monthly_eur * approved))
}

/// BR-WP-02, Sprint E2 scope: actual cost per WP from allocated personnel
/// actuals only (approved `PersonMonthRecord`s), reusing erc-core's
/// `allocate_personnel_cost_by_wp` one calendar month at a time. Expands to
/// include C1/C2/C3/B actuals once those tracking modules exist (Sprint E3).
pub fn calculate_wp_actual_personnel_eur(
    project: &Project,
    records: &[PersonMonthRecord],
    persons: &[Person],
) -> Result<BTreeMap<u8, Decimal>, AppError> {
    let work_packages: Vec<(u8, u32, u32)> = project
        .config
        .work_packages()
        .into_iter()
        .map(|wp| (wp.id, wp.start_month, wp.end_month))
        .collect();

    let mut totals: BTreeMap<u8, Decimal> = BTreeMap::new();

    for record in records {
        let Some(approved) = record.approved_months else {
            continue;
        };
        let Some(person) = persons.iter().find(|p| p.id == record.person_id) else {
            continue;
        };
        let Some(role) = project
            .personnel_roles
            .iter()
            .find(|r| r.id == person.linked_role_id)
        else {
            continue;
        };

        let base_eur =
            convert_try_to_eur(role.current_monthly_salary_try, project.config.try_eur_rate)?;
        let projections = project_salary_chain(
            base_eur,
            role.inflation_rate_pct,
            project.config.duration_years,
        )?;

        let amounts = allocate_personnel_cost_by_wp(
            &projections,
            approved,
            record.project_month,
            record.project_month,
            &work_packages,
        )?;
        for amount in amounts {
            *totals
                .entry(amount.work_package_id)
                .or_insert(Decimal::ZERO) += amount.amount_eur;
        }
    }

    Ok(totals)
}

#[cfg(test)]
mod tests {
    use super::*;
    use erc_core::domain::entities::{Project, ProjectConfig, RoleType};
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    fn make_project(call_opening_date: Option<String>) -> Project {
        let config = ProjectConfig {
            project_title: "Test".to_string(),
            pi_name: "PI".to_string(),
            call_reference: "ERC-2025-CoG".to_string(),
            duration_years: 3,
            work_package_count: 2,
            work_package_names: vec![None, None],
            work_package_start_months: vec![1, 13],
            work_package_end_months: vec![18, 36],
            default_inflation_rate_pct: dec!(0),
            try_eur_rate: dec!(50),
            indirect_cost_rate_pct: dec!(25),
            rate_version_id: "from_2025_05_13".to_string(),
            call_opening_date,
        };
        Project::new(config)
    }

    fn make_role(start_month: u32, end_month: u32, fte: Decimal) -> PersonnelRole {
        PersonnelRole {
            id: Uuid::new_v4(),
            role_label: "PostDoc-1".to_string(),
            role_type: RoleType::PostDoc,
            current_monthly_salary_try: dec!(50000),
            fte_fraction: fte,
            inflation_rate_pct: dec!(0),
            start_month,
            end_month,
        }
    }

    // ─── derive_current_project_month ──────────────────────────────────

    #[test]
    fn test_current_month_defaults_to_1_when_call_opening_date_unset() {
        let project = make_project(None);
        assert_eq!(derive_current_project_month(&project), 1);
    }

    #[test]
    fn test_current_month_is_1_on_call_opening_date_itself() {
        let project = make_project(Some("2026-01-01".to_string()));
        let today = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        assert_eq!(derive_current_project_month_at(&project, today), 1);
    }

    #[test]
    fn test_current_month_advances_with_elapsed_calendar_months() {
        let project = make_project(Some("2026-01-01".to_string()));
        let today = chrono::NaiveDate::from_ymd_opt(2026, 4, 15).unwrap();
        assert_eq!(derive_current_project_month_at(&project, today), 4);
    }

    #[test]
    fn test_current_month_clamps_to_project_duration() {
        let project = make_project(Some("2020-01-01".to_string()));
        let today = chrono::NaiveDate::from_ymd_opt(2030, 1, 1).unwrap();
        // duration_years = 3 → max month 36.
        assert_eq!(derive_current_project_month_at(&project, today), 36);
    }

    // ─── derive_milestone_status ────────────────────────────────────────

    fn make_milestone(status: MilestoneStatus, planned_month: u32) -> Milestone {
        Milestone {
            id: Uuid::new_v4(),
            title: "M".to_string(),
            work_package_id: 1,
            planned_month,
            status,
            actual_completion_month: None,
        }
    }

    #[test]
    fn test_ms_not_started_overdue_becomes_at_risk() {
        let m = make_milestone(MilestoneStatus::NotStarted, 3);
        assert_eq!(derive_milestone_status(&m, 6), MilestoneStatus::AtRisk);
    }

    #[test]
    fn test_ms_not_started_not_yet_due_stays_not_started() {
        let m = make_milestone(MilestoneStatus::NotStarted, 6);
        assert_eq!(derive_milestone_status(&m, 3), MilestoneStatus::NotStarted);
    }

    #[test]
    fn test_ms_completed_stays_completed_even_if_overdue() {
        let m = make_milestone(MilestoneStatus::Completed, 3);
        assert_eq!(derive_milestone_status(&m, 6), MilestoneStatus::Completed);
    }

    // ─── derive_wp_status ───────────────────────────────────────────────

    #[test]
    fn test_wp_status_not_started() {
        assert_eq!(derive_wp_status(6, 18, &[], 3), WpStatus::NotStarted);
    }

    #[test]
    fn test_wp_status_completed_after_end_month() {
        assert_eq!(derive_wp_status(1, 18, &[], 24), WpStatus::Completed);
    }

    #[test]
    fn test_wp_status_on_track_with_no_at_risk_milestones() {
        let m = make_milestone(MilestoneStatus::OnTrack, 10);
        assert_eq!(derive_wp_status(1, 18, &[&m], 6), WpStatus::OnTrack);
    }

    #[test]
    fn test_wp_status_at_risk_from_overdue_milestone() {
        let m = make_milestone(MilestoneStatus::NotStarted, 3);
        assert_eq!(derive_wp_status(1, 18, &[&m], 6), WpStatus::AtRisk);
    }

    #[test]
    fn test_wp_status_at_risk_from_delayed_milestone() {
        let m = make_milestone(MilestoneStatus::Delayed, 10);
        assert_eq!(derive_wp_status(1, 18, &[&m], 6), WpStatus::AtRisk);
    }

    // ─── calculate_planned_fte_months_for_month ────────────────────────

    #[test]
    fn test_planned_fte_months_sums_active_roles_only() {
        let roles = vec![make_role(1, 12, dec!(1.0)), make_role(6, 24, dec!(0.5))];
        // Month 3: only the first role is active.
        assert_eq!(calculate_planned_fte_months_for_month(&roles, 3), dec!(1.0));
        // Month 8: both are active.
        assert_eq!(calculate_planned_fte_months_for_month(&roles, 8), dec!(1.5));
        // Month 20: only the second role is active.
        assert_eq!(
            calculate_planned_fte_months_for_month(&roles, 20),
            dec!(0.5)
        );
    }

    // ─── calculate_person_month_salary_estimate ────────────────────────

    #[test]
    fn test_salary_estimate_none_when_not_approved() {
        let project = make_project(None);
        let role = make_role(1, 12, dec!(1.0));
        let record = PersonMonthRecord {
            id: Uuid::new_v4(),
            person_id: Uuid::new_v4(),
            project_month: 1,
            reported_months: dec!(1.0),
            approved_months: None,
        };
        assert_eq!(
            calculate_person_month_salary_estimate(&record, &role, &project).unwrap(),
            None
        );
    }

    #[test]
    fn test_salary_estimate_computed_when_approved() {
        let project = make_project(None);
        let role = make_role(1, 12, dec!(1.0));
        let record = PersonMonthRecord {
            id: Uuid::new_v4(),
            person_id: Uuid::new_v4(),
            project_month: 1,
            reported_months: dec!(1.0),
            approved_months: Some(dec!(1.0)),
        };
        // 50000 TRY / 50 = 1000 EUR base; year 1, 0% inflation → 1000 EUR × 1.0.
        let estimate = calculate_person_month_salary_estimate(&record, &role, &project)
            .unwrap()
            .unwrap();
        assert_eq!(estimate, dec!(1000));
    }

    // ─── calculate_wp_actual_personnel_eur ──────────────────────────────

    #[test]
    fn test_wp_actual_personnel_eur_allocates_to_active_wp() {
        let project = make_project(None);
        let role = make_role(1, 12, dec!(1.0));
        let role_id = role.id;
        let mut project = project;
        project.personnel_roles.push(role);

        let person = Person {
            id: Uuid::new_v4(),
            full_name: "Ada".to_string(),
            email: None,
            institution: None,
            orcid: None,
            linked_role_id: role_id,
            actual_start_date: "2026-01-01".to_string(),
            actual_end_date: None,
        };
        let persons = vec![person.clone()];

        let record = PersonMonthRecord {
            id: Uuid::new_v4(),
            person_id: person.id,
            project_month: 3, // within WP1's [1,18] range only
            reported_months: dec!(1.0),
            approved_months: Some(dec!(1.0)),
        };

        let totals =
            calculate_wp_actual_personnel_eur(&project, std::slice::from_ref(&record), &persons)
                .unwrap();
        assert_eq!(totals.get(&1), Some(&dec!(1000)));
        assert_eq!(totals.get(&2), None);
    }

    #[test]
    fn test_wp_actual_personnel_eur_ignores_unapproved_records() {
        let mut project = make_project(None);
        let role = make_role(1, 12, dec!(1.0));
        let role_id = role.id;
        project.personnel_roles.push(role);

        let person = Person {
            id: Uuid::new_v4(),
            full_name: "Ada".to_string(),
            email: None,
            institution: None,
            orcid: None,
            linked_role_id: role_id,
            actual_start_date: "2026-01-01".to_string(),
            actual_end_date: None,
        };
        let persons = vec![person.clone()];

        let record = PersonMonthRecord {
            id: Uuid::new_v4(),
            person_id: person.id,
            project_month: 3,
            reported_months: dec!(1.0),
            approved_months: None,
        };

        let totals =
            calculate_wp_actual_personnel_eur(&project, std::slice::from_ref(&record), &persons)
                .unwrap();
        assert!(totals.is_empty());
    }
}
