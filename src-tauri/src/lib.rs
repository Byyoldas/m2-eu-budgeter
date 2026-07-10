//! ERC Budget — Tauri application root.
//!
//! Wires together all modules, initialises the application state,
//! registers IPC commands, and launches the Tauri window.

mod commands;
pub mod domain;
pub mod calculation;
mod validation;
mod persistence;
mod error;

use std::sync::Mutex;
use domain::rate_data::RateData;
use domain::entities::Project;

/// Shared mutable application state injected into every Tauri command.
pub struct AppState {
    /// The currently open project. None until the user creates or loads one.
    pub project: Mutex<Option<Project>>,
    /// File-system path of the currently open .ercbudget file.
    pub project_path: Mutex<Option<std::path::PathBuf>>,
    /// EU travel rate tables loaded at startup. Read-only for the lifetime of the app.
    pub rate_data: RateData,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let rate_data = RateData::load_embedded()
        .expect("Failed to load embedded EU travel rate data. The application bundle may be corrupt.");

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            project: Mutex::new(None),
            project_path: Mutex::new(None),
            rate_data,
        })
        .invoke_handler(tauri::generate_handler![
            // Project lifecycle
            commands::project::create_project,
            commands::project::update_project_config,
            commands::project::load_project,
            commands::project::save_project,
            commands::project::save_project_as,
            commands::project::get_project,
            commands::project::get_rate_versions,
            commands::project::get_countries,
            // Personnel
            commands::personnel::add_personnel_role,
            commands::personnel::update_personnel_role,
            commands::personnel::delete_personnel_role,
            commands::personnel::preview_role_cost,
            // Equipment
            commands::equipment::add_equipment_item,
            commands::equipment::update_equipment_item,
            commands::equipment::delete_equipment_item,
            commands::equipment::preview_equipment_depreciation,
            // Travel
            commands::travel::add_trip,
            commands::travel::update_trip,
            commands::travel::delete_trip,
            commands::travel::preview_trip_cost,
            // Other costs & CFS
            commands::other_costs::add_other_cost,
            commands::other_costs::update_other_cost,
            commands::other_costs::delete_other_cost,
            commands::other_costs::add_cfs_item,
            commands::other_costs::remove_cfs_item,
            commands::other_costs::dismiss_cfs_warning,
            commands::other_costs::set_subcontracting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
