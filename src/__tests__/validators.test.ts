/**
 * Unit tests for Zod validation schemas (validators/schemas.ts).
 *
 * These run purely in the frontend — no Tauri IPC involved.
 * They verify the fast client-side validation that fires before any server call.
 */

import { describe, it, expect } from 'vitest';
import {
  projectSetupSchema,
  budgetSettingsSchema,
  personnelRoleSchema,
  equipmentItemSchema,
  itemizedTripSchema,
  flatTripSchema,
  tripSchema,
  otherCostSchema,
  cfsItemSchema,
  subcontractingSchema,
} from '../validators/schemas';

// ─── projectSetupSchema ───────────────────────────────────────────────────────

describe('projectSetupSchema', () => {
  const validData = {
    project_title: 'My ERC Project',
    pi_name: 'Prof. Jane Smith',
    call_reference: 'ERC-2025-CoG',
    duration_years: 5,
    work_package_count: 3,
  };

  it('accepts a valid 5-year project', () => {
    expect(projectSetupSchema.safeParse(validData).success).toBe(true);
  });

  it('rejects empty project title', () => {
    const r = projectSetupSchema.safeParse({ ...validData, project_title: '' });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].path).toContain('project_title');
  });

  it('rejects empty PI name', () => {
    const r = projectSetupSchema.safeParse({ ...validData, pi_name: '' });
    expect(r.success).toBe(false);
  });

  it('rejects empty call reference', () => {
    const r = projectSetupSchema.safeParse({ ...validData, call_reference: '' });
    expect(r.success).toBe(false);
  });

  it('rejects duration_years = 0', () => {
    const r = projectSetupSchema.safeParse({ ...validData, duration_years: 0 });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].message).toMatch(/at least 1/i);
  });

  it('rejects duration_years = 8 (> max 7)', () => {
    const r = projectSetupSchema.safeParse({ ...validData, duration_years: 8 });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].message).toMatch(/7/);
  });

  it('accepts duration_years = 1 (minimum)', () => {
    expect(projectSetupSchema.safeParse({ ...validData, duration_years: 1 }).success).toBe(true);
  });

  it('accepts duration_years = 7 (maximum)', () => {
    expect(projectSetupSchema.safeParse({ ...validData, duration_years: 7 }).success).toBe(true);
  });

  it('rejects work_package_count = 0', () => {
    const r = projectSetupSchema.safeParse({ ...validData, work_package_count: 0 });
    expect(r.success).toBe(false);
  });

  it('rejects work_package_count = 11 (> max 10)', () => {
    const r = projectSetupSchema.safeParse({ ...validData, work_package_count: 11 });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].message).toMatch(/10/);
  });

  it('accepts work_package_count = 10 (maximum)', () => {
    expect(projectSetupSchema.safeParse({ ...validData, work_package_count: 10 }).success).toBe(true);
  });

  it('coerces string duration_years to number', () => {
    const r = projectSetupSchema.safeParse({ ...validData, duration_years: '5' });
    expect(r.success).toBe(true);
    if (r.success) expect(r.data.duration_years).toBe(5);
  });
});

// ─── budgetSettingsSchema ─────────────────────────────────────────────────────

describe('budgetSettingsSchema', () => {
  const validData = {
    try_eur_rate: '50.62',
    default_inflation_rate_pct: '20',
    indirect_cost_rate_pct: '25',
    rate_version_id: 'v_from_2025_05_13',
  };

  it('accepts valid budget settings', () => {
    expect(budgetSettingsSchema.safeParse(validData).success).toBe(true);
  });

  it('rejects try_eur_rate = "0"', () => {
    const r = budgetSettingsSchema.safeParse({ ...validData, try_eur_rate: '0' });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].message).toMatch(/greater than zero/i);
  });

  it('rejects negative try_eur_rate', () => {
    const r = budgetSettingsSchema.safeParse({ ...validData, try_eur_rate: '-10' });
    expect(r.success).toBe(false);
  });

  it('rejects non-numeric try_eur_rate', () => {
    const r = budgetSettingsSchema.safeParse({ ...validData, try_eur_rate: 'abc' });
    expect(r.success).toBe(false);
  });

  it('accepts 0% inflation (valid lower bound)', () => {
    const r = budgetSettingsSchema.safeParse({ ...validData, default_inflation_rate_pct: '0' });
    expect(r.success).toBe(true);
  });

  it('accepts 100% inflation (valid upper bound)', () => {
    const r = budgetSettingsSchema.safeParse({ ...validData, default_inflation_rate_pct: '100' });
    expect(r.success).toBe(true);
  });

  it('rejects inflation > 100%', () => {
    const r = budgetSettingsSchema.safeParse({ ...validData, default_inflation_rate_pct: '101' });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].message).toMatch(/100/);
  });

  it('rejects indirect rate > 50%', () => {
    const r = budgetSettingsSchema.safeParse({ ...validData, indirect_cost_rate_pct: '51' });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].message).toMatch(/50/);
  });

  it('accepts 0% indirect rate', () => {
    const r = budgetSettingsSchema.safeParse({ ...validData, indirect_cost_rate_pct: '0' });
    expect(r.success).toBe(true);
  });

  it('rejects empty rate_version_id', () => {
    const r = budgetSettingsSchema.safeParse({ ...validData, rate_version_id: '' });
    expect(r.success).toBe(false);
  });
});

