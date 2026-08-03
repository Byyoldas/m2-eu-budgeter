//! ERC Execution — Tauri application root.
//!
//! Wires together all modules, initialises the application state,
//! registers IPC commands, and launches the Tauri window.

pub mod commands;
pub mod domain;
mod engines;
mod error;
pub mod persistence;
mod validation;

use domain::execution_entities::ExecutionData;
use erc_core::domain::entities::Project;
use erc_core::domain::rate_data::RateData;
use std::sync::Mutex;

/// Shared mutable application state injected into every Tauri command.
pub struct AppState {
    /// The open project's budget data (from `.ercbudget`). Source of truth
    /// for all planned values; never mutated by this app.
    pub project: Mutex<Option<Project>>,
    /// The open project's execution data. `None` until a file is opened.
    pub execution_data: Mutex<Option<ExecutionData>>,
    /// File-system path of the currently open `.ercbudget` file.
    pub project_path: Mutex<Option<std::path::PathBuf>>,
    /// EU travel rate tables loaded at startup. Read-only for the lifetime of the app.
    pub rate_data: RateData,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let rate_data = RateData::load_embedded().expect(
        "Failed to load embedded EU travel rate data. The application bundle may be corrupt.",
    );

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(AppState {
            project: Mutex::new(None),
            execution_data: Mutex::new(None),
            project_path: Mutex::new(None),
            rate_data,
        })
        .invoke_handler(tauri::generate_handler![
            // Project lifecycle
            commands::project::open_execution_project,
            commands::project::save_execution_project,
            // M-03: Personnel & Person-Month Tracking
            commands::persons::add_person,
            commands::persons::update_person,
            commands::persons::delete_person,
            commands::person_months::add_person_month_record,
            commands::person_months::update_person_month_record,
            commands::person_months::delete_person_month_record,
            // M-04: Work Package Management
            commands::work_packages::set_work_package_execution,
            // M-06: Milestone Tracking
            commands::milestones::add_milestone,
            commands::milestones::update_milestone,
            commands::milestones::complete_milestone,
            commands::milestones::delete_milestone,
            // Amendment Management (from-scratch design)
            commands::amendments::record_amendment,
            commands::amendments::update_amendment,
            commands::amendments::delete_amendment,
            // M-08: Travel Tracking
            commands::travel::add_trip_execution,
            commands::travel::update_trip_execution,
            commands::travel::delete_trip_execution,
            // M-09: Equipment Tracking
            commands::equipment::add_equipment_procurement,
            commands::equipment::update_equipment_procurement,
            commands::equipment::delete_equipment_procurement,
            // M-10: Other Costs Tracking
            commands::other_costs::add_actual_cost_entry,
            commands::other_costs::update_actual_cost_entry,
            commands::other_costs::delete_actual_cost_entry,
            // M-11: Subcontracting Tracking
            commands::subcontracting::add_subcontracting_line,
            commands::subcontracting::update_subcontracting_line,
            commands::subcontracting::delete_subcontracting_line,
            // M-05: Deliverable Tracking
            commands::deliverables::add_deliverable,
            commands::deliverables::update_deliverable,
            commands::deliverables::delete_deliverable,
            // M-14: Reporting Period Management
            commands::reporting_periods::add_reporting_period,
            commands::reporting_periods::update_reporting_period,
            commands::reporting_periods::delete_reporting_period,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
