//! EU travel rate table data loading and lookup.
//!
//! Rate tables are embedded in the application binary at compile time.
//! They are loaded once at startup into `RateData` and reused for all calculations.
//!
//! JSON field names are preserved as-is; Rust struct field names may differ
//! and are mapped via `#[serde(rename)]`.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::error::AppError;

// ─── Rate Version ─────────────────────────────────────────────────────────────

/// A single version of the EU Annex 2a/2b rate tables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateVersion {
    pub version_id: String,
    /// Human-readable label for UI display. Stored as "description" in JSON.
    #[serde(rename = "description")]
    pub version_label: String,
    /// ISO date string (YYYY-MM-DD). Stored as "valid_from" in JSON.
    #[serde(rename = "valid_from")]
    pub applicable_from: String,
    /// Optional end date. Present in older version files.
    #[serde(rename = "valid_until", default)]
    pub valid_until: Option<String>,
    pub flight_bands: Vec<FlightBand>,
    /// Per-country accommodation and subsistence rates. Stored as "countries" in JSON.
    #[serde(rename = "countries")]
    pub country_rates: Vec<CountryRate>,
}

/// A single flight distance band with its EU unit cost.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightBand {
    /// Band identifier (e.g. "F-01"). Stored as "code" in JSON.
    #[serde(rename = "code")]
    pub band_id: String,
    /// Human-readable label (e.g. "Up to 600 km").
    pub label: String,
    /// Inclusive minimum one-way distance in km.
    pub min_km: u32,
    /// Inclusive maximum one-way distance in km.
    /// The last band uses 999999 to indicate "unbounded".
    pub max_km: u32,
    /// EU unit cost per round trip. Stored as integer in JSON; converted to Decimal on use.
    #[serde(rename = "rate_eur")]
    pub cost_eur: u32,
}

impl FlightBand {
    pub fn cost_decimal(&self) -> Decimal {
        Decimal::from(self.cost_eur)
    }

    /// Returns true if `distance_km` falls within this band's range (inclusive).
    pub fn contains(&self, distance_km: u32) -> bool {
        distance_km >= self.min_km && distance_km <= self.max_km
    }
}

/// Accommodation and subsistence rates for a single country.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryRate {
    pub country_code: String,
    pub country_name: String,
    /// Maximum accommodation rate per night (integer EUR in JSON).
    pub accommodation_eur: u32,
    /// Maximum subsistence rate per day (integer EUR in JSON).
    pub subsistence_eur: u32,
}

impl CountryRate {
    pub fn accommodation_decimal(&self) -> Decimal {
        Decimal::from(self.accommodation_eur)
    }

    pub fn subsistence_decimal(&self) -> Decimal {
        Decimal::from(self.subsistence_eur)
    }
}

// ─── RateData Container ───────────────────────────────────────────────────────

/// All loaded rate versions. Loaded once at startup and kept in `AppState`.
#[derive(Debug, Clone)]
pub struct RateData {
    pub versions: Vec<RateVersion>,
}

impl RateData {
    /// Load all rate versions from the JSON files embedded in the binary.
    pub fn load_embedded() -> Result<Self, AppError> {
        let v_from_2025 = include_str!("../../resources/eu_travel_rates/v_from_2025_05_13.json");
        let v_2024_2025 = include_str!("../../resources/eu_travel_rates/v_2024_07_31_to_2025_05_12.json");
        let v_before_2024 = include_str!("../../resources/eu_travel_rates/v_before_2024_07_31.json");

        let mut versions = Vec::new();
        for raw in [v_from_2025, v_2024_2025, v_before_2024] {
            let version: RateVersion = serde_json::from_str(raw)
                .map_err(|e| AppError::Internal(format!("Failed to parse EU rate data: {e}")))?;
            versions.push(version);
        }

        Ok(Self { versions })
    }

    /// Find a rate version by ID.
    pub fn find_version(&self, version_id: &str) -> Option<&RateVersion> {
        self.versions.iter().find(|v| v.version_id == version_id)
    }

    /// Get summary info for all versions (for UI dropdowns).
    pub fn version_summaries(&self) -> Vec<RateVersionSummary> {
        self.versions
            .iter()
            .map(|v| RateVersionSummary {
                version_id: v.version_id.clone(),
                version_label: v.version_label.clone(),
                applicable_from: v.applicable_from.clone(),
            })
            .collect()
    }
}

/// Lightweight version descriptor for the UI dropdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateVersionSummary {
    pub version_id: String,
    pub version_label: String,
    pub applicable_from: String,
}