// ─── personnelRoleSchema ──────────────────────────────────────────────────────

describe('personnelRoleSchema', () => {
  const validData = {
    role_label: 'PI',
    role_type: 'Pi' as const,
    current_monthly_salary_try: '227900',
    fte_fraction: '0.70',
    inflation_rate_pct: '20',
    start_month: 1,
    end_month: 60,
  };

  it('accepts a valid PI role', () => {
    expect(personnelRoleSchema.safeParse(validData).success).toBe(true);
  });

  it('accepts PostDoc role type', () => {
    expect(personnelRoleSchema.safeParse({ ...validData, role_label: 'PostDoc-1', role_type: 'PostDoc' }).success).toBe(true);
  });

  it('rejects empty role_label', () => {
    const r = personnelRoleSchema.safeParse({ ...validData, role_label: '' });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].path).toContain('role_label');
  });

  it('rejects invalid role_type', () => {
    const r = personnelRoleSchema.safeParse({ ...validData, role_type: 'INVALID' });
    expect(r.success).toBe(false);
  });

  it('rejects zero salary (TRY)', () => {
    const r = personnelRoleSchema.safeParse({ ...validData, current_monthly_salary_try: '0' });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].message).toMatch(/positive/i);
  });

  it('rejects negative salary', () => {
    const r = personnelRoleSchema.safeParse({ ...validData, current_monthly_salary_try: '-1000' });
    expect(r.success).toBe(false);
  });

  it('rejects fte_fraction = "0"', () => {
    const r = personnelRoleSchema.safeParse({ ...validData, fte_fraction: '0' });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].message).toMatch(/greater than 0/i);
  });

  it('rejects fte_fraction > 1', () => {
    const r = personnelRoleSchema.safeParse({ ...validData, fte_fraction: '1.01' });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].message).toMatch(/1\.0/i);
  });

  it('accepts fte_fraction = "1" (100%)', () => {
    expect(personnelRoleSchema.safeParse({ ...validData, fte_fraction: '1' }).success).toBe(true);
  });

  it('accepts fte_fraction = "0.5" (50%)', () => {
    expect(personnelRoleSchema.safeParse({ ...validData, fte_fraction: '0.5' }).success).toBe(true);
  });

  it('rejects inflation > 100%', () => {
    const r = personnelRoleSchema.safeParse({ ...validData, inflation_rate_pct: '101' });
    expect(r.success).toBe(false);
  });

  it('accepts 0% inflation', () => {
    expect(personnelRoleSchema.safeParse({ ...validData, inflation_rate_pct: '0' }).success).toBe(true);
  });

  it('rejects end_month before start_month', () => {
    const r = personnelRoleSchema.safeParse({ ...validData, start_month: 12, end_month: 1 });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].message).toMatch(/on or after/i);
  });

  it('accepts start_month equal to end_month', () => {
    expect(personnelRoleSchema.safeParse({ ...validData, start_month: 5, end_month: 5 }).success).toBe(true);
  });
});

// ─── equipmentItemSchema ──────────────────────────────────────────────────────

