//! Regression safety net for the Milestone 0 → Milestone 1 `erc-core`
//! extraction (see docs/executer/shared-core-roadmap.md §8).
//!
//! These tests load real `.ercbudget` fixtures from the workspace-level
//! `test-fixtures/` directory and pin known-good calculation outputs. If any
//! extraction step changes behaviour, these tests fail before the change
//! reaches the Budget Application's users.

use erc_budget_lib::calculation::budget_summary::calculate_budget_summary;
use erc_budget_lib::domain::dto::CfsStatus;
use erc_budget_lib::domain::rate_data::RateData;
use erc_budget_lib::persistence::{load_project, save_project};
use rust_decimal_macros::dec;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-fixtures")).join(name)
}

fn rate_data() -> RateData {
    RateData::load_embedded().expect("embedded EU travel rate tables must parse")
}

// ─── Load correctness ──────────────────────────────────────────────────────────

#[test]
fn test_existing_v1_0_file_loads_correctly_simple() {
    let project = load_project(&fixture_path("simple_project_v1_0.ercbudget"))
        .expect("simple_project_v1_0.ercbudget must load");

    assert_eq!(project.config.duration_years, 1);
    assert_eq!(project.config.work_package_count, 1);
    assert_eq!(project.personnel_roles.len(), 1);
    assert_eq!(project.personnel_roles[0].role_label, "PI");

    let summary = calculate_budget_summary(&project, &rate_data())
        .expect("budget summary must calculate for the simple fixture");
    assert_eq!(summary.category_a_total, dec!(12000));
    assert_eq!(summary.category_e_total, dec!(3000));
    assert_eq!(summary.total_eligible_costs, dec!(15000));
    assert_eq!(summary.requested_eu_contribution, dec!(15000));
    assert_eq!(summary.cfs_status, CfsStatus::NotRequired);
}

#[test]
fn test_existing_v1_0_file_loads_correctly_full() {
    let project = load_project(&fixture_path("full_project_v1_0.ercbudget"))
        .expect("full_project_v1_0.ercbudget must load");

    assert_eq!(project.config.duration_years, 5);
    assert_eq!(project.config.work_package_count, 3);
    assert_eq!(project.personnel_roles.len(), 3);
    assert_eq!(project.equipment_items.len(), 2);
    assert_eq!(project.trips.len(), 2);
    assert_eq!(project.other_cost_items.len(), 3);

    let summary = calculate_budget_summary(&project, &rate_data())
        .expect("budget summary must calculate for the full fixture");
    assert_eq!(summary.wp_budgets.len(), 3);
    assert_eq!(summary.role_detail.len(), 3);
}

// ─── Round-trip fidelity ───────────────────────────────────────────────────────

#[test]
fn test_file_save_and_reload_roundtrip() {
    for name in [
        "simple_project_v1_0.ercbudget",
        "full_project_v1_0.ercbudget",
    ] {
        let original = load_project(&fixture_path(name)).expect("fixture must load");

        let tmp = std::env::temp_dir().join(format!(
            "erc-budget-regression-roundtrip-{name}-{}.ercbudget",
            std::process::id()
        ));
        save_project(&original, &tmp).expect("save_project must succeed");
        let reloaded = load_project(&tmp).expect("reload must succeed");
        std::fs::remove_file(&tmp).ok();

        // Compare via the calculated summary rather than deriving PartialEq on
        // every entity: any field lost or corrupted in the round trip will
        // change at least one downstream calculated value.
        let rates = rate_data();
        let original_summary = calculate_budget_summary(&original, &rates).unwrap();
        let reloaded_summary = calculate_budget_summary(&reloaded, &rates).unwrap();
        assert_eq!(
            original_summary.requested_eu_contribution, reloaded_summary.requested_eu_contribution,
            "round-trip changed the calculated EU contribution for {name}"
        );
        assert_eq!(original.id, reloaded.id);
        assert_eq!(
            original.personnel_roles.len(),
            reloaded.personnel_roles.len()
        );
        assert_eq!(
            original.equipment_items.len(),
            reloaded.equipment_items.len()
        );
        assert_eq!(original.trips.len(), reloaded.trips.len());
        assert_eq!(
            original.other_cost_items.len(),
            reloaded.other_cost_items.len()
        );
    }
}

// ─── Calculation reference values (regression baseline) ───────────────────────
//
// These values must not change unless a deliberate, approved specification
// change is made. They were established by running `full_project_v1_0.ercbudget`
// through `calculate_budget_summary` against the real embedded EU rate tables.

#[test]
fn test_calculation_reference_values() {
    let project = load_project(&fixture_path("full_project_v1_0.ercbudget")).unwrap();
    let summary = calculate_budget_summary(&project, &rate_data()).unwrap();

    // Category C2 — Laptop capped at €2,500 + Audio Recorder €36
    assert_eq!(summary.category_c2_total.round_dp(2), dec!(2536.00));
    // Category C1 — India Fieldwork €8,908 + Vienna Conference €6,015
    assert_eq!(summary.category_c1_total.round_dp(2), dec!(14923.00));
    // Category C3 — three €5,000 publication items
    assert_eq!(summary.category_c3_total, dec!(15000));
    // Category B — no subcontracting in this fixture
    assert_eq!(summary.category_b_total, dec!(0));
    // Category A — PI + PostDoc-1 + Expert-1 salary projection (TRY→EUR, compounding inflation)
    assert_eq!(summary.category_a_total.round_dp(2), dec!(631693.98));

    let base = summary.category_a_total
        + summary.category_c1_total
        + summary.category_c2_total
        + summary.category_c3_total;
    let expected_e = (base * dec!(0.25)).round_dp(10);
    assert_eq!(summary.category_e_total.round_dp(10), expected_e);

    let expected_direct = summary.category_a_total
        + summary.category_b_total
        + summary.category_c1_total
        + summary.category_c2_total
        + summary.category_c3_total;
    assert_eq!(summary.total_direct_costs, expected_direct);
    assert_eq!(
        summary.total_eligible_costs,
        summary.total_direct_costs + summary.category_e_total
    );
    assert_eq!(
        summary.requested_eu_contribution,
        summary.total_eligible_costs
    );

    // This fixture's requested EU contribution is well above the €430,000
    // CFS threshold and no CFS item has been added.
    assert_eq!(summary.cfs_status, CfsStatus::RequiredAndUnaddressed);
    assert!(summary.cfs_threshold_exceeded);
}
