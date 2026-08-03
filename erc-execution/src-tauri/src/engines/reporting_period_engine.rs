//! Reporting Period Management engine (M-14).

use crate::domain::dto::ReportingPeriodCoverageDto;
use crate::domain::enums::ReportingPeriodStatus;
use crate::domain::execution_entities::ReportingPeriod;
use uuid::Uuid;

const DEFAULT_PERIOD_LENGTH_MONTHS: u32 = 18;

/// BR-RP-05: default periods pre-populated on project open, when
/// `reporting_periods` is empty. Periods are `DEFAULT_PERIOD_LENGTH_MONTHS`
/// (18) long, except the final one, which absorbs whatever remains of the
/// project duration — this generalises the spec's literal ERC CoG example
/// (P1 M1–18, P2 M19–36, P3 M37–60 for a 60-month project) to any duration:
/// plugging in 60 here reproduces that exact example.
pub fn generate_default_reporting_periods(duration_months: u32) -> Vec<ReportingPeriod> {
    let mut periods = Vec::new();
    let mut start = 1u32;
    let mut period_number = 1u32;

    while start <= duration_months {
        let tentative_end = start + DEFAULT_PERIOD_LENGTH_MONTHS - 1;
        // If another full period wouldn't fit after this one, this period
        // absorbs the rest of the project instead of leaving a tiny trailing
        // remainder period.
        let end = if tentative_end + DEFAULT_PERIOD_LENGTH_MONTHS > duration_months {
            duration_months
        } else {
            tentative_end
        };
        periods.push(ReportingPeriod {
            id: Uuid::new_v4(),
            period_number,
            start_month: start,
            end_month: end,
            submission_deadline: None,
            technical_report_submitted: false,
            financial_report_submitted: false,
            status: ReportingPeriodStatus::Open,
        });
        start = end + 1;
        period_number += 1;
    }

    periods
}

/// BR-RP-01/02 advisory coverage check — see
/// `validation::validate_reporting_period`'s doc comment for why this isn't
/// hard-enforced per edit.
pub fn compute_coverage(periods: &[ReportingPeriod], max_month: u32) -> ReportingPeriodCoverageDto {
    let mut sorted: Vec<&ReportingPeriod> = periods.iter().collect();
    sorted.sort_by_key(|p| p.start_month);

    let mut gaps_detected = sorted.is_empty();
    let mut expected_start = 1;
    for p in &sorted {
        if p.start_month != expected_start {
            gaps_detected = true;
            break;
        }
        expected_start = p.end_month + 1;
    }

    let final_period_covers_project_end = sorted.last().is_some_and(|p| p.end_month == max_month);

    ReportingPeriodCoverageDto {
        gaps_detected,
        final_period_covers_project_end,
    }
}

/// BR-DEL-05: which reporting period (by number) a given effective planned
/// month falls into, if any.
pub fn find_period_for_month(periods: &[ReportingPeriod], month: u32) -> Option<u32> {
    periods
        .iter()
        .find(|p| month >= p.start_month && month <= p.end_month)
        .map(|p| p.period_number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_periods_matches_erc_cog_example_for_60_months() {
        let periods = generate_default_reporting_periods(60);
        let ranges: Vec<(u32, u32)> = periods
            .iter()
            .map(|p| (p.start_month, p.end_month))
            .collect();
        assert_eq!(ranges, vec![(1, 18), (19, 36), (37, 60)]);
    }

    #[test]
    fn test_default_periods_for_36_months() {
        let periods = generate_default_reporting_periods(36);
        let ranges: Vec<(u32, u32)> = periods
            .iter()
            .map(|p| (p.start_month, p.end_month))
            .collect();
        assert_eq!(ranges, vec![(1, 18), (19, 36)]);
    }

    #[test]
    fn test_default_periods_absorbs_short_remainder_into_single_period() {
        // 24 months: an 18-month first chunk would leave only 6 months, too
        // short for its own period, so it's absorbed into one period.
        let periods = generate_default_reporting_periods(24);
        let ranges: Vec<(u32, u32)> = periods
            .iter()
            .map(|p| (p.start_month, p.end_month))
            .collect();
        assert_eq!(ranges, vec![(1, 24)]);
    }

    #[test]
    fn test_default_periods_short_project_gets_one_period() {
        let periods = generate_default_reporting_periods(12);
        let ranges: Vec<(u32, u32)> = periods
            .iter()
            .map(|p| (p.start_month, p.end_month))
            .collect();
        assert_eq!(ranges, vec![(1, 12)]);
    }

    fn make_period(number: u32, start: u32, end: u32) -> ReportingPeriod {
        ReportingPeriod {
            id: Uuid::new_v4(),
            period_number: number,
            start_month: start,
            end_month: end,
            submission_deadline: None,
            technical_report_submitted: false,
            financial_report_submitted: false,
            status: ReportingPeriodStatus::Open,
        }
    }

    #[test]
    fn test_coverage_no_gaps_full_coverage() {
        let periods = vec![make_period(1, 1, 18), make_period(2, 19, 36)];
        let coverage = compute_coverage(&periods, 36);
        assert!(!coverage.gaps_detected);
        assert!(coverage.final_period_covers_project_end);
    }

    #[test]
    fn test_coverage_detects_gap() {
        let periods = vec![make_period(1, 1, 18), make_period(2, 20, 36)];
        let coverage = compute_coverage(&periods, 36);
        assert!(coverage.gaps_detected);
    }

    #[test]
    fn test_coverage_detects_final_period_short_of_project_end() {
        let periods = vec![make_period(1, 1, 18), make_period(2, 19, 30)];
        let coverage = compute_coverage(&periods, 36);
        assert!(!coverage.gaps_detected);
        assert!(!coverage.final_period_covers_project_end);
    }

    #[test]
    fn test_coverage_empty_periods_detects_gap() {
        let coverage = compute_coverage(&[], 36);
        assert!(coverage.gaps_detected);
        assert!(!coverage.final_period_covers_project_end);
    }

    #[test]
    fn test_find_period_for_month() {
        let periods = vec![make_period(1, 1, 18), make_period(2, 19, 36)];
        assert_eq!(find_period_for_month(&periods, 5), Some(1));
        assert_eq!(find_period_for_month(&periods, 25), Some(2));
        assert_eq!(find_period_for_month(&periods, 99), None);
    }
}
