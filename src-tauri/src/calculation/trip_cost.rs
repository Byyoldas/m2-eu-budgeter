//! CALC-07 — Flight Cost Lookup
//! CALC-08 — Accommodation Cost per Trip Instance
//! CALC-09 — Subsistence Cost per Trip Instance
//! CALC-10 — Itemized Trip Total Cost
//! CALC-11 — Flat Amount Trip Total Cost
//! CALC-12 — Annual Travel Budget (Category C1)
//!
//! All travel costs are computed from EU official unit rates embedded in the application.
//! Only domestic transport is entered as a flat amount by the user.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::domain::rate_data::RateVersion;
use crate::error::{AppError, calc_error};

// ─── Output Types ─────────────────────────────────────────────────────────────

/// Result of the flight cost lookup (CALC-07).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightCostResult {
    #[serde(with = "rust_decimal::serde::str")]
    pub flight_cost_eur: Decimal,
    /// E.g. "4,501–6,000 km". Empty string when no flight applicable.
    pub band_label: String,
    /// True when one_way_distance_km < 400.
    pub no_flight_applicable: bool,
}

/// Full trip cost breakdown for one itemized trip (CALC-10 output).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemizedTripCost {
    #[serde(with = "rust_decimal::serde::str")]
    pub flight_cost_per_instance: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub accommodation_cost_per_instance: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub subsistence_cost_per_instance: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub domestic_transport_per_instance: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub per_instance_total_eur: Decimal,
    pub number_of_instances: u32,
    #[serde(with = "rust_decimal::serde::str")]
    pub total_trip_cost_eur: Decimal,
    // Rate info for display
    pub band_label: String,
    pub no_flight_applicable: bool,
    #[serde(with = "rust_decimal::serde::str")]
    pub accommodation_rate_eur_per_night: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub subsistence_rate_eur_per_day: Decimal,
}

/// Result for a flat-amount trip (CALC-11 output).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlatTripCost {
    #[serde(with = "rust_decimal::serde::str")]
    pub flat_amount_per_instance: Decimal,
    pub number_of_instances: u32,
    #[serde(with = "rust_decimal::serde::str")]
    pub total_trip_cost_eur: Decimal,
}

/// Unified trip cost result (wraps either variant).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TripCostResult {
    Itemized(ItemizedTripCost),
    FlatAmount(FlatTripCost),
}

impl TripCostResult {
    pub fn total_cost(&self) -> Decimal {
        match self {
            TripCostResult::Itemized(r) => r.total_trip_cost_eur,
            TripCostResult::FlatAmount(r) => r.total_trip_cost_eur,
        }
    }
}

/// One year's aggregated travel cost (CALC-12 intermediate).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearTravelCost {
    pub year: u8,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount_eur: Decimal,
}

/// Category C1 total and per-year breakdown (CALC-12 output).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TravelCategoryTotals {
    pub by_year: Vec<YearTravelCost>,
    #[serde(with = "rust_decimal::serde::str")]
    pub total: Decimal,
}

// ─── CALC-07 ─────────────────────────────────────────────────────────────────

/// CALC-07: Determine the EU flight unit cost for a given one-way distance.
///
/// Returns zero and `no_flight_applicable = true` when distance < 400 km.
/// The rate table is looked up from the embedded `RateVersion` data.
pub fn lookup_flight_cost(
    one_way_distance_km: u32,
    rate_version: &RateVersion,
) -> Result<FlightCostResult, AppError> {
    if one_way_distance_km < 400 {
        return Ok(FlightCostResult {
            flight_cost_eur: Decimal::ZERO,
            band_label: String::new(),
            no_flight_applicable: true,
        });
    }

    let band = rate_version
        .find_flight_band(one_way_distance_km)
        .ok_or_else(|| calc_error(
            "BAND_NOT_FOUND",
            format!("No flight band found for distance {one_way_distance_km} km. This is an internal error — please report it."),
        ))?;

    Ok(FlightCostResult {
        flight_cost_eur: band.cost_decimal(),
        band_label: band.label.clone(),
        no_flight_applicable: false,
    })
}

// ─── CALC-08 ─────────────────────────────────────────────────────────────────

