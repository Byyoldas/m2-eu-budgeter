//! Integration tests for CALC-19 (full budget summary orchestration).
//!
//! These tests exercise the full calculation pipeline end-to-end by constructing
//! complete `Project` entities and passing them through `calculate_budget_summary`.
//!
//! Test data is drawn from the workbook sample project (IT-01 scenario):
//!   - 5-year ERC-CoG project (60 months)
//!   - PI: 227,900 TRY/month, FTE 0.70, months 1-60 (all 5 years), 20% inflation
//!   - PostDoc-1: 151,860 TRY/month, FTE 1.0, months 13-60 (years 2-5), 20% inflation
//!   - Expert-1: 138,000 TRY/month, FTE 0.40, months 1-12 (year 1), 20% inflation
//!   - Laptop: €2,500, 48m lifetime, 100%, 55m use → capped at €2,500
//!   - Audio recorder: €60, 60m lifetime, 100%, 36m → €36
//!   - India fieldwork: 4× per year, 5800km, 4 nights, 5 days, €340 domestic
//!   - Vienna conference: 3×, 2100km, 5 nights, 6 days, €0 domestic
//!   - Publications budget: 3 C3 items, €5000 each
//!   - TRY/EUR: 50.62, inflation: 20%, indirect: 25%, rate: v_from_2025_05_13

use erc_budget_lib::domain::entities::*;
use erc_budget_lib::domain::rate_data::{RateData, RateVersion, FlightBand, CountryRate};
use erc_budget_lib::calculation::budget_summary::calculate_budget_summary;
use erc_budget_lib::domain::dto::CfsStatus;
use rust_decimal_macros::dec;
use uuid::Uuid;

// ─── Rate data fixture ─────────────────────────────────────────────────────────

fn make_rate_data() -> RateData {
    let version = RateVersion {
        version_id: "v_from_2025_05_13".to_string(),
        version_label: "From 2025-05-13".to_string(),
        applicable_from: "2025-05-13".to_string(),
        valid_until: None,
        flight_bands: vec![
            FlightBand { band_id: "F-01".to_string(), label: "400-600 km".to_string(),    min_km: 400,   max_km: 600,    cost_eur: 340 },
            FlightBand { band_id: "F-02".to_string(), label: "601-1600 km".to_string(),   min_km: 601,   max_km: 1600,   cost_eur: 365 },
            FlightBand { band_id: "F-03".to_string(), label: "1601-2500 km".to_string(),  min_km: 1601,  max_km: 2500,   cost_eur: 429 },
            FlightBand { band_id: "F-04".to_string(), label: "2501-3500 km".to_string(),  min_km: 2501,  max_km: 3500,   cost_eur: 541 },
            FlightBand { band_id: "F-05".to_string(), label: "3501-4500 km".to_string(),  min_km: 3501,  max_km: 4500,   cost_eur: 743 },
            FlightBand { band_id: "F-06".to_string(), label: "4501-6000 km".to_string(),  min_km: 4501,  max_km: 6000,   cost_eur: 857 },
            FlightBand { band_id: "F-07".to_string(), label: "6001-7500 km".to_string(),  min_km: 6001,  max_km: 7500,   cost_eur: 1021 },
            FlightBand { band_id: "F-08".to_string(), label: "7501-10000 km".to_string(), min_km: 7501,  max_km: 10000,  cost_eur: 1250 },
            FlightBand { band_id: "F-09".to_string(), label: "10001+ km".to_string(),     min_km: 10001, max_km: 999999, cost_eur: 1595 },
        ],
        country_rates: vec![
            CountryRate { country_code: "IN".to_string(), country_name: "India".to_string(),    accommodation_eur: 195, subsistence_eur: 50 },
            CountryRate { country_code: "AT".to_string(), country_name: "Austria".to_string(),   accommodation_eur: 158, subsistence_eur: 131 },
            CountryRate { country_code: "FR".to_string(), country_name: "France".to_string(),    accommodation_eur: 212, subsistence_eur: 127 },
            CountryRate { country_code: "TR".to_string(), country_name: "Turkey".to_string(),    accommodation_eur: 165, subsistence_eur: 55 },
            CountryRate { country_code: "GB".to_string(), country_name: "UK".to_string(),        accommodation_eur: 209, subsistence_eur: 125 },
            CountryRate { country_code: "US".to_string(), country_name: "USA".to_string(),       accommodation_eur: 200, subsistence_eur: 80 },
            CountryRate { country_code: "OTHER".to_string(), country_name: "Other".to_string(),  accommodation_eur: 120, subsistence_eur: 40 },
        ],
    };
    RateData { versions: vec![version] }
}