describe('equipmentItemSchema', () => {
  const validData = {
    name: 'Laptop',
    purchase_cost_eur: '2500',
    useful_lifetime_months: 48,
    grant_usage_pct: '100',
    grant_usage_months: 55,
    work_package_id: 1,
  };

  it('accepts a valid laptop item', () => {
    expect(equipmentItemSchema.safeParse(validData).success).toBe(true);
  });

  it('rejects empty name', () => {
    const r = equipmentItemSchema.safeParse({ ...validData, name: '' });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].path).toContain('name');
  });

  it('rejects zero purchase cost', () => {
    const r = equipmentItemSchema.safeParse({ ...validData, purchase_cost_eur: '0' });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].message).toMatch(/positive/i);
  });

  it('rejects useful_lifetime_months = 0', () => {
    const r = equipmentItemSchema.safeParse({ ...validData, useful_lifetime_months: 0 });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].message).toMatch(/at least 1/i);
  });

  it('rejects grant_usage_pct = "0"', () => {
    const r = equipmentItemSchema.safeParse({ ...validData, grant_usage_pct: '0' });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].message).toMatch(/0%.*100%|exclusive/i);
  });

  it('rejects grant_usage_pct > 100', () => {
    const r = equipmentItemSchema.safeParse({ ...validData, grant_usage_pct: '101' });
    expect(r.success).toBe(false);
  });

  it('accepts grant_usage_pct = "100" (exactly 100%)', () => {
    expect(equipmentItemSchema.safeParse({ ...validData, grant_usage_pct: '100' }).success).toBe(true);
  });

  it('rejects grant_usage_months = 0', () => {
    const r = equipmentItemSchema.safeParse({ ...validData, grant_usage_months: 0 });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].message).toMatch(/at least 1/i);
  });

  it('accepts partial grant usage (80%)', () => {
    expect(equipmentItemSchema.safeParse({ ...validData, grant_usage_pct: '80' }).success).toBe(true);
  });

  it('rejects a missing Work Package', () => {
    const r = equipmentItemSchema.safeParse({ ...validData, work_package_id: undefined });
    expect(r.success).toBe(false);
  });
});

// ─── itemizedTripSchema ───────────────────────────────────────────────────────

describe('itemizedTripSchema', () => {
  const validData = {
    name: 'India Fieldwork',
    trip_kind: 'Itemized' as const,
    destination_country_code: 'IN',
    one_way_distance_km: 5800,
    number_of_nights: 4,
    number_of_days: 5,
    domestic_transport_per_instance_eur: '340',
    number_of_instances: 4,
    work_package_ids: [1],
  };

  it('accepts a valid itemized trip', () => {
    expect(itemizedTripSchema.safeParse(validData).success).toBe(true);
  });

  it('rejects empty trip name', () => {
    const r = itemizedTripSchema.safeParse({ ...validData, name: '' });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].path).toContain('name');
  });

  it('rejects empty destination country', () => {
    const r = itemizedTripSchema.safeParse({ ...validData, destination_country_code: '' });
    expect(r.success).toBe(false);
  });

  it('accepts distance = 0 (no flight needed for nearby trips)', () => {
    expect(itemizedTripSchema.safeParse({ ...validData, one_way_distance_km: 0 }).success).toBe(true);
  });

  it('rejects negative distance', () => {
    const r = itemizedTripSchema.safeParse({ ...validData, one_way_distance_km: -1 });
    expect(r.success).toBe(false);
  });

  it('rejects number_of_nights = 0', () => {
    const r = itemizedTripSchema.safeParse({ ...validData, number_of_nights: 0 });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].message).toMatch(/at least 1/i);
  });

  it('rejects number_of_days = 0', () => {
    const r = itemizedTripSchema.safeParse({ ...validData, number_of_days: 0 });
    expect(r.success).toBe(false);
  });

  it('accepts domestic_transport = "0" (zero is allowed)', () => {
    const r = itemizedTripSchema.safeParse({ ...validData, domestic_transport_per_instance_eur: '0' });
    expect(r.success).toBe(true);
  });

  it('rejects number_of_instances = 0', () => {
    const r = itemizedTripSchema.safeParse({ ...validData, number_of_instances: 0 });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].message).toMatch(/at least 1/i);
  });

  it('rejects an empty Work Package selection', () => {
    const r = itemizedTripSchema.safeParse({ ...validData, work_package_ids: [] });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].message).toMatch(/at least one/i);
  });
});

