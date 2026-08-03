import { describe, it, expect, beforeEach } from 'vitest';
import { useExecutionStore } from './executionStore';
import type { ExecutionProjectSummaryDto } from '../types';

const summary: ExecutionProjectSummaryDto = {
  project_info: {
    project_title: 'Test Project',
    pi_name: 'PI',
    call_reference: 'ERC-2025-CoG',
    duration_years: 5,
    work_package_count: 3,
  },
  planned: {
    wp_budgets: [],
    category_a_total: '100',
    category_b_total: '0',
    category_c1_total: '0',
    category_c2_total: '0',
    category_c3_total: '0',
    indirect_base_total: '100',
    category_e_total: '25',
    total_direct_costs: '100',
    total_eligible_costs: '125',
    requested_eu_contribution: '125',
    cfs_status: 'NOT_REQUIRED',
    cfs_threshold_exceeded: false,
    cfs_warning_active: false,
    cfs_prompt_required: false,
    role_detail: [],
    equipment_detail: [],
    trip_detail: [],
    other_cost_detail: [],
  },
  current_project_month: 1,
  personnel_roles: [],
  planned_trips: [],
  planned_equipment: [],
  planned_other_costs: [],
  persons: [],
  person_months: [],
  work_packages: [],
  milestones: [],
  amendments: [],
  actuals: {
    a_actual: '0',
    b_actual: '0',
    c1_actual: '0',
    c2_actual: '0',
    c3_actual: '0',
    e_actual: '0',
    total_direct_actual: '0',
    total_eligible_actual: '0',
    requested_eu_contribution_actual: '0',
    cfs_status_actual: 'NOT_REQUIRED',
    category_a_overrun: false,
    category_b_overrun: false,
    category_c1_overrun: false,
    category_c2_overrun: false,
    category_c3_overrun: false,
  },
  trip_executions: [],
  equipment_procurements: [],
  actual_cost_entries: [],
  subcontracting_lines: [],
  deliverables: [],
  reporting_periods: [],
  reporting_period_coverage: {
    gaps_detected: false,
    final_period_covers_project_end: true,
  },
  risks: [],
  issues: [],
};

describe('useExecutionStore', () => {
  beforeEach(() => {
    useExecutionStore.setState({
      summary: null,
      projectPath: null,
      activeScreen: 'welcome',
      isLoading: false,
      isDirty: false,
      error: null,
    });
  });

  it('starts on the welcome screen with no summary', () => {
    const state = useExecutionStore.getState();
    expect(state.activeScreen).toBe('welcome');
    expect(state.summary).toBeNull();
  });

  it('setSummary stores the summary/path and navigates to dashboard', () => {
    useExecutionStore.getState().setSummary(summary, '/tmp/project.ercbudget');
    const state = useExecutionStore.getState();
    expect(state.summary).toEqual(summary);
    expect(state.projectPath).toBe('/tmp/project.ercbudget');
    expect(state.activeScreen).toBe('dashboard');
    expect(state.error).toBeNull();
  });

  it('setError records the error without changing the screen', () => {
    useExecutionStore.getState().setError({ kind: 'NoProject' });
    const state = useExecutionStore.getState();
    expect(state.error).toEqual({ kind: 'NoProject' });
    expect(state.activeScreen).toBe('welcome');
  });

  it('updateSummary replaces the summary without changing the active screen', () => {
    useExecutionStore.setState({ activeScreen: 'personnel' });
    useExecutionStore.getState().updateSummary(summary);
    const state = useExecutionStore.getState();
    expect(state.summary).toEqual(summary);
    expect(state.activeScreen).toBe('personnel');
  });
});