// ─── Project fixture ───────────────────────────────────────────────────────────

fn make_sample_project() -> Project {
    let config = ProjectConfig {
        project_title: "Integration Test Project".to_string(),
        pi_name: "Prof. Test".to_string(),
        call_reference: "ERC-2025-CoG".to_string(),
        duration_years: 5,
        work_package_count: 3,
        work_package_names: vec![None, None, None],
        work_package_start_months: vec![1, 1, 1],
        work_package_end_months: vec![60, 60, 60],
        default_inflation_rate_pct: dec!(20),
        try_eur_rate: dec!(50.62),
        indirect_cost_rate_pct: dec!(25),
        rate_version_id: "v_from_2025_05_13".to_string(),
        call_opening_date: None,
    };

    // PI: 227,900 TRY/month, FTE 0.70, months 1-60 (all 5 years), 20% inflation
    let pi = PersonnelRole {
        id: Uuid::new_v4(),
        role_label: "PI".to_string(),
        role_type: RoleType::Pi,
        current_monthly_salary_try: dec!(227900),
        fte_fraction: dec!(0.70),
        inflation_rate_pct: dec!(20),
        start_month: 1,
        end_month: 60,
    };

    // PostDoc-1: 151,860 TRY/month, FTE 1.0, months 13-60 (years 2-5)
    let postdoc = PersonnelRole {
        id: Uuid::new_v4(),
        role_label: "PostDoc-1".to_string(),
        role_type: RoleType::PostDoc,
        current_monthly_salary_try: dec!(151860),
        fte_fraction: dec!(1.0),
        inflation_rate_pct: dec!(20),
        start_month: 13,
        end_month: 60,
    };

    // Expert-1: 138,000 TRY/month, FTE 0.40, months 1-12 (year 1 only)
    let expert = PersonnelRole {
        id: Uuid::new_v4(),
        role_label: "Expert-1".to_string(),
        role_type: RoleType::Expert,
        current_monthly_salary_try: dec!(138000),
        fte_fraction: dec!(0.40),
        inflation_rate_pct: dec!(20),
        start_month: 1,
        end_month: 12,
    };

    // Equipment: Laptop - should be capped
    let laptop = EquipmentItem {
        id: Uuid::new_v4(),
        name: "Laptop".to_string(),
        purchase_cost_eur: dec!(2500),
        useful_lifetime_months: 48,
        grant_usage_pct: dec!(100),
        grant_usage_months: 55,
        work_package_id: 1,
    };

    // Equipment: Audio recorder - NOT capped
    let recorder = EquipmentItem {
        id: Uuid::new_v4(),
        name: "Audio Recorder".to_string(),
        purchase_cost_eur: dec!(60),
        useful_lifetime_months: 60,
        grant_usage_pct: dec!(100),
        grant_usage_months: 36,
        work_package_id: 1,
    };

    // Travel: India fieldwork, 4 instances, 5800km
    // per instance: €857 + (4×€195=€780) + (5×€50=€250) + €340 = €2,227
    // total: €2,227 × 4 = €8,908
    let india_trip = Trip {
        id: Uuid::new_v4(),
        name: "India Fieldwork".to_string(),
        trip_type: TripType::Itemized {
            destination_country_code: "IN".to_string(),
            one_way_distance_km: 5800,
            number_of_nights: 4,
            number_of_days: 5,
            domestic_transport_per_instance_eur: dec!(340),
        },
        number_of_instances: 4,
        work_package_ids: vec![1],
    };

    // Travel: Vienna conference, 3 instances, 2100km
    // per instance: €429 + (5×€158=€790) + (6×€131=€786) = €2,005
    // total: €2,005 × 3 = €6,015
    let vienna_trip = Trip {
        id: Uuid::new_v4(),
        name: "Vienna Conference".to_string(),
        trip_type: TripType::Itemized {
            destination_country_code: "AT".to_string(),
            one_way_distance_km: 2100,
            number_of_nights: 5,
            number_of_days: 6,
            domestic_transport_per_instance_eur: dec!(0),
        },
        number_of_instances: 3,
        work_package_ids: vec![2],
    };

    // C3 items: publications
    let pub3 = OtherDirectCostItem {
        id: Uuid::new_v4(),
        name: "Publications Year 3".to_string(),
        amount_eur: dec!(5000),
        is_cfs_item: false,
        notes: None,
        work_package_ids: vec![1],
    };
    let pub4 = OtherDirectCostItem {
        id: Uuid::new_v4(),
        name: "Publications Year 4".to_string(),
        amount_eur: dec!(5000),
        is_cfs_item: false,
        notes: None,
        work_package_ids: vec![1],
    };
    let pub5 = OtherDirectCostItem {
        id: Uuid::new_v4(),
        name: "Publications Year 5".to_string(),
        amount_eur: dec!(5000),
        is_cfs_item: false,
        notes: None,
        work_package_ids: vec![1],
    };

    Project {
        id: Uuid::new_v4(),
        config,
        personnel_roles: vec![pi, postdoc, expert],
        equipment_items: vec![laptop, recorder],
        trips: vec![india_trip, vienna_trip],
        other_cost_items: vec![pub3, pub4, pub5],
        subcontracting: Subcontracting::default(),
        cfs_warning_dismissed: false,
    }
}

