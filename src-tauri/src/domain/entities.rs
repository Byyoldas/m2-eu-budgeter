//! Domain entities — the core data model of the ERC Budget application.
//!
//! These structs are the single source of truth for all project data.
//! They are persisted to `.ercbudget` files and fed into the calculation engine.
//!
//! All monetary fields use `rust_decimal::Decimal` to guarantee exact arithmetic.
//! All IDs are `uuid::Uuid` v4.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Project Root ──────────────────────────────────────────────────────────────

/// The root entity. Holds all project data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub config: ProjectConfig,
    pub personnel_roles: Vec<PersonnelRole>,
    pub equipment_items: Vec<EquipmentItem>,
    pub trips: Vec<Trip>,
    pub other_cost_items: Vec<OtherDirectCostItem>,
    pub subcontracting: Subcontracting,
    /// True when the user dismissed the CFS modal without entering an amount.
    pub cfs_warning_dismissed: bool,
}

impl Project {
    /// Create a new empty project from configuration.
    pub fn new(config: ProjectConfig) -> Self {
        Self {
            id: Uuid::new_v4(),
            config,
            personnel_roles: Vec::new(),
            equipment_items: Vec::new(),
            trips: Vec::new(),
            other_cost_items: Vec::new(),
            subcontracting: Subcontracting::default(),
            cfs_warning_dismissed: false,
        }
    }

    pub fn has_cfs_item(&self) -> bool {
        self.other_cost_items.iter().any(|i| i.is_cfs_item)
    }
}

// ─── Project Configuration ─────────────────────────────────────────────────────

/// Project-level parameters that govern all downstream calculations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Administrative: grant project title (display only).
    pub project_title: String,
    /// Administrative: PI name (display only).
    pub pi_name: String,
    /// Administrative: ERC call reference, e.g. "ERC-2025-CoG".
    pub call_reference: String,
    /// Total number of grant years. Range: 1–7.
    pub duration_years: u8,
    /// Number of Work Packages. Range: 1–10.
    pub work_package_count: u8,
    /// Optional descriptive names for each WP.
    /// Length must equal `work_package_count`; entries may be None.
    pub work_package_names: Vec<Option<String>>,
    /// First project month (1-indexed) each WP is active, for the Gantt chart.
    /// Length must equal `work_package_count`. Defaults to empty for files saved
    /// before this field existed.
    #[serde(default)]
    pub work_package_start_months: Vec<u32>,
    /// Last project month (1-indexed, inclusive) each WP is active, for the Gantt chart.
    /// Length must equal `work_package_count`. Defaults to empty for files saved
    /// before this field existed.
    #[serde(default)]
    pub work_package_end_months: Vec<u32>,
    /// Project-level default annual salary inflation rate (%).
    /// Stored as a percentage, e.g. 15.0 means 15%.
    #[serde(with = "rust_decimal::serde::str")]
    pub default_inflation_rate_pct: Decimal,
    /// TRY per 1 EUR exchange rate. Example: 50.62.
    #[serde(with = "rust_decimal::serde::str")]
    pub try_eur_rate: Decimal,
    /// Overhead rate (%). Default 25.0 per ERC rules.
    #[serde(with = "rust_decimal::serde::str")]
    pub indirect_cost_rate_pct: Decimal,
    /// ID of the EU travel rate version to apply.
    /// Tied to the ERC call opening date.
    pub rate_version_id: String,
    /// Date the call was published (ISO 8601 date string, for display).
    pub call_opening_date: Option<String>,
}

// ─── Personnel ────────────────────────────────────────────────────────────────

/// Generic role type. Determines the role prefix in the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoleType {
    Pi,
    Expert,
    PostDoc,
    PhdStudent,
    MscStudent,
    Admin,
}

