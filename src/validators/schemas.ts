/**
 * Zod schemas for front-end form validation.
 * These run instantly (no IPC round-trip) for basic field-level checks.
 * Cross-entity constraints are enforced server-side.
 */

import { z } from 'zod';

// Positive decimal string helper
const decimalStr = (label: string) =>
  z.string().refine((v) => !isNaN(parseFloat(v)) && parseFloat(v) > 0, {
    message: `${label} must be a positive number.`,
  });

const nonNegDecimalStr = (label: string) =>
  z.string().refine((v) => !isNaN(parseFloat(v)) && parseFloat(v) >= 0, {
    message: `${label} must be zero or a positive number.`,
  });

// ─── Project Setup Schema ─────────────────────────────────────────────────────

export const projectSetupSchema = z.object({
  project_title: z.string().min(1, 'Project title is required.'),
  pi_name: z.string().min(1, 'PI name is required.'),
  call_reference: z.string().min(1, 'Call reference is required.'),
  duration_years: z.coerce
    .number()
    .int()
    .min(1, 'Duration must be at least 1 year.')
    .max(7, 'Duration cannot exceed 7 years.'),
  work_package_count: z.coerce
    .number()
    .int()
    .min(1, 'At least 1 Work Package required.')
    .max(10, 'Maximum 10 Work Packages.'),
  call_opening_date: z.string().nullable().optional(),
});

export type ProjectSetupFormData = z.infer<typeof projectSetupSchema>;

// ─── Budget Settings Schema ────────────────────────────────────────────────────

export const budgetSettingsSchema = z.object({
  try_eur_rate: z
    .string()
    .refine((v) => !isNaN(parseFloat(v)) && parseFloat(v) > 0, {
      message: 'Exchange rate must be greater than zero.',
    }),
  default_inflation_rate_pct: z
    .string()
    .refine((v) => {
      const n = parseFloat(v);
      return !isNaN(n) && n >= 0 && n <= 100;
    }, { message: 'Inflation rate must be between 0% and 100%.' }),
  indirect_cost_rate_pct: z
    .string()
    .refine((v) => {
      const n = parseFloat(v);
      return !isNaN(n) && n >= 0 && n <= 50;
    }, { message: 'Indirect rate must be between 0% and 50%.' }),
  rate_version_id: z.string().min(1, 'Please select a rate version.'),
});

export type BudgetSettingsFormData = z.infer<typeof budgetSettingsSchema>;

// ─── Personnel Role Schema ────────────────────────────────────────────────────

export const personnelRoleSchema = z.object({
  role_label: z.string().min(1, 'Role label is required.'),
  role_type: z.enum(['Pi', 'Expert', 'PostDoc', 'PhdStudent', 'Admin'] as const),
  current_monthly_salary_try: decimalStr('Monthly salary (TRY)'),
  fte_fraction: z
    .string()
    .refine((v) => {
      const n = parseFloat(v);
      return !isNaN(n) && n > 0 && n <= 1;
    }, { message: 'PM must be greater than 0 and at most 1.0.' }),
  inflation_rate_pct: z
    .string()
    .refine((v) => {
      const n = parseFloat(v);
      return !isNaN(n) && n >= 0 && n <= 100;
    }, { message: 'Inflation rate must be between 0% and 100%.' }),
  active_years: z.array(z.number().int().positive()).min(1, 'Select at least one active year.'),
  work_package_ids: z.array(z.number().int().positive()),
});

export type PersonnelRoleFormData = z.infer<typeof personnelRoleSchema>;

// ─── Equipment Item Schema ────────────────────────────────────────────────────

export const equipmentItemSchema = z.object({
  name: z.string().min(1, 'Item name is required.'),
  purchase_cost_eur: decimalStr('Purchase cost'),
  useful_lifetime_months: z.coerce.number().int().min(1, 'Useful lifetime must be at least 1 month.'),
  grant_usage_pct: z
    .string()
    .refine((v) => {
      const n = parseFloat(v);
      return !isNaN(n) && n > 0 && n <= 100;
    }, { message: 'Grant usage must be between 0% (exclusive) and 100%.' }),
  grant_usage_months: z.coerce.number().int().min(1, 'Usage months must be at least 1.'),
  year_of_purchase: z.coerce.number().int().positive().nullable().optional(),
  work_package_ids: z.array(z.number().int().positive()),
});

export type EquipmentItemFormData = z.infer<typeof equipmentItemSchema>;

// ─── Trip Schemas ─────────────────────────────────────────────────────────────

export const itemizedTripSchema = z.object({
  name: z.string().min(1, 'Trip name is required.'),
  trip_kind: z.literal('Itemized'),
  destination_country_code: z.string().min(1, 'Destination country is required.'),
  one_way_distance_km: z.coerce.number().int().min(0, 'Distance must be 0 or greater.'),
  number_of_nights: z.coerce.number().int().min(1, 'At least 1 night required.'),
  number_of_days: z.coerce.number().int().min(1, 'At least 1 day required.'),
  domestic_transport_per_instance_eur: nonNegDecimalStr('Domestic transport'),
  project_year: z.coerce.number().int().min(1),
  number_of_instances: z.coerce.number().int().min(1, 'At least 1 instance required.'),
  work_package_id: z.coerce.number().int().positive().nullable().optional(),
});

export const flatTripSchema = z.object({
  name: z.string().min(1, 'Trip name is required.'),
  trip_kind: z.literal('FlatAmount'),
  flat_amount_per_instance_eur: decimalStr('Flat amount'),
  project_year: z.coerce.number().int().min(1),
  number_of_instances: z.coerce.number().int().min(1, 'At least 1 instance required.'),
  work_package_id: z.coerce.number().int().positive().nullable().optional(),
});

export const tripSchema = z.discriminatedUnion('trip_kind', [itemizedTripSchema, flatTripSchema]);
export type TripFormData = z.infer<typeof tripSchema>;

// ─── Other Cost Schema ────────────────────────────────────────────────────────

export const otherCostSchema = z.object({
  name: z.string().min(1, 'Item name is required.'),
  amount_eur: decimalStr('Amount'),
  project_year: z.coerce.number().int().min(1),
  notes: z.string().nullable().optional(),
  work_package_id: z.coerce.number().int().positive().nullable().optional(),
});

export type OtherCostFormData = z.infer<typeof otherCostSchema>;

// ─── CFS Item Schema ──────────────────────────────────────────────────────────

export const cfsItemSchema = z.object({
  amount_eur: decimalStr('CFS amount'),
  project_year: z.coerce.number().int().min(1),
});

export type CfsItemFormData = z.infer<typeof cfsItemSchema>;

// ─── Subcontracting Schema ────────────────────────────────────────────────────

export const subcontractingSchema = z.object({
  amount_eur: nonNegDecimalStr('Subcontracting amount'),
});

export type SubcontractingFormData = z.infer<typeof subcontractingSchema>;
