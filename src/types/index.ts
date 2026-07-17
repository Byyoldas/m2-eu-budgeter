/**
 * TypeScript types mirroring the Rust domain DTOs.
 * All monetary Decimal values are strings (serialised by Rust as "rust_decimal::serde::str").
 */

// ─── Enums ─────────────────────────────────────────────────────────────────────

export type RoleType = 'Pi' | 'Expert' | 'PostDoc' | 'PhdStudent' | 'MscStudent' | 'Admin';

export type CfsStatus =
  | 'NOT_REQUIRED'
  | 'REQUIRED_AND_PRESENT'
  | 'REQUIRED_BUT_DISMISSED'
  | 'REQUIRED_AND_UNADDRESSED';

// ─── Trip Types ─────────────────────────────────────────────────────────────────

export interface ItemizedTripType {
  Itemized: {
    destination_country_code: string;
    one_way_distance_km: number;
    number_of_nights: number;
    number_of_days: number;
    domestic_transport_per_instance_eur: string; // Decimal
  };
}

export interface FlatAmountTripType {
  FlatAmount: {
    flat_amount_per_instance_eur: string; // Decimal
  };
}

export type TripType = ItemizedTripType | FlatAmountTripType;

// ─── Input DTOs (frontend → backend) ──────────────────────────────────────────

export interface ProjectConfigInput {
  project_title: string;
  pi_name: string;
  call_reference: string;
  duration_years: number;
  work_package_count: number;
  work_package_names: (string | null)[];
  work_package_start_months: number[];
  work_package_end_months: number[];
  default_inflation_rate_pct: string; // Decimal
  try_eur_rate: string;               // Decimal
  indirect_cost_rate_pct: string;     // Decimal
  rate_version_id: string;
  call_opening_date: string | null;
}

export interface PersonnelRoleInput {
  role_label: string;
  role_type: RoleType;
  current_monthly_salary_try: string; // Decimal
  fte_fraction: string;               // Decimal
  inflation_rate_pct: string;         // Decimal
  start_month: number;
  end_month: number;
}

export interface EquipmentItemInput {
  name: string;
  purchase_cost_eur: string;       // Decimal
  useful_lifetime_months: number;
  grant_usage_pct: string;         // Decimal
  grant_usage_months: number;
  work_package_id: number;
}

export interface TripInput {
  name: string;
  trip_type: TripType;
  number_of_instances: number;
  work_package_ids: number[];
}

export interface OtherCostInput {
  name: string;
  amount_eur: string;  // Decimal
  notes: string | null;
  work_package_ids: number[];
}

// ─── Output DTOs (backend → frontend) ────────────────────────────────────────

export interface WpCostAmountDto {
  work_package_id: number;
  amount_eur: string; // Decimal
}

export interface RoleCostLineDto {
  year: number;
  is_active: boolean;
  active_months: number;
  monthly_salary_eur: string; // Decimal
  annual_cost_eur: string;    // Decimal
}

export interface PersonnelRoleDetailDto {
  id: string; // UUID
  role_label: string;
  role_type: RoleType;
  current_monthly_salary_try: string; // Decimal
  inflation_rate_pct: string; // Decimal
  fte_fraction: string; // Decimal
  start_month: number;
  end_month: number;
  cost_lines: RoleCostLineDto[];
  total_cost_eur: string; // Decimal
  wp_breakdown: WpCostAmountDto[];
}

export interface RoleCostPreviewDto {
  base_monthly_eur: string; // Decimal
  cost_lines: RoleCostLineDto[];
  total_cost_eur: string;   // Decimal
  wp_breakdown: WpCostAmountDto[];
}

export interface EquipmentItemDetailDto {
  id: string; // UUID
  name: string;
  purchase_cost_eur: string; // Decimal
  useful_lifetime_months: number;
  grant_usage_pct: string; // Decimal
  grant_usage_months: number;
  work_package_id: number;
  theoretical_eligible_eur: string; // Decimal
  maximum_eligible_eur: string;     // Decimal
  is_capped: boolean;
  eligible_depreciation_eur: string; // Decimal
}

export interface EquipmentPreviewDto {
  theoretical_eligible_eur: string;
  maximum_eligible_eur: string;
  is_capped: boolean;
  eligible_depreciation_eur: string;
}

