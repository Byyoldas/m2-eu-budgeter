//! IPC commands for travel/trip management.

use tauri::State;
use uuid::Uuid;
use crate::AppState;
use crate::domain::entities::Trip;
use crate::domain::dto::{TripInputDto, BudgetSummaryDto, TripCostPreviewDto};
use crate::validation::validate_trip;
use crate::calculation::calculate_budget_summary;
use crate::calculation::trip_cost::{
    calculate_itemized_trip_cost, calculate_flat_trip_cost, TripCostResult,
};
use crate::domain::entities::TripType;
use crate::persistence::auto_save;
use crate::error::AppError;

/// Add a new trip to the project.
#[tauri::command]
pub fn add_trip(
    state: State<'_, AppState>,
    input: TripInputDto,
) -> Result<BudgetSummaryDto, AppError> {
    let mut lock = state.project.lock().unwrap();
    let project = lock.as_mut().ok_or(AppError::NoProject)?;

    validate_trip(&input, project.config.duration_years)?;

    let trip = Trip {
        id: Uuid::new_v4(),
        name: input.name,
        trip_type: input.trip_type,
        project_year: input.project_year,
        number_of_instances: input.number_of_instances,
        work_package_id: input.work_package_id,
    };
    project.trips.push(trip);

    let summary = calculate_budget_summary(project, &state.rate_data)?;

    let path_lock = state.project_path.lock().unwrap();
    let _ = auto_save(project, path_lock.as_deref());

    Ok(summary)
}

/// Update an existing trip by UUID.
#[tauri::command]
pub fn update_trip(
    state: State<'_, AppState>,
    id: Uuid,
    input: TripInputDto,
) -> Result<BudgetSummaryDto, AppError> {
    let mut lock = state.project.lock().unwrap();
    let project = lock.as_mut().ok_or(AppError::NoProject)?;

    validate_trip(&input, project.config.duration_years)?;

    let trip = project
        .trips
        .iter_mut()
        .find(|t| t.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Trip {id} not found.")))?;

    trip.name = input.name;
    trip.trip_type = input.trip_type;
    trip.project_year = input.project_year;
    trip.number_of_instances = input.number_of_instances;
    trip.work_package_id = input.work_package_id;

    let summary = calculate_budget_summary(project, &state.rate_data)?;

    let path_lock = state.project_path.lock().unwrap();
    let _ = auto_save(project, path_lock.as_deref());

    Ok(summary)
}

/// Delete a trip by UUID.
#[tauri::command]
pub fn delete_trip(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<BudgetSummaryDto, AppError> {
    let mut lock = state.project.lock().unwrap();
    let project = lock.as_mut().ok_or(AppError::NoProject)?;

    let before = project.trips.len();
    project.trips.retain(|t| t.id != id);
    if project.trips.len() == before {
        return Err(AppError::NotFound(format!("Trip {id} not found.")));
    }

    let summary = calculate_budget_summary(project, &state.rate_data)?;

    let path_lock = state.project_path.lock().unwrap();
    let _ = auto_save(project, path_lock.as_deref());

    Ok(summary)
}

/// Live cost preview for a trip being edited — does NOT mutate state.
/// Returns itemized breakdown for display in the travel form's live preview box.
#[tauri::command]
pub fn preview_trip_cost(
    state: State<'_, AppState>,
    input: TripInputDto,
) -> Result<TripCostPreviewDto, AppError> {
    let lock = state.project.lock().unwrap();
    let project = lock.as_ref().ok_or(AppError::NoProject)?;

    let rate_version = state.rate_data
        .find_version(&project.config.rate_version_id)
        .ok_or_else(|| AppError::NotFound(
            format!("Rate version '{}' not found.", project.config.rate_version_id)
        ))?;

    match &input.trip_type {
        TripType::Itemized {
            destination_country_code,
            one_way_distance_km,
            number_of_nights,
            number_of_days,
            domestic_transport_per_instance_eur,
        } => {
            let result = calculate_itemized_trip_cost(
                destination_country_code,
                *one_way_distance_km,
                *number_of_nights,
                *number_of_days,
                *domestic_transport_per_instance_eur,
                input.number_of_instances,
                rate_version,
            )?;
            Ok(TripCostPreviewDto {
                per_instance_total_eur: result.per_instance_total_eur,
                total_trip_cost_eur: result.total_trip_cost_eur,
                flight_cost_per_instance: Some(result.flight_cost_per_instance.to_string()),
                accommodation_cost_per_instance: Some(result.accommodation_cost_per_instance.to_string()),
                subsistence_cost_per_instance: Some(result.subsistence_cost_per_instance.to_string()),
                domestic_transport_per_instance: Some(result.domestic_transport_per_instance.to_string()),
                flight_band_label: if result.no_flight_applicable {
                    None
                } else {
                    Some(result.band_label.clone())
                },
                no_flight_applicable: result.no_flight_applicable,
                accommodation_rate_eur: Some(result.accommodation_rate_eur_per_night.to_string()),
                subsistence_rate_eur: Some(result.subsistence_rate_eur_per_day.to_string()),
            })
        }
        TripType::FlatAmount { flat_amount_per_instance_eur } => {
            let result = calculate_flat_trip_cost(
                *flat_amount_per_instance_eur,
                input.number_of_instances,
            )?;
            Ok(TripCostPreviewDto {
                per_instance_total_eur: result.flat_amount_per_instance,
                total_trip_cost_eur: result.total_trip_cost_eur,
                flight_cost_per_instance: None,
                accommodation_cost_per_instance: None,
                subsistence_cost_per_instance: None,
                domestic_transport_per_instance: None,
                flight_band_label: None,
                no_flight_applicable: false,
                accommodation_rate_eur: None,
                subsistence_rate_eur: None,
            })
        }
    }
}
