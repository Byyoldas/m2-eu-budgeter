//! Domain entities — the core data model shared by the ERC Budget and ERC
//! Execution applications.
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
    /// Category B — Subcontracting items. `#[serde(default)]` so a pre-1.2
    /// file (which had a single `subcontracting` lump sum instead of this
    /// list) still loads — `persistence::load_project` migrates that old
    /// value into a one-item list on first load, see there.
    #[serde(default)]
    pub subcontracting_items: Vec<SubcontractingItem>,
    /// True when the user dismissed the CFS modal without entering an amount.
    pub cfs_warning_dismissed: bool,
}

impl Project {
    /// Create a new empty project from configuration.
    pub fn new(config: ProjectConfig) -> Self {
        let mut project = Self {
            id: Uuid::new_v4(),
            config,
            personnel_roles: Vec::new(),
            equipment_items: Vec::new(),
            trips: Vec::new(),
            other_cost_items: Vec::new(),
            subcontracting_items: Vec::new(),
            cfs_warning_dismissed: false,
        };
        project.enforce_funding_model();
        project
    }

    /// For an Actual Cost project, collapse the Work-Package set to the single
    /// implicit WP and point every existing cost item at it. A no-op for Lump
    /// Sum projects. Idempotent — safe to call after any config change.
    pub fn enforce_funding_model(&mut self) {
        if !self.config.is_actual_cost() {
            return;
        }
        self.config.collapse_to_single_work_package();
        for item in &mut self.equipment_items {
            item.work_package_id = 1;
        }
        for trip in &mut self.trips {
            trip.work_package_ids = vec![1];
        }
        for item in &mut self.other_cost_items {
            item.work_package_ids = vec![1];
        }
        for item in &mut self.subcontracting_items {
            item.work_package_ids = vec![1];
        }
    }

    pub fn has_cfs_item(&self) -> bool {
        self.other_cost_items.iter().any(|i| i.is_cfs_item)
    }
}

// ─── Funding Model ────────────────────────────────────────────────────────────

/// How the grant is funded and therefore how its budget is built and reported.
///
/// - `LumpSum`: the budget is organised by Work Package (each WP has its own
///   timeline and its own cost lines). This is how every project created
///   before funding models existed behaves, so it is also the serde default.
/// - `ActualCost`: only the total cost per category matters. There is no
///   per-Work-Package breakdown — the project is held as a single implicit
///   Work Package covering the whole duration (see
///   [`ProjectConfig::collapse_to_single_work_package`]), and cost items
///   carry no WP selection of their own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(export))]
pub enum FundingModel {
    ActualCost,
    #[default]
    LumpSum,
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
    /// Date the call was published (ISO 8601 date string, for display). This
    /// is an administrative EC-calendar fact used only to pick the right
    /// `rate_version_id` -- it is NOT the project's own start date, which is
    /// usually months or years later (after evaluation and Grant Agreement
    /// signature). See `project_start_date` for that.
    pub call_opening_date: Option<String>,
    /// The project's actual official start date (ISO 8601 date string) --
    /// when the Grant Agreement takes effect and "Month 1" truly begins.
    /// `None` until set; execution/progress tracking (current project
    /// month, calendar-year mapping for reporting) falls back to Month 1
    /// when unset, the same precedent as every other calendar-anchored
    /// calculation in this codebase.
    #[serde(default)]
    pub project_start_date: Option<String>,
    /// Actual Cost vs Lump Sum funding. Defaults to `LumpSum` for files saved
    /// before this field existed (they were always Work-Package based).
    #[serde(default)]
    pub funding_model: FundingModel,
}

impl ProjectConfig {
    /// True when the budget is total-cost based, with no Work-Package breakdown.
    pub fn is_actual_cost(&self) -> bool {
        self.funding_model == FundingModel::ActualCost
    }

    /// Replace the Work-Package set with one implicit WP spanning the whole
    /// project. Actual Cost projects have no Work Packages of their own, but
    /// the calculation engine still aggregates into WP buckets — a single
    /// all-covering bucket makes "the WP total" equal "the project total" and
    /// keeps every WP-coverage invariant (no uncovered months) trivially true.
    pub fn collapse_to_single_work_package(&mut self) {
        self.work_package_count = 1;
        self.work_package_names = vec![None];
        self.work_package_start_months = vec![1];
        self.work_package_end_months = vec![self.duration_years as u32 * 12];
    }

