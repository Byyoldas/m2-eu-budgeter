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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