/// CALC-08: Compute accommodation cost for one trip instance.
pub fn calculate_accommodation_cost(
    country_code: &str,
    number_of_nights: u32,
    rate_version: &RateVersion,
) -> Result<(Decimal, Decimal), AppError> { // (cost, nightly_rate)
    if number_of_nights < 1 {
        return Err(calc_error(
            "INVALID_NIGHTS",
            "Number of nights must be at least 1.",
        ));
    }

    let country = rate_version
        .find_country_rates(country_code)
        .ok_or_else(|| calc_error(
            "COUNTRY_NOT_IN_RATE_TABLE",
            format!("Country '{country_code}' is not in the EU travel rate table. Please enter the accommodation cost manually."),
        ))?;

    let nightly_rate = country.accommodation_decimal();
    let cost = nightly_rate * Decimal::from(number_of_nights);
    Ok((cost, nightly_rate))
}

// ─── CALC-09 ─────────────────────────────────────────────────────────────────

/// CALC-09: Compute subsistence cost for one trip instance.
pub fn calculate_subsistence_cost(
    country_code: &str,
    number_of_days: u32,
    rate_version: &RateVersion,
) -> Result<(Decimal, Decimal), AppError> { // (cost, daily_rate)
    if number_of_days < 1 {
        return Err(calc_error(
            "INVALID_DAYS",
            "Number of days must be at least 1.",
        ));
    }

    let country = rate_version
        .find_country_rates(country_code)
        .ok_or_else(|| calc_error(
            "COUNTRY_NOT_IN_RATE_TABLE",
            format!("Country '{country_code}' is not in the EU travel rate table. Please enter the subsistence cost manually."),
        ))?;

    let daily_rate = country.subsistence_decimal();
    let cost = daily_rate * Decimal::from(number_of_days);
    Ok((cost, daily_rate))
}

// ─── CALC-10 ─────────────────────────────────────────────────────────────────

/// CALC-10: Compute the total cost for one itemized trip.
///
/// Components: flight + accommodation + subsistence + domestic transport.
/// Domestic transport is the user-entered flat amount per instance.
pub fn calculate_itemized_trip_cost(
    country_code: &str,
    one_way_distance_km: u32,
    number_of_nights: u32,
    number_of_days: u32,
    domestic_transport_per_instance: Decimal,
    number_of_instances: u32,
    rate_version: &RateVersion,
) -> Result<ItemizedTripCost, AppError> {
    if domestic_transport_per_instance < Decimal::ZERO {
        return Err(calc_error(
            "INVALID_DOMESTIC_TRANSPORT",
            "Domestic transport cost cannot be negative.",
        ));
    }
    if number_of_instances < 1 {
        return Err(calc_error(
            "INVALID_INSTANCES",
            "Number of trip instances must be at least 1.",
        ));
    }

    let flight = lookup_flight_cost(one_way_distance_km, rate_version)?;
    let (accommodation_cost, accommodation_rate) =
        calculate_accommodation_cost(country_code, number_of_nights, rate_version)?;
    let (subsistence_cost, subsistence_rate) =
        calculate_subsistence_cost(country_code, number_of_days, rate_version)?;

    let per_instance = flight.flight_cost_eur
        + accommodation_cost
        + subsistence_cost
        + domestic_transport_per_instance;

    let total = per_instance * Decimal::from(number_of_instances);

    // Post-condition: total = per_instance * instances
    let expected = per_instance * Decimal::from(number_of_instances);
    if total != expected {
        return Err(calc_error(
            "INTERNAL_CALC_ERROR",
            "Trip total does not equal per-instance × instances. This is a bug.",
        ));
    }

    Ok(ItemizedTripCost {
        flight_cost_per_instance: flight.flight_cost_eur,
        accommodation_cost_per_instance: accommodation_cost,
        subsistence_cost_per_instance: subsistence_cost,
        domestic_transport_per_instance,
        per_instance_total_eur: per_instance,
        number_of_instances,
        total_trip_cost_eur: total,
        band_label: flight.band_label,
        no_flight_applicable: flight.no_flight_applicable,
        accommodation_rate_eur_per_night: accommodation_rate,
        subsistence_rate_eur_per_day: subsistence_rate,
    })
}

// ─── CALC-11 ─────────────────────────────────────────────────────────────────