    /// Derive explicit `WorkPackage` structs from the array-based representation.
    ///
    /// The Budget Application continues to store WPs as parallel arrays
    /// (`work_package_names`/`work_package_start_months`/`work_package_end_months`)
    /// for backward file compatibility; this promotes them to a first-class
    /// entity for consumers (e.g. the Execution Application) that need to
    /// attach tasks, deliverables, or milestones to a specific WP.
    pub fn work_packages(&self) -> Vec<WorkPackage> {
        (0..self.work_package_count as usize)
            .map(|i| WorkPackage {
                id: (i + 1) as u8,
                name: self.work_package_names.get(i).cloned().flatten(),
                start_month: self.work_package_start_months.get(i).copied().unwrap_or(1),
                end_month: self
                    .work_package_end_months
                    .get(i)
                    .copied()
                    .unwrap_or(self.duration_years as u32 * 12),
            })
            .collect()
    }
}

/// Explicit WorkPackage entity, derived from `ProjectConfig`'s parallel
/// arrays (see [`ProjectConfig::work_packages`]). Not stored separately in
/// `.ercbudget` v1.0 files — it is a read view, not a new persisted field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkPackage {
    pub id: u8,
    pub name: Option<String>,
    pub start_month: u32,
    pub end_month: u32,
}

// ─── Personnel ────────────────────────────────────────────────────────────────

/// Generic role type. Determines the role prefix in the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(export))]
pub enum RoleType {
    Pi,
    Expert,
    PostDoc,
    PhdStudent,
    MscStudent,
    Admin,
}

/// Currency a role's monthly salary was entered in.
///
/// Serialised as `"TRY"` / `"EUR"`. `Try` is the default so projects saved
/// before this existed (all salaries were TRY) keep loading unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "UPPERCASE")]
pub enum SalaryCurrency {
    #[default]
    Try,
    Eur,
}