// ─── IT-01: Full project calculation ──────────────────────────────────────────

#[test]
fn test_it01_budget_summary_returns_ok() {
    let project = make_sample_project();
    let rate_data = make_rate_data();
    let result = calculate_budget_summary(&project, &rate_data);
    assert!(result.is_ok(), "Budget summary failed: {:?}", result.err());
}

#[test]
fn test_it01_wp_budgets_length_equals_wp_count() {
    let project = make_sample_project();
    let rate_data = make_rate_data();
    let summary = calculate_budget_summary(&project, &rate_data).unwrap();
    assert_eq!(summary.wp_budgets.len(), 3);
}

#[test]
fn test_it01_equipment_c2_total() {
    // Laptop capped at €2,500 + Audio recorder €36 = €2,536
    let project = make_sample_project();
    let rate_data = make_rate_data();
    let summary = calculate_budget_summary(&project, &rate_data).unwrap();
    assert_eq!(summary.category_c2_total.round_dp(2), dec!(2536.00));
}

#[test]
fn test_it01_equipment_detail_capped_flag() {
    let project = make_sample_project();
    let rate_data = make_rate_data();
    let summary = calculate_budget_summary(&project, &rate_data).unwrap();
    let laptop = summary.equipment_detail.iter().find(|e| e.name == "Laptop").unwrap();
    let recorder = summary.equipment_detail.iter().find(|e| e.name == "Audio Recorder").unwrap();
    assert!(laptop.is_capped, "Laptop should be capped");
    assert!(!recorder.is_capped, "Audio recorder should NOT be capped");
    assert_eq!(laptop.eligible_depreciation_eur, dec!(2500));
    assert_eq!(recorder.eligible_depreciation_eur, dec!(36));
}