// ─── flatTripSchema ───────────────────────────────────────────────────────────

describe('flatTripSchema', () => {
  const validData = {
    name: 'Domestic Conference',
    trip_kind: 'FlatAmount' as const,
    flat_amount_per_instance_eur: '2000',
    number_of_instances: 3,
    work_package_ids: [1],
  };

  it('accepts a valid flat-amount trip', () => {
    expect(flatTripSchema.safeParse(validData).success).toBe(true);
  });

  it('rejects zero flat amount', () => {
    const r = flatTripSchema.safeParse({ ...validData, flat_amount_per_instance_eur: '0' });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].message).toMatch(/positive/i);
  });

  it('rejects negative flat amount', () => {
    const r = flatTripSchema.safeParse({ ...validData, flat_amount_per_instance_eur: '-500' });
    expect(r.success).toBe(false);
  });
});

// ─── tripSchema (discriminated union) ────────────────────────────────────────

describe('tripSchema (discriminated union)', () => {
  it('routes Itemized trips to itemizedTripSchema', () => {
    const r = tripSchema.safeParse({
      name: 'India',
      trip_kind: 'Itemized',
      destination_country_code: 'IN',
      one_way_distance_km: 5800,
      number_of_nights: 4,
      number_of_days: 5,
      domestic_transport_per_instance_eur: '0',
      number_of_instances: 2,
      work_package_ids: [1],
    });
    expect(r.success).toBe(true);
  });

  it('routes FlatAmount trips to flatTripSchema', () => {
    const r = tripSchema.safeParse({
      name: 'Flat',
      trip_kind: 'FlatAmount',
      flat_amount_per_instance_eur: '1500',
      number_of_instances: 1,
      work_package_ids: [1],
    });
    expect(r.success).toBe(true);
  });

  it('rejects unknown trip_kind', () => {
    const r = tripSchema.safeParse({ trip_kind: 'Unknown' });
    expect(r.success).toBe(false);
  });
});

// ─── otherCostSchema ──────────────────────────────────────────────────────────

describe('otherCostSchema', () => {
  const validData = {
    name: 'MAXQDA License',
    amount_eur: '9870',
    work_package_ids: [1],
  };

  it('accepts a valid C3 item', () => {
    expect(otherCostSchema.safeParse(validData).success).toBe(true);
  });

  it('rejects empty name', () => {
    const r = otherCostSchema.safeParse({ ...validData, name: '' });
    expect(r.success).toBe(false);
  });

  it('rejects zero amount', () => {
    const r = otherCostSchema.safeParse({ ...validData, amount_eur: '0' });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].message).toMatch(/positive/i);
  });

  it('rejects negative amount', () => {
    const r = otherCostSchema.safeParse({ ...validData, amount_eur: '-100' });
    expect(r.success).toBe(false);
  });

  it('rejects an empty Work Package selection', () => {
    const r = otherCostSchema.safeParse({ ...validData, work_package_ids: [] });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].message).toMatch(/at least one/i);
  });
});

// ─── cfsItemSchema ────────────────────────────────────────────────────────────

describe('cfsItemSchema', () => {
  it('accepts a valid CFS item', () => {
    expect(cfsItemSchema.safeParse({ amount_eur: '12000' }).success).toBe(true);
  });

  it('rejects zero CFS amount', () => {
    const r = cfsItemSchema.safeParse({ amount_eur: '0' });
    expect(r.success).toBe(false);
  });
});

// ─── subcontractingSchema ─────────────────────────────────────────────────────

describe('subcontractingSchema', () => {
  it('accepts zero amount (no subcontracting)', () => {
    expect(subcontractingSchema.safeParse({ amount_eur: '0', work_package_id: 1 }).success).toBe(true);
  });

  it('accepts positive subcontracting amount', () => {
    expect(subcontractingSchema.safeParse({ amount_eur: '15000', work_package_id: 1 }).success).toBe(true);
  });

  it('rejects negative amount', () => {
    const r = subcontractingSchema.safeParse({ amount_eur: '-1', work_package_id: 1 });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].message).toMatch(/zero or a positive/i);
  });

  it('rejects a missing Work Package', () => {
    const r = subcontractingSchema.safeParse({ amount_eur: '0', work_package_id: undefined });
    expect(r.success).toBe(false);
  });
});