/// A single staff member charged to the grant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonnelRole {
    pub id: Uuid,
    /// Generic label unique within the project, e.g. "PostDoc-1".
    pub role_label: String,
    pub role_type: RoleType,
    /// Current monthly gross salary, in `salary_currency`. Basis for projection.
    ///
    /// The field keeps its original `_try` name for `.ercbudget` file
    /// compatibility (it predates EUR salaries); when `salary_currency` is
    /// `Eur` this holds a EUR amount and no TRY conversion is applied.
    #[serde(with = "rust_decimal::serde::str")]
    pub current_monthly_salary_try: Decimal,
    /// Currency `current_monthly_salary_try` is expressed in.
    #[serde(default)]
    pub salary_currency: SalaryCurrency,
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
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(export))]
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
        #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
        domestic_transport_per_instance_eur: Decimal,
    },
    /// User enters the total cost per trip instance directly.
    FlatAmount {
        #[serde(with = "rust_decimal::serde::str")]
        #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
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

/// A single item in Category B — Subcontracting. Entered item-by-item, same
/// shape as `OtherDirectCostItem`: cost is split evenly across every listed
/// Work Package (see `wp_budget::aggregate_wp_budgets`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubcontractingItem {
    pub id: Uuid,
    pub name: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount_eur: Decimal,
    pub notes: Option<String>,
    pub work_package_ids: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(
        wp_count: u8,
        names: Vec<Option<&str>>,
        starts: Vec<u32>,
        ends: Vec<u32>,
    ) -> ProjectConfig {
        ProjectConfig {
            project_title: "Test".to_string(),
            pi_name: "PI".to_string(),
            call_reference: "ERC-2025-CoG".to_string(),
            duration_years: 5,
            work_package_count: wp_count,
            work_package_names: names.into_iter().map(|n| n.map(String::from)).collect(),
            work_package_start_months: starts,
            work_package_end_months: ends,
            default_inflation_rate_pct: Decimal::ZERO,
            try_eur_rate: Decimal::ONE,
            indirect_cost_rate_pct: Decimal::ZERO,
            rate_version_id: "from_2025_05_13".to_string(),
            call_opening_date: None,
            project_start_date: None,
            funding_model: FundingModel::LumpSum,
        }
    }

    #[test]
    fn test_work_packages_derives_from_arrays() {
        let config = make_config(
            2,
            vec![Some("Data Collection"), None],
            vec![1, 19],
            vec![18, 60],
        );
        let wps = config.work_packages();
        assert_eq!(wps.len(), 2);
        assert_eq!(wps[0].id, 1);
        assert_eq!(wps[0].name.as_deref(), Some("Data Collection"));
        assert_eq!(wps[0].start_month, 1);
        assert_eq!(wps[0].end_month, 18);
        assert_eq!(wps[1].id, 2);
        assert_eq!(wps[1].name, None);
        assert_eq!(wps[1].start_month, 19);
        assert_eq!(wps[1].end_month, 60);
    }

    #[test]
    fn test_work_packages_falls_back_when_arrays_empty() {
        // Backward compatibility: files saved before start/end months existed
        // have empty arrays (`#[serde(default)]`); derive full-project-span
        // defaults instead of panicking or losing WPs.
        let config = make_config(1, vec![None], vec![], vec![]);
        let wps = config.work_packages();
        assert_eq!(wps.len(), 1);
        assert_eq!(wps[0].id, 1);
        assert_eq!(wps[0].start_month, 1);
        assert_eq!(wps[0].end_month, 60); // duration_years(5) * 12
    }

    #[test]
    fn test_project_new_sets_defaults() {
        let config = make_config(1, vec![None], vec![1], vec![60]);
        let project = Project::new(config);
        assert!(project.personnel_roles.is_empty());
        assert!(project.equipment_items.is_empty());
        assert!(project.trips.is_empty());
        assert!(project.other_cost_items.is_empty());
        assert!(project.subcontracting_items.is_empty());
        assert!(!project.cfs_warning_dismissed);
        assert!(!project.has_cfs_item());
    }

    #[test]
    fn test_project_has_cfs_item() {
        let config = make_config(1, vec![None], vec![1], vec![60]);
        let mut project = Project::new(config);
        assert!(!project.has_cfs_item());
        project.other_cost_items.push(OtherDirectCostItem {
            id: Uuid::new_v4(),
            name: "CFS".to_string(),
            amount_eur: Decimal::from(1000),
            is_cfs_item: true,
            notes: None,
            work_package_ids: vec![],
        });
        assert!(project.has_cfs_item());
    }
    // ── Funding model ──────────────────────────────────────────────────────

    #[test]
    fn test_legacy_config_json_without_funding_model_defaults_to_lump_sum() {
        let json = r#"{
            "project_title": "Old", "pi_name": "PI", "call_reference": "ERC",
            "duration_years": 1, "work_package_count": 1,
            "work_package_names": [null],
            "default_inflation_rate_pct": "0", "try_eur_rate": "50",
            "indirect_cost_rate_pct": "25", "rate_version_id": "v",
            "call_opening_date": null
        }"#;
        let config: ProjectConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.funding_model, FundingModel::LumpSum);
        assert!(!config.is_actual_cost());
    }

    #[test]
    fn test_actual_cost_project_collapses_to_one_all_covering_work_package() {
        let mut config = make_config(
            3,
            vec![Some("A"), Some("B"), None],
            vec![1, 13, 37],
            vec![24, 48, 60],
        );
        config.funding_model = FundingModel::ActualCost;
        let project = Project::new(config);
        assert_eq!(project.config.work_package_count, 1);
        assert_eq!(project.config.work_package_names, vec![None]);
        assert_eq!(project.config.work_package_start_months, vec![1]);
        assert_eq!(project.config.work_package_end_months, vec![60]);
    }

    #[test]
    fn test_enforce_funding_model_repoints_existing_items_to_wp1() {
        let mut project = Project::new(make_config(
            3,
            vec![None, None, None],
            vec![1, 1, 1],
            vec![60, 60, 60],
        ));
        project.trips.push(Trip {
            id: Uuid::new_v4(),
            name: "T".to_string(),
            trip_type: TripType::FlatAmount {
                flat_amount_per_instance_eur: Decimal::ONE,
            },
            number_of_instances: 1,
            work_package_ids: vec![2, 3],
        });
        project.other_cost_items.push(OtherDirectCostItem {
            id: Uuid::new_v4(),
            name: "O".to_string(),
            amount_eur: Decimal::ONE,
            is_cfs_item: false,
            notes: None,
            work_package_ids: vec![3],
        });
        // Lump Sum: untouched.
        project.enforce_funding_model();
        assert_eq!(project.trips[0].work_package_ids, vec![2, 3]);
        assert_eq!(project.config.work_package_count, 3);

        // Switch to Actual Cost: everything collapses to WP1.
        project.config.funding_model = FundingModel::ActualCost;
        project.enforce_funding_model();
        assert_eq!(project.config.work_package_count, 1);
        assert_eq!(project.trips[0].work_package_ids, vec![1]);
        assert_eq!(project.other_cost_items[0].work_package_ids, vec![1]);
    }

    #[test]
    fn test_salary_currency_serialises_uppercase_and_defaults_to_try() {
        assert_eq!(
            serde_json::to_string(&SalaryCurrency::Eur).unwrap(),
            "\"EUR\""
        );
        assert_eq!(
            serde_json::to_string(&SalaryCurrency::Try).unwrap(),
            "\"TRY\""
        );
        assert_eq!(SalaryCurrency::default(), SalaryCurrency::Try);
        let role: PersonnelRole = serde_json::from_str(
            r#"{"id":"11111111-1111-4111-8111-111111111111","role_label":"R",
                "role_type":"Expert","current_monthly_salary_try":"100",
                "fte_fraction":"1","inflation_rate_pct":"0",
                "start_month":1,"end_month":12}"#,
        )
        .unwrap();
        assert_eq!(role.salary_currency, SalaryCurrency::Try);
    }
}