#[test]
fn test_it01_travel_india_fieldwork() {
    // India: 4 trips, per instance €2,227 → €8,908
    let project = make_sample_project();
    let rate_data = make_rate_data();
    let summary = calculate_budget_summary(&project, &rate_data).unwrap();
    let india = summary.trip_detail.iter().find(|t| t.name == "India Fieldwork").unwrap();
    assert_eq!(india.work_package_ids, vec![1]);
    assert_eq!(india.number_of_instances, 4);
    assert_eq!(india.per_instance_total_eur.round_dp(2), dec!(2227.00));
    assert_eq!(india.total_trip_cost_eur.round_dp(2), dec!(8908.00));
}

#[test]
fn test_it01_travel_vienna_conference() {
    // Vienna: 3 trips, 2100km → F-03 €429, 5n×€158=€790, 6d×€131=€786, €0 domestic
    // per instance: €429 + €790 + €786 + €0 = €2,005
    // total: €2,005 × 3 = €6,015
    let project = make_sample_project();
    let rate_data = make_rate_data();
    let summary = calculate_budget_summary(&project, &rate_data).unwrap();
    let vienna = summary.trip_detail.iter().find(|t| t.name == "Vienna Conference").unwrap();
    assert_eq!(vienna.work_package_ids, vec![2]);
    assert_eq!(vienna.total_trip_cost_eur.round_dp(2), dec!(6015.00));
}

#[test]
fn test_it01_c1_total_is_india_plus_vienna() {
    let project = make_sample_project();
    let rate_data = make_rate_data();
    let summary = calculate_budget_summary(&project, &rate_data).unwrap();
    assert_eq!(summary.category_c1_total.round_dp(2), dec!(8908.00) + dec!(6015.00));
}

#[test]
fn test_it01_wp_budgets_travel_lands_in_correct_wp() {
    // India trip tagged WP1, Vienna trip tagged WP2 — verify per-WP travel split.
    let project = make_sample_project();
    let rate_data = make_rate_data();
    let summary = calculate_budget_summary(&project, &rate_data).unwrap();
    let wp1 = summary.wp_budgets.iter().find(|w| w.work_package_id == 1).unwrap();
    let wp2 = summary.wp_budgets.iter().find(|w| w.work_package_id == 2).unwrap();
    assert_eq!(wp1.travel_eur.round_dp(2), dec!(8908.00));
    assert_eq!(wp2.travel_eur.round_dp(2), dec!(6015.00));
}

#[test]
fn test_it01_c3_total_15000() {
    // 3 × €5,000 publications
    let project = make_sample_project();
    let rate_data = make_rate_data();
    let summary = calculate_budget_summary(&project, &rate_data).unwrap();
    assert_eq!(summary.category_c3_total, dec!(15000));
}

#[test]
fn test_it01_category_b_zero_no_subcontracting() {
    let project = make_sample_project();
    let rate_data = make_rate_data();
    let summary = calculate_budget_summary(&project, &rate_data).unwrap();
    assert_eq!(summary.category_b_total, dec!(0));
}

#[test]
fn test_it01_indirect_costs_25pct_of_a_c1_c2_c3() {
    // E = 25% × (A + C1 + C2 + C3). B is excluded.
    let project = make_sample_project();
    let rate_data = make_rate_data();
    let summary = calculate_budget_summary(&project, &rate_data).unwrap();
    let base = summary.category_a_total
        + summary.category_c1_total
        + summary.category_c2_total
        + summary.category_c3_total;
    let expected_e = (base * dec!(0.25)).round_dp(10);
    assert_eq!(summary.category_e_total.round_dp(10), expected_e);
}

#[test]
fn test_it01_total_direct_costs_equals_a_plus_b_c1_c2_c3() {
    let project = make_sample_project();
    let rate_data = make_rate_data();
    let summary = calculate_budget_summary(&project, &rate_data).unwrap();
    let expected = summary.category_a_total
        + summary.category_b_total
        + summary.category_c1_total
        + summary.category_c2_total
        + summary.category_c3_total;
    assert_eq!(summary.total_direct_costs, expected);
}

