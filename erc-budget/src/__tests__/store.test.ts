/**
 * Unit tests for the Zustand project store (store/projectStore.ts).
 *
 * Tests cover initial state, mutations, and the reset() action.
 * Selector hooks (usePersonnelRoles, etc.) are thin Zustand subscriptions —
 * the underlying state is verified via getState() to avoid React render cycles.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import type {
  BudgetSummaryDto,
  ProjectConfigInput,
  RateVersionSummary,
  CfsStatus,
} from '../types';
import { useProjectStore } from '../store/projectStore';

// ─── Fixtures ─────────────────────────────────────────────────────────────────

function makeWpBudgets(): BudgetSummaryDto['wp_budgets'] {
  return Array.from({ length: 3 }, (_, i) => ({
    work_package_id: i + 1,
    work_package_name: null,
    personnel_eur: '0',
    equipment_eur: '0',
    travel_eur: '0',
    other_costs_eur: '0',
    subcontracting_eur: '0',
    total_eur: '0',
  }));
}

function makeSummary(overrides: Partial<BudgetSummaryDto> = {}): BudgetSummaryDto {
  return {
    wp_budgets: makeWpBudgets(),
    category_a_total: '0',
    category_b_total: '0',
    category_c1_total: '0',
    category_c2_total: '0',
    category_c3_total: '0',
    indirect_base_total: '0',
    category_e_total: '0',
    total_direct_costs: '0',
    total_eligible_costs: '100000',
    requested_eu_contribution: '100000',
    cfs_status: 'NOT_REQUIRED' as CfsStatus,
    cfs_threshold_exceeded: false,
    cfs_warning_active: false,
    cfs_prompt_required: false,
    role_detail: [],
    equipment_detail: [],
    trip_detail: [],
    other_cost_detail: [],
    ...overrides,
  };
}

function makeConfig(): ProjectConfigInput {
  return {
    project_title: 'Test Project',
    pi_name: 'Prof. Test',
    call_reference: 'ERC-2025-CoG',
    duration_years: 5,
    work_package_count: 3,
    work_package_names: [null, null, null],
    work_package_start_months: [1, 1, 1],
    work_package_end_months: [60, 60, 60],
    default_inflation_rate_pct: '20',
    try_eur_rate: '50.62',
    indirect_cost_rate_pct: '25',
    rate_version_id: 'v_from_2025_05_13',
    call_opening_date: null,
  };
}

function makeRateVersions(): RateVersionSummary[] {
  return [
    {
      version_id: 'v_from_2025_05_13',
      version_label: 'From 2025-05-13',
      applicable_from: '2025-05-13',
    },
    {
      version_id: 'v_before_2024_07_31',
      version_label: 'Before 2024-07-31',
      applicable_from: '2020-01-01',
    },
  ];
}

// ─── Reset store between tests ────────────────────────────────────────────────

beforeEach(() => {
  useProjectStore.getState().reset();
});

// ─── Initial state ─────────────────────────────────────────────────────────────

describe('initial state', () => {
  it('starts at the welcome screen', () => {
    expect(useProjectStore.getState().screen).toBe('welcome');
  });

  it('has no summary', () => {
    expect(useProjectStore.getState().summary).toBeNull();
  });

  it('has no project path', () => {
    expect(useProjectStore.getState().projectPath).toBeNull();
  });

  it('has no project config', () => {
    expect(useProjectStore.getState().projectConfig).toBeNull();
  });

  it('has empty rate versions list', () => {
    expect(useProjectStore.getState().rateVersions).toEqual([]);
  });

  it('is not loading', () => {
    expect(useProjectStore.getState().isLoading).toBe(false);
  });

  it('has no global error', () => {
    expect(useProjectStore.getState().globalError).toBeNull();
  });

  it('is not dirty', () => {
    expect(useProjectStore.getState().isDirty).toBe(false);
  });
});

// ─── setScreen ────────────────────────────────────────────────────────────────

describe('setScreen', () => {
  it('navigates to project-setup', () => {
    useProjectStore.getState().setScreen('project-setup');
    expect(useProjectStore.getState().screen).toBe('project-setup');
  });

  it('navigates to review-export', () => {
    useProjectStore.getState().setScreen('review-export');
    expect(useProjectStore.getState().screen).toBe('review-export');
  });

  it('traverses the full wizard sequence', () => {
    const steps = [
      'project-setup', 'budget-settings', 'work-packages',
      'personnel', 'equipment', 'travel', 'other-costs', 'review-export',
    ] as const;
    for (const step of steps) {
      useProjectStore.getState().setScreen(step);
      expect(useProjectStore.getState().screen).toBe(step);
    }
  });

  it('can return to welcome screen', () => {
    useProjectStore.getState().setScreen('personnel');
    useProjectStore.getState().setScreen('welcome');
    expect(useProjectStore.getState().screen).toBe('welcome');
  });
});

// ─── setSummary ───────────────────────────────────────────────────────────────

describe('setSummary', () => {
  it('stores the summary', () => {
    const s = makeSummary({ requested_eu_contribution: '200000' });
    useProjectStore.getState().setSummary(s);
    expect(useProjectStore.getState().summary?.requested_eu_contribution).toBe('200000');
  });

  it('sets isDirty when summary is updated', () => {
    useProjectStore.getState().setSummary(makeSummary());
    expect(useProjectStore.getState().isDirty).toBe(true);
  });

  it('replaces previous summary', () => {
    useProjectStore.getState().setSummary(makeSummary({ requested_eu_contribution: '100000' }));
    useProjectStore.getState().setSummary(makeSummary({ requested_eu_contribution: '250000' }));
    expect(useProjectStore.getState().summary?.requested_eu_contribution).toBe('250000');
  });

  it('stores role_detail correctly', () => {
    const s = makeSummary({
      role_detail: [{
        id: 'uuid-1',
        role_label: 'PI',
        role_type: 'Pi',
        current_monthly_salary_try: '227900',
        inflation_rate_pct: '20',
        fte_fraction: '0.70',
        start_month: 1,
        end_month: 60,
        cost_lines: [],
        total_cost_eur: '45000',
        wp_breakdown: [],
      }],
    });
    useProjectStore.getState().setSummary(s);
    expect(useProjectStore.getState().summary?.role_detail).toHaveLength(1);
    expect(useProjectStore.getState().summary?.role_detail[0].role_label).toBe('PI');
  });

  it('stores equipment_detail with is_capped flag', () => {
    const s = makeSummary({
      equipment_detail: [{
        id: 'uuid-2',
        name: 'Laptop',
        purchase_cost_eur: '3000',
        useful_lifetime_months: 48,
        grant_usage_pct: '100',
        grant_usage_months: 55,
        work_package_id: 1,
        theoretical_eligible_eur: '2864.58',
        maximum_eligible_eur: '2500',
        is_capped: true,
        eligible_depreciation_eur: '2500',
      }],
    });
    useProjectStore.getState().setSummary(s);
    expect(useProjectStore.getState().summary?.equipment_detail[0].is_capped).toBe(true);
  });

  it('stores trip_detail', () => {
    const s = makeSummary({
      trip_detail: [{
        id: 'uuid-3',
        name: 'India Fieldwork',
        work_package_ids: [1],
        number_of_instances: 4,
        destination_country_code: 'IN',
        one_way_distance_km: 5800,
        number_of_nights: 4,
        number_of_days: 5,
        flight_cost_per_instance: '857',
        accommodation_cost_per_instance: '780',
        subsistence_cost_per_instance: '250',
        domestic_transport_per_instance: '340',
        per_instance_total_eur: '2227',
        total_trip_cost_eur: '8908',
      }],
    });
    useProjectStore.getState().setSummary(s);
    expect(useProjectStore.getState().summary?.trip_detail[0].total_trip_cost_eur).toBe('8908');
  });

  it('reflects CFS status REQUIRED_AND_UNADDRESSED', () => {
    const s = makeSummary({
      requested_eu_contribution: '500000',
      cfs_status: 'REQUIRED_AND_UNADDRESSED',
      cfs_threshold_exceeded: true,
      cfs_prompt_required: true,
    });
    useProjectStore.getState().setSummary(s);
    expect(useProjectStore.getState().summary?.cfs_status).toBe('REQUIRED_AND_UNADDRESSED');
    expect(useProjectStore.getState().summary?.cfs_threshold_exceeded).toBe(true);
    expect(useProjectStore.getState().summary?.cfs_prompt_required).toBe(true);
  });

  it('reflects CFS status REQUIRED_AND_PRESENT', () => {
    const s = makeSummary({
      cfs_status: 'REQUIRED_AND_PRESENT',
      cfs_threshold_exceeded: true,
      cfs_warning_active: false,
    });
    useProjectStore.getState().setSummary(s);
    expect(useProjectStore.getState().summary?.cfs_status).toBe('REQUIRED_AND_PRESENT');
  });
});

// ─── setProjectPath ───────────────────────────────────────────────────────────

describe('setProjectPath', () => {
  it('stores an absolute file path', () => {
    useProjectStore.getState().setProjectPath('/Users/test/project.ercbudget');
    expect(useProjectStore.getState().projectPath).toBe('/Users/test/project.ercbudget');
  });

  it('accepts null to clear the path', () => {
    useProjectStore.getState().setProjectPath('/path/to/file.ercbudget');
    useProjectStore.getState().setProjectPath(null);
    expect(useProjectStore.getState().projectPath).toBeNull();
  });

  it('is independent of screen state', () => {
    useProjectStore.getState().setScreen('personnel');
    useProjectStore.getState().setProjectPath('/path.ercbudget');
    expect(useProjectStore.getState().screen).toBe('personnel');
    expect(useProjectStore.getState().projectPath).toBe('/path.ercbudget');
  });
});

// ─── setProjectConfig ─────────────────────────────────────────────────────────

describe('setProjectConfig', () => {
  it('stores a complete project config', () => {
    const config = makeConfig();
    useProjectStore.getState().setProjectConfig(config);
    expect(useProjectStore.getState().projectConfig).toEqual(config);
  });

  it('reflects project title', () => {
    useProjectStore.getState().setProjectConfig(makeConfig());
    expect(useProjectStore.getState().projectConfig?.project_title).toBe('Test Project');
  });

  it('reflects duration_years', () => {
    useProjectStore.getState().setProjectConfig(makeConfig());
    expect(useProjectStore.getState().projectConfig?.duration_years).toBe(5);
  });
});

// ─── setRateVersions ──────────────────────────────────────────────────────────

describe('setRateVersions', () => {
  it('stores rate versions', () => {
    const versions = makeRateVersions();
    useProjectStore.getState().setRateVersions(versions);
    expect(useProjectStore.getState().rateVersions).toHaveLength(2);
  });

  it('stores correct version_id', () => {
    useProjectStore.getState().setRateVersions(makeRateVersions());
    expect(useProjectStore.getState().rateVersions[0].version_id).toBe('v_from_2025_05_13');
  });

  it('replaces existing versions', () => {
    useProjectStore.getState().setRateVersions(makeRateVersions());
    useProjectStore.getState().setRateVersions([]);
    expect(useProjectStore.getState().rateVersions).toEqual([]);
  });
});

// ─── setLoading / setGlobalError ─────────────────────────────────────────────

describe('setLoading', () => {
  it('sets isLoading to true', () => {
    useProjectStore.getState().setLoading(true);
    expect(useProjectStore.getState().isLoading).toBe(true);
  });

  it('sets isLoading back to false', () => {
    useProjectStore.getState().setLoading(true);
    useProjectStore.getState().setLoading(false);
    expect(useProjectStore.getState().isLoading).toBe(false);
  });
});

describe('setGlobalError', () => {
  it('stores a Calculation error', () => {
    useProjectStore.getState().setGlobalError({
      kind: 'Calculation',
      detail: { code: 'INVALID_FTE', message: 'FTE must be > 0' },
    });
    const err = useProjectStore.getState().globalError;
    expect(err?.kind).toBe('Calculation');
  });

  it('stores a Validation error', () => {
    useProjectStore.getState().setGlobalError({
      kind: 'Validation',
      detail: [{ field: 'role_label', code: 'REQUIRED', message: 'Required' }],
    });
    expect(useProjectStore.getState().globalError?.kind).toBe('Validation');
  });

  it('clears global error when set to null', () => {
    useProjectStore.getState().setGlobalError({ kind: 'NoProject' });
    useProjectStore.getState().setGlobalError(null);
    expect(useProjectStore.getState().globalError).toBeNull();
  });
});

// ─── reset ────────────────────────────────────────────────────────────────────

describe('reset', () => {
  it('resets all state to initial values', () => {
    // Mutate everything
    useProjectStore.getState().setScreen('review-export');
    useProjectStore.getState().setSummary(makeSummary());
    useProjectStore.getState().setProjectPath('/path.ercbudget');
    useProjectStore.getState().setProjectConfig(makeConfig());
    useProjectStore.getState().setRateVersions(makeRateVersions());
    useProjectStore.getState().setLoading(true);

    // Reset
    useProjectStore.getState().reset();

    const state = useProjectStore.getState();
    expect(state.screen).toBe('welcome');
    expect(state.summary).toBeNull();
    expect(state.projectPath).toBeNull();
    expect(state.projectConfig).toBeNull();
    expect(state.rateVersions).toEqual([]);
    expect(state.isLoading).toBe(false);
    expect(state.isDirty).toBe(false);
    expect(state.globalError).toBeNull();
  });
});

// ─── Edge cases ───────────────────────────────────────────────────────────────

describe('edge cases', () => {
  it('multiple independent mutations do not interfere', () => {
    useProjectStore.getState().setScreen('personnel');
    useProjectStore.getState().setProjectPath('/path.ercbudget');
    useProjectStore.getState().setSummary(makeSummary({ requested_eu_contribution: '350000' }));
    useProjectStore.getState().setLoading(false);

    const state = useProjectStore.getState();
    expect(state.screen).toBe('personnel');
    expect(state.projectPath).toBe('/path.ercbudget');
    expect(state.summary?.requested_eu_contribution).toBe('350000');
    expect(state.isLoading).toBe(false);
  });

  it('summary with all-zero amounts is stored correctly', () => {
    const s = makeSummary({ requested_eu_contribution: '0', total_eligible_costs: '0' });
    useProjectStore.getState().setSummary(s);
    expect(useProjectStore.getState().summary?.requested_eu_contribution).toBe('0');
  });
});