export interface OtherCostItemDetailDto {
  id: string; // UUID
  name: string;
  amount_eur: string; // Decimal
  is_cfs_item: boolean;
  notes: string | null;
  work_package_ids: number[];
}

export interface TripDetailDto {
  id: string; // UUID
  name: string;
  work_package_ids: number[];
  number_of_instances: number;
  destination_country_code: string | null;
  one_way_distance_km: number | null;
  number_of_nights: number | null;
  number_of_days: number | null;
  flight_cost_per_instance: string | null;
  accommodation_cost_per_instance: string | null;
  subsistence_cost_per_instance: string | null;
  domestic_transport_per_instance: string | null;
  per_instance_total_eur: string; // Decimal
  total_trip_cost_eur: string;    // Decimal
}

export interface TripCostPreviewDto {
  flight_cost_per_instance: string | null;
  accommodation_cost_per_instance: string | null;
  subsistence_cost_per_instance: string | null;
  domestic_transport_per_instance: string | null;
  per_instance_total_eur: string;
  total_trip_cost_eur: string;
  flight_band_label: string | null;
  no_flight_applicable: boolean;
  accommodation_rate_eur: string | null;
  subsistence_rate_eur: string | null;
}

export interface WpBudgetDto {
  work_package_id: number;
  work_package_name: string | null;
  personnel_eur: string;      // Decimal
  equipment_eur: string;      // Decimal
  travel_eur: string;         // Decimal
  other_costs_eur: string;    // Decimal
  subcontracting_eur: string; // Decimal
  total_eur: string;          // Decimal
}

export interface BudgetSummaryDto {
  wp_budgets: WpBudgetDto[];
  // Personnel (A)
  category_a_total: string;
  // Subcontracting (B)
  category_b_total: string;
  // Travel (C1)
  category_c1_total: string;
  // Equipment (C2)
  category_c2_total: string;
  // Other Direct Costs (C3)
  category_c3_total: string;
  // Indirect (E)
  indirect_base_total: string;
  category_e_total: string;
  // Totals
  total_direct_costs: string;
  total_eligible_costs: string;
  requested_eu_contribution: string;
  // CFS
  cfs_status: CfsStatus;
  cfs_threshold_exceeded: boolean;
  cfs_warning_active: boolean;
  cfs_prompt_required: boolean;
  // Detail rows
  role_detail: PersonnelRoleDetailDto[];
  equipment_detail: EquipmentItemDetailDto[];
  trip_detail: TripDetailDto[];
  other_cost_detail: OtherCostItemDetailDto[];
}

// ─── Rate Data Types ──────────────────────────────────────────────────────────

export interface RateVersionSummary {
  version_id: string;
  version_label: string;
  applicable_from: string;
}

export interface CountrySummary {
  country_code: string;
  country_name: string;
  accommodation_eur_per_night: string; // Decimal
  subsistence_eur_per_day: string;     // Decimal
}

// ─── App Error ────────────────────────────────────────────────────────────────

export interface FieldError {
  field: string | null;
  code: string;
  message: string;
}

export type AppError =
  | { kind: 'Validation'; detail: FieldError[] }
  | { kind: 'Calculation'; detail: { code: string; message: string } }
  | { kind: 'Persistence'; detail: string }
  | { kind: 'NotFound'; detail: string }
  | { kind: 'NoProject' }
  | { kind: 'Internal'; detail: string };

// ─── App UI State ─────────────────────────────────────────────────────────────

export type Screen =
  | 'welcome'
  | 'project-setup'
  | 'budget-settings'
  | 'work-packages'
  | 'personnel'
  | 'equipment'
  | 'travel'
  | 'other-costs'
  | 'review-export';

export const SCREENS: Screen[] = [
  'project-setup',
  'budget-settings',
  'work-packages',
  'personnel',
  'equipment',
  'travel',
  'other-costs',
  'review-export',
];

export const SCREEN_LABELS: Record<Screen, string> = {
  'welcome': 'Welcome',
  'project-setup': 'Project Setup',
  'budget-settings': 'Budget Settings',
  'work-packages': 'Work Packages',
  'personnel': 'Personnel (A)',
  'equipment': 'Equipment (C2)',
  'travel': 'Travel (C1)',
  'other-costs': 'Other Costs (C3)',
  'review-export': 'Review & Export',
};