#[test]
fn test_it01_total_eligible_equals_direct_plus_indirect() {
    let project = make_sample_project();
    let rate_data = make_rate_data();
    let summary = calculate_budget_summary(&project, &rate_data).unwrap();
    assert_eq!(
        summary.total_eligible_costs,
        summary.total_direct_costs + summary.category_e_total
    );
}

#[test]
fn test_it01_eu_contribution_equals_total_eligible() {
    // 100% EU funding for ERC Actual Costs
    let project = make_sample_project();
    let rate_data = make_rate_data();
    let summary = calculate_budget_summary(&project, &rate_data).unwrap();
    assert_eq!(summary.requested_eu_contribution, summary.total_eligible_costs);
}

#[test]
fn test_it01_role_detail_count() {
    let project = make_sample_project();
    let rate_data = make_rate_data();
    let summary = calculate_budget_summary(&project, &rate_data).unwrap();
    assert_eq!(summary.role_detail.len(), 3);
}

#[test]
fn test_it01_expert_inactive_in_year2() {
    // Expert-1 is active only months 1-12 (year 1); verify year 2 cost = 0.
    let project = make_sample_project();
    let rate_data = make_rate_data();
    let summary = calculate_budget_summary(&project, &rate_data).unwrap();
    let expert = summary.role_detail.iter().find(|r| r.role_label == "Expert-1").unwrap();
    let year2 = expert.cost_lines.iter().find(|l| l.year == 2).unwrap();
    assert!(!year2.is_active);
    assert_eq!(year2.annual_cost_eur, dec!(0));
}

#[test]
fn test_it01_postdoc_inactive_year1() {
    let project = make_sample_project();
    let rate_data = make_rate_data();
    let summary = calculate_budget_summary(&project, &rate_data).unwrap();
    let postdoc = summary.role_detail.iter().find(|r| r.role_label == "PostDoc-1").unwrap();
    let year1 = postdoc.cost_lines.iter().find(|l| l.year == 1).unwrap();
    assert!(!year1.is_active);
    assert_eq!(year1.annual_cost_eur, dec!(0));
}

#[test]
fn test_it01_pi_wp_breakdown_lands_entirely_in_wp1() {
    // WPs all span months 1-60 in this fixture, so the PI's cost (also months 1-60)
    // should be split evenly across all 3 overlapping WPs rather than concentrated.
    let project = make_sample_project();
    let rate_data = make_rate_data();
    let summary = calculate_budget_summary(&project, &rate_data).unwrap();
    let pi = summary.role_detail.iter().find(|r| r.role_label == "PI").unwrap();
    let wp_total: rust_decimal::Decimal = pi.wp_breakdown.iter().map(|w| w.amount_eur).sum();
    assert_eq!(wp_total.round_dp(2), pi.total_cost_eur.round_dp(2));
}

#[test]
fn test_it01_cfs_not_required_below_threshold() {
    // With our sample data the total is well above 430k, so let's test
    // a minimal project below threshold
    let mut project = make_sample_project();
    // Zero out everything except config
    project.personnel_roles.clear();
    project.equipment_items.clear();
    project.trips.clear();
    project.other_cost_items.clear();
    let rate_data = make_rate_data();
    let summary = calculate_budget_summary(&project, &rate_data).unwrap();
    assert_eq!(summary.cfs_status, CfsStatus::NotRequired);
    assert!(!summary.cfs_threshold_exceeded);
}