/// A single staff member charged to the grant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonnelRole {
    pub id: Uuid,
    /// Generic label unique within the project, e.g. "PostDoc-1".
    pub role_label: String,
    pub role_type: RoleType,
    /// Current monthly gross salary in Turkish Lira (TRY). Basis for projection.
    #[serde(with = "rust_decimal::serde::str")]
    pub current_monthly_salary_try: Decimal,
    /// Fraction of working time dedicated to the grant. Range: (0, 1].
    #[serde(with = "rust_decimal::serde::str")]
    pub fte_fraction: Decimal,
    /// Per-role annual salary inflation rate (%). Range: [0, 100].
    #[serde(with = "rust_decimal::serde::str")]
    pub inflation_rate_pct: Decimal,
    /// First project month (1-indexed) this role is charged.
    pub start_month: u32,
    /// Last project month (1-indexed, inclusive) this role is charged.
    pub end_month: u32,
}

// ─── Equipment ────────────────────────────────────────────────────────────────

/// A single equipment item whose depreciation is claimed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentItem {
    pub id: Uuid,
    pub name: String,
    /// Total purchase price in EUR.
    #[serde(with = "rust_decimal::serde::str")]
    pub purchase_cost_eur: Decimal,
    /// Standard economic useful lifetime in months (e.g. 48 for a laptop).
    pub useful_lifetime_months: u32,
    /// Share of total use dedicated to grant activities (%). Range: (0, 100].
    #[serde(with = "rust_decimal::serde::str")]
    pub grant_usage_pct: Decimal,
    /// Months the item is in use during the grant period.
    pub grant_usage_months: u32,
    /// The single Work Package this item's cost is charged to.
    pub work_package_id: u8,
}

// ─── Travel ───────────────────────────────────────────────────────────────────

/// The two supported trip cost calculation modes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TripType {
    /// Cost computed from EU official unit rates (flight + accommodation + subsistence + domestic).
    Itemized {
        destination_country_code: String,
        /// One-way distance in km. 0 means no flight needed.
        one_way_distance_km: u32,
        number_of_nights: u32,
        number_of_days: u32,
        /// Optional in-country transport cost per instance, entered by user.
        #[serde(with = "rust_decimal::serde::str")]
        domestic_transport_per_instance_eur: Decimal,
    },
    /// User enters the total cost per trip instance directly.
    FlatAmount {
        #[serde(with = "rust_decimal::serde::str")]
        flat_amount_per_instance_eur: Decimal,
    },
}

/// A registered trip entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trip {
    pub id: Uuid,
    pub name: String,
    pub trip_type: TripType,
    /// Number of times this trip occurs.
    pub number_of_instances: u32,
    /// The Work Package(s) this trip's cost is charged to. Non-empty; cost is
    /// split evenly across all listed WPs for the per-WP budget view.
    pub work_package_ids: Vec<u8>,
}

// ─── Other Direct Costs (C3) ──────────────────────────────────────────────────

/// A single item in the "Other Goods, Works and Services" category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtherDirectCostItem {
    pub id: Uuid,
    pub name: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount_eur: Decimal,
    /// True for the Certificate on Financial Statements item created by OC-02 auto-trigger.
    pub is_cfs_item: bool,
    pub notes: Option<String>,
    /// The Work Package(s) this item's cost is charged to. Non-empty for regular
    /// items; may be empty for the auto-triggered CFS item. Cost is split evenly
    /// across all listed WPs for the per-WP budget view.
    pub work_package_ids: Vec<u8>,
}

// ─── Subcontracting (B) ───────────────────────────────────────────────────────

/// Category B — Subcontracting. Default is zero.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subcontracting {
    #[serde(with = "rust_decimal::serde::str")]
    pub amount_eur: Decimal,
    /// The Work Package this lump sum is charged to.
    pub work_package_id: u8,
}

impl Default for Subcontracting {
    fn default() -> Self {
        Self {
            amount_eur: Decimal::ZERO,
            work_package_id: 1,
        }
    }
}