impl RateVersion {
    /// Look up the flight band for the given one-way distance.
    /// Returns None if distance < 400 km (no flight; use train/bus at flat subsistence).
    pub fn find_flight_band(&self, one_way_distance_km: u32) -> Option<&FlightBand> {
        if one_way_distance_km < 400 {
            return None;
        }
        self.flight_bands
            .iter()
            .find(|b| b.contains(one_way_distance_km))
    }

    /// Look up accommodation and subsistence rates for a country (ISO alpha-2 code).
    /// Falls back to the "OTHER" entry if the country code is not found.
    pub fn find_country_rates(&self, country_code: &str) -> Option<&CountryRate> {
        let code = country_code.to_uppercase();
        self.country_rates
            .iter()
            .find(|c| c.country_code == code)
            .or_else(|| self.country_rates.iter().find(|c| c.country_code == "OTHER"))
    }

    /// Return all countries sorted by name (for UI dropdowns).
    pub fn sorted_countries(&self) -> Vec<CountrySummary> {
        let mut countries: Vec<CountrySummary> = self.country_rates
            .iter()
            .filter(|c| c.country_code != "OTHER") // "Other" goes at the end
            .map(|c| CountrySummary {
                country_code: c.country_code.clone(),
                country_name: c.country_name.clone(),
                accommodation_eur_per_night: c.accommodation_decimal(),
                subsistence_eur_per_day: c.subsistence_decimal(),
            })
            .collect();
        countries.sort_by(|a, b| a.country_name.cmp(&b.country_name));

        // Append "Other (not listed)" at the bottom
        if let Some(other) = self.country_rates.iter().find(|c| c.country_code == "OTHER") {
            countries.push(CountrySummary {
                country_code: other.country_code.clone(),
                country_name: other.country_name.clone(),
                accommodation_eur_per_night: other.accommodation_decimal(),
                subsistence_eur_per_day: other.subsistence_decimal(),
            });
        }

        countries
    }
}

/// Country descriptor for the UI travel form dropdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountrySummary {
    pub country_code: String,
    pub country_name: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub accommodation_eur_per_night: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub subsistence_eur_per_day: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test guarding the embedded EU rate JSON files against parse
    /// errors and gross data corruption — the tables themselves are
    /// transcribed from the official "EU Grants: Additional Information on
    /// Unit Costs and Contributions" Annex 2a/2b, so this doesn't re-verify
    /// every value, just structural sanity (loads, has an OTHER fallback,
    /// every version's flight bands cover 400km+ with no gaps).
    #[test]
    fn test_load_embedded_all_versions_parse_and_are_well_formed() {
        let rate_data = RateData::load_embedded().expect("embedded rate JSON must parse");
        assert_eq!(rate_data.versions.len(), 3);

        for version in &rate_data.versions {
            assert!(
                version.country_rates.iter().any(|c| c.country_code == "OTHER"),
                "version {} is missing the OTHER fallback country",
                version.version_id
            );
            assert!(!version.flight_bands.is_empty());

            let mut bands = version.flight_bands.clone();
            bands.sort_by_key(|b| b.min_km);
            assert_eq!(bands[0].min_km, 400, "version {} flight bands must start at 400km", version.version_id);
            for w in bands.windows(2) {
                assert_eq!(
                    w[0].max_km + 1,
                    w[1].min_km,
                    "version {} has a gap/overlap between flight bands {} and {}",
                    version.version_id, w[0].band_id, w[1].band_id
                );
            }
        }
    }

    #[test]
    fn test_current_version_matches_official_source_values() {
        // Spot-checks a handful of values from the "as from 13 May 2025"
        // table against the source PDF, to catch a bad transcription/edit.
        let rate_data = RateData::load_embedded().unwrap();
        let current = rate_data.find_version("from_2025_05_13").unwrap();

        let at = current.find_country_rates("AT").unwrap();
        assert_eq!(at.accommodation_eur, 158);
        assert_eq!(at.subsistence_eur, 131);

        let other = current.find_country_rates("ZZ").unwrap(); // falls back to OTHER
        assert_eq!(other.country_code, "OTHER");
        assert_eq!(other.accommodation_eur, 145);
        assert_eq!(other.subsistence_eur, 60);

        // Flight band for 1900km one-way should be the 1601-2500 band (429).
        let band = current.find_flight_band(1900).unwrap();
        assert_eq!(band.cost_eur, 429);
    }
}