#[test]
fn test_it01_cfs_required_unaddressed_when_over_threshold() {
    // Full project budget should be well above €430,000
    let project = make_sample_project();
    let rate_data = make_rate_data();
    let summary = calculate_budget_summary(&project, &rate_data).unwrap();
    // Check that total exceeds threshold; if so CFS must be unaddressed
    if summary.requested_eu_contribution > dec!(430000) {
        assert_eq!(summary.cfs_status, CfsStatus::RequiredAndUnaddressed);
        assert!(summary.cfs_threshold_exceeded);
        assert!(summary.cfs_prompt_required);
    }
}

#[test]
fn test_it01_adding_cfs_item_sets_required_and_present() {
    use erc_budget_lib::domain::entities::OtherDirectCostItem;
    let mut project = make_sample_project();
    project.other_cost_items.push(OtherDirectCostItem {
        id: Uuid::new_v4(),
        name: "Certificate on Financial Statements".to_string(),
        amount_eur: dec!(12000),
        is_cfs_item: true,
        notes: None,
        work_package_ids: vec![],
    });
    let rate_data = make_rate_data();
    let summary = calculate_budget_summary(&project, &rate_data).unwrap();
    if summary.cfs_threshold_exceeded {
        assert_eq!(summary.cfs_status, CfsStatus::RequiredAndPresent);
        assert!(!summary.cfs_warning_active);
    }
}

// ─── IT-02: Empty project ──────────────────────────────────────────────────────

#[test]
fn test_it02_empty_project_all_zeros() {
    let config = ProjectConfig {
        project_title: "Empty".to_string(),
        pi_name: "".to_string(),
        call_reference: "".to_string(),
        duration_years: 5,
        work_package_count: 1,
        work_package_names: vec![None],
        work_package_start_months: vec![1],
        work_package_end_months: vec![60],
        default_inflation_rate_pct: dec!(20),
        try_eur_rate: dec!(50.62),
        indirect_cost_rate_pct: dec!(25),
        rate_version_id: "v_from_2025_05_13".to_string(),
        call_opening_date: None,
    };
    let project = Project::new(config);
    let rate_data = make_rate_data();
    let summary = calculate_budget_summary(&project, &rate_data).unwrap();
    assert_eq!(summary.total_eligible_costs, dec!(0));
    assert_eq!(summary.requested_eu_contribution, dec!(0));
    assert_eq!(summary.category_a_total, dec!(0));
    assert_eq!(summary.category_c1_total, dec!(0));
    assert_eq!(summary.category_c2_total, dec!(0));
    assert_eq!(summary.category_c3_total, dec!(0));
    assert_eq!(summary.category_e_total, dec!(0));
    assert_eq!(summary.cfs_status, CfsStatus::NotRequired);
}

// ─── IT-03: 1-year project ────────────────────────────────────────────────────

#[test]
fn test_it03_one_year_project() {
    let config = ProjectConfig {
        project_title: "One Year".to_string(),
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
        rate_version_id: "v_from_2025_05_13".to_string(),
        call_opening_date: None,
    };
    let pi = PersonnelRole {
        id: Uuid::new_v4(),
        role_label: "PI".to_string(),
        role_type: RoleType::Pi,
        current_monthly_salary_try: dec!(50000), // €1,000/month
        fte_fraction: dec!(1.0),
        inflation_rate_pct: dec!(0),
        start_month: 1,
        end_month: 12,
    };
    let mut project = Project::new(config);
    project.personnel_roles.push(pi);
    let rate_data = make_rate_data();
    let summary = calculate_budget_summary(&project, &rate_data).unwrap();
    // Year 1 salary with 0% inflation: €1,000 × 1.00 = €1,000/month, × 12 × 1.0 = €12,000
    assert_eq!(summary.category_a_total, dec!(12000));
    // E = 25% × 12,000 = 3,000
    assert_eq!(summary.category_e_total, dec!(3000));
    assert_eq!(summary.total_eligible_costs, dec!(15000));
    assert_eq!(summary.requested_eu_contribution, dec!(15000));
}

// ─── IT-04: Flat-amount trip ──────────────────────────────────────────────────

