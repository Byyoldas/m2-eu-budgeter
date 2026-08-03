//! Risk Register (M-12) and Issue Log (M-13) derivation.

use crate::domain::enums::{IssueStatus, Level};
use crate::domain::execution_entities::IssueEntry;
use chrono::NaiveDate;

fn level_value(level: Level) -> u8 {
    match level {
        Level::Low => 1,
        Level::Medium => 2,
        Level::High => 3,
    }
}

/// BR-RK-01: `probability_value × impact_value`, range 1–9.
pub fn risk_score(probability: Level, impact: Level) -> u8 {
    level_value(probability) * level_value(impact)
}

/// BR-RK-02: score ≥6 High, 3–5 Medium, 1–2 Low.
pub fn derive_risk_priority(score: u8) -> Level {
    if score >= 6 {
        Level::High
    } else if score >= 3 {
        Level::Medium
    } else {
        Level::Low
    }
}

/// BR-IS-02: a `High` priority issue still `Open` more than 14 days after
/// `raised_date` triggers a dashboard warning.
pub fn is_issue_stale_high_priority(issue: &IssueEntry, today: NaiveDate) -> bool {
    if issue.priority != Level::High || issue.status != IssueStatus::Open {
        return false;
    }
    let Ok(raised) = NaiveDate::parse_from_str(&issue.raised_date, "%Y-%m-%d") else {
        return false;
    };
    (today - raised).num_days() > 14
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::execution_entities::IssueEntry;
    use uuid::Uuid;

    #[test]
    fn test_risk_score_high_high_is_nine() {
        assert_eq!(risk_score(Level::High, Level::High), 9);
    }

    #[test]
    fn test_risk_score_low_low_is_one() {
        assert_eq!(risk_score(Level::Low, Level::Low), 1);
    }

    #[test]
    fn test_risk_score_medium_high_is_six() {
        assert_eq!(risk_score(Level::Medium, Level::High), 6);
    }

    #[test]
    fn test_derive_priority_high_at_six() {
        assert_eq!(derive_risk_priority(6), Level::High);
    }

    #[test]
    fn test_derive_priority_medium_at_three_to_five() {
        assert_eq!(derive_risk_priority(3), Level::Medium);
        assert_eq!(derive_risk_priority(5), Level::Medium);
    }

    #[test]
    fn test_derive_priority_low_at_one_to_two() {
        assert_eq!(derive_risk_priority(1), Level::Low);
        assert_eq!(derive_risk_priority(2), Level::Low);
    }

    fn make_issue(priority: Level, status: IssueStatus, raised_date: &str) -> IssueEntry {
        IssueEntry {
            id: Uuid::new_v4(),
            description: "Test issue".to_string(),
            work_package_id: None,
            raised_date: raised_date.to_string(),
            priority,
            owner_role_id: None,
            status,
            resolution: None,
            linked_risk_id: None,
        }
    }

    #[test]
    fn test_issue_stale_when_high_open_over_14_days() {
        let issue = make_issue(Level::High, IssueStatus::Open, "2026-01-01");
        let today = NaiveDate::from_ymd_opt(2026, 1, 20).unwrap();
        assert!(is_issue_stale_high_priority(&issue, today));
    }

    #[test]
    fn test_issue_not_stale_within_14_days() {
        let issue = make_issue(Level::High, IssueStatus::Open, "2026-01-01");
        let today = NaiveDate::from_ymd_opt(2026, 1, 10).unwrap();
        assert!(!is_issue_stale_high_priority(&issue, today));
    }

    #[test]
    fn test_issue_not_stale_when_not_high_priority() {
        let issue = make_issue(Level::Medium, IssueStatus::Open, "2026-01-01");
        let today = NaiveDate::from_ymd_opt(2026, 1, 20).unwrap();
        assert!(!is_issue_stale_high_priority(&issue, today));
    }

    #[test]
    fn test_issue_not_stale_when_closed() {
        let issue = make_issue(Level::High, IssueStatus::Closed, "2026-01-01");
        let today = NaiveDate::from_ymd_opt(2026, 1, 20).unwrap();
        assert!(!is_issue_stale_high_priority(&issue, today));
    }
}
