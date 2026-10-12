/**
 * The Work Packages step must explain itself: which months no WP covers (live,
 * before submitting) and whatever validation message the backend rejects with.
 * Previously field-level backend errors had no place on this screen, so a
 * rejected save looked like "Next does nothing".
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { WorkPackages } from './WorkPackages';
import { ProjectSetup } from './ProjectSetup';
import { useProjectStore } from '../store/projectStore';
import * as commands from '../ipc/commands';
import type { ProjectConfigInput } from '../types';

vi.mock('../ipc/commands', () => ({ updateProjectConfig: vi.fn() }));

// recharts' ResponsiveContainer (inside the Gantt chart) needs this in jsdom.
(globalThis as unknown as { ResizeObserver: unknown }).ResizeObserver = class {
  observe() {}
  unobserve() {}
  disconnect() {}
};

const config = (overrides: Partial<ProjectConfigInput> = {}): ProjectConfigInput => ({
  project_title: 'T', pi_name: 'P', call_reference: 'C', duration_years: 5,
  work_package_count: 3, work_package_names: [null, null, null],
  work_package_start_months: [1, 1, 1], work_package_end_months: [60, 60, 60],
  default_inflation_rate_pct: '10', try_eur_rate: '50', indirect_cost_rate_pct: '25',
  rate_version_id: 'v', call_opening_date: null, project_start_date: null, funding_model: 'LumpSum',
  ...overrides,
});

beforeEach(() => {
  vi.clearAllMocks();
  useProjectStore.setState({ summary: null, projectConfig: config() });
});

describe('Work Packages step', () => {
  it('shows no warning while the Work Packages cover the whole project', () => {
    render(<WorkPackages onNext={vi.fn()} onBack={vi.fn()} />);
    expect(screen.queryByText(/not covered by any Work Package/)).toBeNull();
  });

  it('names the uncovered months as soon as the ranges leave a gap', () => {
    render(<WorkPackages onNext={vi.fn()} onBack={vi.fn()} />);
    // WP2 and WP3 start out spanning 1-60; pull both back to 1-24 and 1-12.
    fireEvent.change(document.getElementById('wp-1-end')!, { target: { value: '24' } });
    fireEvent.change(document.getElementById('wp-2-end')!, { target: { value: '12' } });
    fireEvent.change(document.getElementById('wp-0-end')!, { target: { value: '12' } });

    expect(screen.getByText(/Months 25–60 not covered by any Work Package/)).toBeTruthy();
    expect(screen.getByText(/whole 60-month project/)).toBeTruthy();
  });

  it('shows the backend validation message instead of silently doing nothing', async () => {
    vi.mocked(commands.updateProjectConfig).mockRejectedValue({
      kind: 'Validation',
      detail: [{
        field: 'work_package_start_months',
        code: 'WP_COVERAGE_GAP',
        message: 'Work Packages must collectively cover the entire 60-month project duration. Month 25 is not covered by any Work Package.',
      }],
    });
    const onNext = vi.fn();
    render(<WorkPackages onNext={onNext} onBack={vi.fn()} />);

    fireEvent.click(screen.getByText(/Next: Personnel/));

    expect(await screen.findByText(/Month 25 is not covered by any Work Package/)).toBeTruthy();
    expect(onNext).not.toHaveBeenCalled();
  });
});

describe('Project Setup — switching back from Actual Cost', () => {
  it('offers a real Work Package count again instead of the collapsed single WP', () => {
    useProjectStore.setState({
      projectConfig: config({
        funding_model: 'ActualCost', work_package_count: 1, work_package_names: [null],
        work_package_start_months: [1], work_package_end_months: [60],
      }),
    });
    render(<ProjectSetup onNext={vi.fn()} />);

    fireEvent.click(screen.getByRole('radio', { name: 'Lumpsum Funding' }));

    expect((screen.getByLabelText(/^Work Packages/) as HTMLInputElement).value).toBe('3');
  });

  it('leaves a count the user already chose alone', () => {
    useProjectStore.setState({ projectConfig: config({ work_package_count: 4, work_package_names: [null, null, null, null], work_package_start_months: [1, 1, 1, 1], work_package_end_months: [60, 60, 60, 60] }) });
    render(<ProjectSetup onNext={vi.fn()} />);

    fireEvent.click(screen.getByRole('radio', { name: 'Actual Cost' }));
    fireEvent.click(screen.getByRole('radio', { name: 'Lumpsum Funding' }));

    expect((screen.getByLabelText(/^Work Packages/) as HTMLInputElement).value).toBe('4');
  });
});