/// CALC-11: Compute the total cost for a flat-amount trip.
pub fn calculate_flat_trip_cost(
    flat_amount_per_instance: Decimal,
    number_of_instances: u32,
) -> Result<FlatTripCost, AppError> {
    if flat_amount_per_instance <= Decimal::ZERO {
        return Err(calc_error(
            "INVALID_FLAT_AMOUNT",
            "Flat amount per trip instance must be greater than zero.",
        ));
    }
    if number_of_instances < 1 {
        return Err(calc_error(
            "INVALID_INSTANCES",
            "Number of trip instances must be at least 1.",
        ));
    }

    let total = flat_amount_per_instance * Decimal::from(number_of_instances);

    Ok(FlatTripCost {
        flat_amount_per_instance,
        number_of_instances,
        total_trip_cost_eur: total,
    })
}

// ─── CALC-12 ─────────────────────────────────────────────────────────────────

/// CALC-12: Aggregate all trip costs by assigned project year.
///
/// # Arguments
/// * `trip_costs` — Vec of (project_year, total_cost) pairs.
/// * `duration_years` — Total project duration (initialises zero entries).
pub fn aggregate_travel_by_year(
    trip_costs: &[(u8, Decimal)], // (year, total_cost)
    duration_years: u8,
) -> Result<TravelCategoryTotals, AppError> {
    let mut year_totals: Vec<Decimal> = vec![Decimal::ZERO; duration_years as usize];

    for &(year, cost) in trip_costs {
        if year < 1 || year > duration_years {
            return Err(calc_error(
                "YEAR_OUT_OF_RANGE",
                format!("Trip is assigned to year {year}, which is outside the project duration of {duration_years} years."),
            ));
        }
        year_totals[(year - 1) as usize] += cost;
    }

    let total: Decimal = year_totals.iter().sum();

    let by_year: Vec<YearTravelCost> = year_totals
        .into_iter()
        .enumerate()
        .map(|(i, amt)| YearTravelCost { year: (i + 1) as u8, amount_eur: amt })
        .collect();

    Ok(TravelCategoryTotals { by_year, total })
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use crate::domain::rate_data::{RateVersion, FlightBand, CountryRate};

    fn make_test_rate_version() -> RateVersion {
        RateVersion {
            version_id: "test".to_string(),
            version_label: "Test".to_string(),
            applicable_from: "2025-05-13".to_string(),
            valid_until: None,
            flight_bands: vec![
                FlightBand { band_id: "F-00".to_string(), label: "Up to 399 km".to_string(),  min_km: 0,     max_km: 399,    cost_eur: 0 },
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
                CountryRate { country_code: "IN".to_string(), country_name: "India".to_string(),          accommodation_eur: 195, subsistence_eur: 50 },
                CountryRate { country_code: "FR".to_string(), country_name: "France".to_string(),         accommodation_eur: 212, subsistence_eur: 127 },
                CountryRate { country_code: "AT".to_string(), country_name: "Austria".to_string(),        accommodation_eur: 158, subsistence_eur: 131 },
                CountryRate { country_code: "AU".to_string(), country_name: "Australia".to_string(),      accommodation_eur: 135, subsistence_eur: 75 },
                CountryRate { country_code: "GB".to_string(), country_name: "United Kingdom".to_string(), accommodation_eur: 209, subsistence_eur: 125 },
                CountryRate { country_code: "TR".to_string(), country_name: "Turkey".to_string(),         accommodation_eur: 165, subsistence_eur: 55 },
                CountryRate { country_code: "US".to_string(), country_name: "United States".to_string(),  accommodation_eur: 200, subsistence_eur: 80 },
            ],
        }
    }

    // ── CALC-07 tests ──

    #[test]
    fn test_calc_07_istanbul_to_london_2500km() {
        let rv = make_test_rate_version();
        let result = lookup_flight_cost(2500, &rv).unwrap();
        assert_eq!(result.flight_cost_eur, dec!(429));
        assert!(!result.no_flight_applicable);
    }

    #[test]
    fn test_calc_07_istanbul_to_mumbai_5800km() {
        let rv = make_test_rate_version();
        let result = lookup_flight_cost(5800, &rv).unwrap();
        assert_eq!(result.flight_cost_eur, dec!(857));
    }

    #[test]
    fn test_calc_07_melbourne_13800km_over_10000() {
        let rv = make_test_rate_version();
        let result = lookup_flight_cost(13800, &rv).unwrap();
        assert_eq!(result.flight_cost_eur, dec!(1595));
    }

    #[test]
    fn test_calc_07_istanbul_to_ankara_350km_no_flight() {
        let rv = make_test_rate_version();
        let result = lookup_flight_cost(350, &rv).unwrap();
        assert_eq!(result.flight_cost_eur, Decimal::ZERO);
        assert!(result.no_flight_applicable);
    }

    #[test]
    fn test_calc_07_zero_distance_no_flight() {
        let rv = make_test_rate_version();
        let result = lookup_flight_cost(0, &rv).unwrap();
        assert!(result.no_flight_applicable);
        assert_eq!(result.flight_cost_eur, Decimal::ZERO);
    }

    #[test]
    fn test_calc_07_boundary_exactly_600km_lower_band() {
        // Exactly 600 km → band F-01 (400–600) → €340
        let rv = make_test_rate_version();
        let result = lookup_flight_cost(600, &rv).unwrap();
        assert_eq!(result.flight_cost_eur, dec!(340));
    }

    #[test]
    fn test_calc_07_boundary_601km_higher_band() {
        // 601 km → band F-02 (601–1,600) → €365
        let rv = make_test_rate_version();
        let result = lookup_flight_cost(601, &rv).unwrap();
        assert_eq!(result.flight_cost_eur, dec!(365));
    }

    #[test]
    fn test_calc_07_exactly_400km_minimum_flight() {
        let rv = make_test_rate_version();
        let result = lookup_flight_cost(400, &rv).unwrap();
        assert_eq!(result.flight_cost_eur, dec!(340));
        assert!(!result.no_flight_applicable);
    }

    #[test]
    fn test_calc_07_vienna_1500km() {
        let rv = make_test_rate_version();
        let result = lookup_flight_cost(1500, &rv).unwrap();
        assert_eq!(result.flight_cost_eur, dec!(365));
    }

    // ── CALC-08 tests ──

    #[test]
    fn test_calc_08_india_4_nights() {
        let rv = make_test_rate_version();
        let (cost, rate) = calculate_accommodation_cost("IN", 4, &rv).unwrap();
        assert_eq!(rate, dec!(195));
        assert_eq!(cost, dec!(780));
    }

    #[test]
    fn test_calc_08_france_5_nights() {
        let rv = make_test_rate_version();
        let (cost, rate) = calculate_accommodation_cost("FR", 5, &rv).unwrap();
        assert_eq!(rate, dec!(212));
        assert_eq!(cost, dec!(1060));
    }

    #[test]
    fn test_calc_08_austria_rate_is_158_not_170() {
        // Verifies correction of workbook error E-02
        let rv = make_test_rate_version();
        let (_, rate) = calculate_accommodation_cost("AT", 1, &rv).unwrap();
        assert_eq!(rate, dec!(158)); // NOT 170 (the workbook's incorrect rate)
    }

    #[test]
    fn test_calc_08_zero_nights_returns_error() {
        let rv = make_test_rate_version();
        let result = calculate_accommodation_cost("IN", 0, &rv);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_NIGHTS"));
    }

    #[test]
    fn test_calc_08_unknown_country_returns_error() {
        let rv = make_test_rate_version();
        let result = calculate_accommodation_cost("XX", 3, &rv);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "COUNTRY_NOT_IN_RATE_TABLE"));
    }

    // ── CALC-09 tests ──

    #[test]
    fn test_calc_09_india_5_days() {
        let rv = make_test_rate_version();
        let (cost, rate) = calculate_subsistence_cost("IN", 5, &rv).unwrap();
        assert_eq!(rate, dec!(50));
        assert_eq!(cost, dec!(250));
    }

    #[test]
    fn test_calc_09_france_6_days() {
        let rv = make_test_rate_version();
        let (cost, _) = calculate_subsistence_cost("FR", 6, &rv).unwrap();
        assert_eq!(cost, dec!(762));
    }

    #[test]
    fn test_calc_09_austria_6_days() {
        let rv = make_test_rate_version();
        let (cost, rate) = calculate_subsistence_cost("AT", 6, &rv).unwrap();
        assert_eq!(rate, dec!(131));
        assert_eq!(cost, dec!(786));
    }

    // ── CALC-10 tests ──

    #[test]
    fn test_calc_10_india_fieldwork_4_instances() {
        // flight €857 + accommodation €780 (4×€195) + subsistence €250 (5×€50) + domestic €340
        // per instance = 2227; total = 2227 × 4 = 8908
        let rv = make_test_rate_version();
        let result = calculate_itemized_trip_cost("IN", 5800, 4, 5, dec!(340), 4, &rv).unwrap();
        assert_eq!(result.flight_cost_per_instance, dec!(857));
        assert_eq!(result.accommodation_cost_per_instance, dec!(780));
        assert_eq!(result.subsistence_cost_per_instance, dec!(250));
        assert_eq!(result.domestic_transport_per_instance, dec!(340));
        assert_eq!(result.per_instance_total_eur, dec!(2227));
        assert_eq!(result.total_trip_cost_eur, dec!(8908));
    }

    #[test]
    fn test_calc_10_france_conference_3_instances() {
        // flight €429 (2100km) + accommodation €1060 (5×€212) + subsistence €762 (6×€127) + domestic €0
        // per instance = 2251; total = 2251 × 3 = 6753
        let rv = make_test_rate_version();
        let result = calculate_itemized_trip_cost("FR", 2100, 5, 6, Decimal::ZERO, 3, &rv).unwrap();
        assert_eq!(result.per_instance_total_eur, dec!(2251));
        assert_eq!(result.total_trip_cost_eur, dec!(6753));
        assert!(!result.no_flight_applicable);
    }

    #[test]
    fn test_calc_10_no_flight_distance_zero() {
        let rv = make_test_rate_version();
        let result = calculate_itemized_trip_cost("TR", 0, 2, 3, Decimal::ZERO, 1, &rv).unwrap();
        assert_eq!(result.flight_cost_per_instance, Decimal::ZERO);
        assert!(result.no_flight_applicable);
    }

    #[test]
    fn test_calc_10_negative_domestic_transport_returns_error() {
        let rv = make_test_rate_version();
        let result = calculate_itemized_trip_cost("IN", 5800, 4, 5, dec!(-100), 2, &rv);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_DOMESTIC_TRANSPORT"));
    }

    #[test]
    fn test_calc_10_zero_instances_returns_error() {
        let rv = make_test_rate_version();
        let result = calculate_itemized_trip_cost("IN", 5800, 4, 5, Decimal::ZERO, 0, &rv);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_INSTANCES"));
    }

    // ── CALC-11 tests ──

    #[test]
    fn test_calc_11_flat_2000_times_3() {
        let result = calculate_flat_trip_cost(dec!(2000), 3).unwrap();
        assert_eq!(result.total_trip_cost_eur, dec!(6000));
    }

    #[test]
    fn test_calc_11_single_instance() {
        let result = calculate_flat_trip_cost(dec!(1500), 1).unwrap();
        assert_eq!(result.total_trip_cost_eur, dec!(1500));
    }

    #[test]
    fn test_calc_11_zero_flat_amount_returns_error() {
        let result = calculate_flat_trip_cost(Decimal::ZERO, 3);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_FLAT_AMOUNT"));
    }

    #[test]
    fn test_calc_11_zero_instances_returns_error() {
        let result = calculate_flat_trip_cost(dec!(1000), 0);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "INVALID_INSTANCES"));
    }

    // ── CALC-12 tests ──

    #[test]
    fn test_calc_12_trips_spread_across_years() {
        let trip_costs = vec![(1, dec!(8908)), (2, dec!(6753)), (3, dec!(6000))];
        let totals = aggregate_travel_by_year(&trip_costs, 5).unwrap();
        assert_eq!(totals.by_year[0].amount_eur, dec!(8908));
        assert_eq!(totals.by_year[1].amount_eur, dec!(6753));
        assert_eq!(totals.by_year[2].amount_eur, dec!(6000));
        assert_eq!(totals.by_year[3].amount_eur, Decimal::ZERO);
        assert_eq!(totals.by_year[4].amount_eur, Decimal::ZERO);
        assert_eq!(totals.total, dec!(21661));
    }

    #[test]
    fn test_calc_12_multiple_trips_same_year() {
        let trip_costs = vec![(1, dec!(3000)), (1, dec!(2000))];
        let totals = aggregate_travel_by_year(&trip_costs, 2).unwrap();
        assert_eq!(totals.by_year[0].amount_eur, dec!(5000));
    }

    #[test]
    fn test_calc_12_no_trips_all_zeros() {
        let totals = aggregate_travel_by_year(&[], 5).unwrap();
        assert_eq!(totals.total, Decimal::ZERO);
        assert_eq!(totals.by_year.len(), 5);
    }

    #[test]
    fn test_calc_12_year_out_of_range_returns_error() {
        let trip_costs = vec![(6, dec!(1000))]; // project is 5 years
        let result = aggregate_travel_by_year(&trip_costs, 5);
        assert!(matches!(result, Err(AppError::Calculation { code, .. }) if code == "YEAR_OUT_OF_RANGE"));
    }
}