#[test]
fn test_it04_flat_amount_trip() {
    let config = ProjectConfig {
        project_title: "Flat Trip Test".to_string(),
        pi_name: "PI".to_string(),
        call_reference: "ERC-2025-CoG".to_string(),
        duration_years: 3,
        work_package_count: 1,
        work_package_names: vec![None],
        work_package_start_months: vec![1],
        work_package_end_months: vec![36],
        default_inflation_rate_pct: dec!(0),
        try_eur_rate: dec!(50),
        indirect_cost_rate_pct: dec!(25),
        rate_version_id: "v_from_2025_05_13".to_string(),
        call_opening_date: None,
    };
    let flat_trip = Trip {
        id: Uuid::new_v4(),
        name: "Domestic Conference".to_string(),
        trip_type: TripType::FlatAmount { flat_amount_per_instance_eur: dec!(2000) },
        number_of_instances: 3,
        work_package_ids: vec![1],
    };
    let mut project = Project::new(config);
    project.trips.push(flat_trip);
    let rate_data = make_rate_data();
    let summary = calculate_budget_summary(&project, &rate_data).unwrap();
    assert_eq!(summary.category_c1_total, dec!(6000)); // 3 × €2,000
    let trip = &summary.trip_detail[0];
    assert_eq!(trip.per_instance_total_eur, dec!(2000));
    assert_eq!(trip.total_trip_cost_eur, dec!(6000));
    // Flat trips have no flight/accommodation breakdown
    assert!(trip.flight_cost_per_instance.is_none());
}

#[test]
fn test_it05_subcontracting_included_in_eligible_and_requested_excluded_from_indirect_base() {
    // Subcontracting (B) counts toward total_eligible_costs / requested_eu_contribution,
    // but must not enter the indirect cost (E) base.
    let config = ProjectConfig {
        project_title: "Subcontracting Test".to_string(),
        pi_name: "PI".to_string(),
        call_reference: "ERC-2025-CoG".to_string(),
        duration_years: 1,
        work_package_count: 1,
        work_package_start_months: vec![1],
        work_package_end_months: vec![12],
        work_package_names: vec![None],
        default_inflation_rate_pct: dec!(0),
        try_eur_rate: dec!(50),
        indirect_cost_rate_pct: dec!(25),
        rate_version_id: "v_from_2025_05_13".to_string(),
        call_opening_date: None,
    };
    let mut project = Project::new(config);
    project.subcontracting = Subcontracting { amount_eur: dec!(20000), work_package_id: 1 };
    project.other_cost_items.push(OtherDirectCostItem {
        id: Uuid::new_v4(),
        name: "Publications".to_string(),
        amount_eur: dec!(5000),
        is_cfs_item: false,
        notes: None,
        work_package_ids: vec![1],
    });

    let rate_data = make_rate_data();
    let summary = calculate_budget_summary(&project, &rate_data).unwrap();

    assert_eq!(summary.category_b_total, dec!(20000));
    assert_eq!(summary.category_c3_total, dec!(5000));
    // Indirect = 25% of (A + C1 + C2 + C3) = 25% of 5000 = 1250. B excluded from this base.
    assert_eq!(summary.category_e_total, dec!(1250));
    // Total direct = A + B + C1 + C2 + C3 = 0 + 20000 + 0 + 0 + 5000 = 25000.
    assert_eq!(summary.total_direct_costs, dec!(25000));
    // Eligible = total_direct + E = 25000 + 1250 = 26250. B is included here.
    assert_eq!(summary.total_eligible_costs, dec!(26250));
    assert_eq!(summary.requested_eu_contribution, dec!(26250));
    // Subcontracting is tagged to WP1 and should appear in its per-WP budget line.
    let wp1 = summary.wp_budgets.iter().find(|w| w.work_package_id == 1).unwrap();
    assert_eq!(wp1.subcontracting_eur, dec!(20000));
}
