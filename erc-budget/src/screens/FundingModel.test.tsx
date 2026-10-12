/**
 * UI behaviour of the two settings that change how a project is entered:
 *
 *  - Funding model (Actual Cost vs Lumpsum): Actual Cost has no Work
 *    Packages, so the WP count, the Work Packages wizard step and every
 *    per-item WP selector disappear, and items are filed under the single
 *    implicit WP.
 *  - Salary currency (TRY vs EUR) on a personnel role.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import { ProjectSetup } from './ProjectSetup';
import { OtherCosts } from './OtherCosts';
import { Personnel } from './Personnel';
import { ProgressStepper } from '../components/ProgressStepper';
import { useProjectStore } from '../store/projectStore';
import * as commands from '../ipc/commands';
import type { BudgetSummaryDto, ProjectConfigInput } from '../types';

vi.mock('../ipc/commands', () => ({
  addOtherCost: vi.fn(),
  updateOtherCost: vi.fn(),
  deleteOtherCost: vi.fn(),
  removeCfsItem: vi.fn(),
  addPersonnelRole: vi.fn(),
  updatePersonnelRole: vi.fn(),
  deletePersonnelRole: vi.fn(),
  previewRoleCost: vi.fn(),
}));

function makeConfig(overrides: Partial<ProjectConfigInput> = {}): ProjectConfigInput {
  return {
    project_title: 'Test Project',
    pi_name: 'Prof. Test',
    call_reference: 'ERC-2025-CoG',
    duration_years: 5,
    work_package_count: 3,
    work_package_names: [null, null, null],
    work_package_start_months: [1, 1, 1],
    work_package_end_months: [60, 60, 60],
    default_inflation_rate_pct: '10',
    try_eur_rate: '50',
    indirect_cost_rate_pct: '25',
    rate_version_id: 'v_from_2025_05_13',
    call_opening_date: null,
    project_start_date: null,
    funding_model: 'LumpSum',
    ...overrides,
  };
}

const actualCostConfig = () => makeConfig({
  funding_model: 'ActualCost',
  work_package_count: 1,
  work_package_names: [null],
  work_package_start_months: [1],
  work_package_end_months: [60],
});

beforeEach(() => {
  vi.clearAllMocks();
  useProjectStore.setState({ projectConfig: null, summary: null });
});

describe('Project Setup — funding model', () => {
  it('asks for the funding model, defaulting to Lumpsum with the Work Packages field visible', () => {
    render(<ProjectSetup onNext={vi.fn()} />);

    expect(screen.getByRole('radio', { name: 'Lumpsum Funding' }).getAttribute('aria-checked')).toBe('true');
    expect(screen.getByRole('radio', { name: 'Actual Cost' }).getAttribute('aria-checked')).toBe('false');
    expect(screen.getByLabelText(/^Work Packages/)).toBeTruthy();
  });

  it('choosing Actual Cost hides the Work Packages field', () => {
    render(<ProjectSetup onNext={vi.fn()} />);

    fireEvent.click(screen.getByRole('radio', { name: 'Actual Cost' }));

    expect(screen.getByRole('radio', { name: 'Actual Cost' }).getAttribute('aria-checked')).toBe('true');
    expect(screen.queryByLabelText(/^Work Packages/)).toBeNull();
  });

  it('submitting an Actual Cost project stores one implicit Work Package spanning the whole duration', async () => {
    const onNext = vi.fn();
    render(<ProjectSetup onNext={onNext} />);

    fireEvent.change(screen.getByLabelText(/Project Title/), { target: { value: 'Grant' } });
    fireEvent.change(screen.getByLabelText(/Principal Investigator/), { target: { value: 'Dr. X' } });
    fireEvent.change(screen.getByLabelText(/Call Reference/), { target: { value: 'ERC-2026-CoG' } });
    fireEvent.click(screen.getByRole('radio', { name: 'Actual Cost' }));
    fireEvent.click(screen.getByText('Next: Budget Settings →'));

    await waitFor(() => expect(onNext).toHaveBeenCalled());
    expect(useProjectStore.getState().projectConfig).toMatchObject({
      funding_model: 'ActualCost',
      duration_years: 5,
      work_package_count: 1,
      work_package_names: [null],
      work_package_start_months: [1],
      work_package_end_months: [60],
    });
  });

  it('a Lumpsum project keeps the Work Package count the user chose', async () => {
    const onNext = vi.fn();
    render(<ProjectSetup onNext={onNext} />);

    fireEvent.change(screen.getByLabelText(/Project Title/), { target: { value: 'Grant' } });
    fireEvent.change(screen.getByLabelText(/Principal Investigator/), { target: { value: 'Dr. X' } });
    fireEvent.change(screen.getByLabelText(/Call Reference/), { target: { value: 'ERC-2026-CoG' } });
    fireEvent.change(screen.getByLabelText(/^Work Packages/), { target: { value: '4' } });
    fireEvent.click(screen.getByText('Next: Budget Settings →'));

    await waitFor(() => expect(onNext).toHaveBeenCalled());
    expect(useProjectStore.getState().projectConfig).toMatchObject({
      funding_model: 'LumpSum',
      work_package_count: 4,
    });
  });
});

describe('Wizard steps — funding model', () => {
  const labels = () => screen.getAllByRole('button').map((b) => b.textContent ?? '');

  it('Lumpsum shows the Work Packages step', () => {
    useProjectStore.setState({ projectConfig: makeConfig(), summary: {} as BudgetSummaryDto });
    render(<ProgressStepper onNavigate={vi.fn()} />);
    expect(labels().some((l) => l.includes('Work Packages'))).toBe(true);
  });

  it('Actual Cost skips the Work Packages step but keeps every cost step', () => {
    useProjectStore.setState({ projectConfig: actualCostConfig(), summary: {} as BudgetSummaryDto });
    render(<ProgressStepper onNavigate={vi.fn()} />);
    const all = labels();
    expect(all.some((l) => l.includes('Work Packages'))).toBe(false);
    for (const step of ['Personnel', 'Equipment', 'Travel', 'Other Costs', 'Subcontracting', 'Review & Export']) {
      expect(all.some((l) => l.includes(step))).toBe(true);
    }
  });
});

describe('Cost items — Work Package selection', () => {
  const fillAndSubmitItem = () => {
    fireEvent.click(screen.getAllByText('+ Add Item')[0]);
    fireEvent.change(screen.getByLabelText(/Item Name/), { target: { value: 'Lab consumables' } });
    fireEvent.change(screen.getByLabelText(/Amount/), { target: { value: '1500' } });
  };

  it('Lumpsum still asks which Work Package(s) an item belongs to', () => {
    useProjectStore.setState({ projectConfig: makeConfig() });
    render(<OtherCosts onNext={vi.fn()} onBack={vi.fn()} />);
    fillAndSubmitItem();
    expect(screen.getByText('Work Package(s)')).toBeTruthy();
  });

  it('Actual Cost shows no Work Package selector and files the item under the single implicit WP', async () => {
    useProjectStore.setState({ projectConfig: actualCostConfig() });
    vi.mocked(commands.addOtherCost).mockResolvedValue({} as BudgetSummaryDto);
    render(<OtherCosts onNext={vi.fn()} onBack={vi.fn()} />);

    fillAndSubmitItem();
    expect(screen.queryByText('Work Package(s)')).toBeNull();

    // Submits with no WP interaction — and passes validation, which demands
    // at least one Work Package, because the form is pre-assigned to WP1.
    fireEvent.click(screen.getByText('Add Item'));
    await waitFor(() => expect(commands.addOtherCost).toHaveBeenCalled());
    expect(commands.addOtherCost).toHaveBeenCalledWith(
      expect.objectContaining({ name: 'Lab consumables', amount_eur: '1500', work_package_ids: [1] }),
    );
  });
});

describe('Personnel — salary currency', () => {
  it('defaults to TRY and labels the salary field accordingly', () => {
    useProjectStore.setState({ projectConfig: makeConfig() });
    render(<Personnel onNext={vi.fn()} onBack={vi.fn()} />);
    fireEvent.click(screen.getByText('+ Add Role'));

    expect(screen.getByRole('radio', { name: /TRY/ }).getAttribute('aria-checked')).toBe('true');
    expect(screen.getByLabelText('Current Monthly Salary (TRY)')).toBeTruthy();
  });

  it('choosing EUR relabels the field and sends the EUR currency with the preview and the saved role', async () => {
    useProjectStore.setState({ projectConfig: makeConfig() });
    vi.mocked(commands.previewRoleCost).mockResolvedValue({
      base_monthly_eur: '4500', cost_lines: [], total_cost_eur: '0', wp_breakdown: [],
    });
    vi.mocked(commands.addPersonnelRole).mockResolvedValue({} as BudgetSummaryDto);
    render(<Personnel onNext={vi.fn()} onBack={vi.fn()} />);
    fireEvent.click(screen.getByText('+ Add Role'));

    fireEvent.click(screen.getByRole('radio', { name: /EUR/ }));
    expect(screen.getByLabelText('Current Monthly Salary (EUR)')).toBeTruthy();
    expect(screen.getByText(/no TRY→EUR conversion/)).toBeTruthy();

    fireEvent.change(screen.getByLabelText(/Role Label/), { target: { value: 'Postdoc' } });
    fireEvent.change(screen.getByLabelText('Current Monthly Salary (EUR)'), { target: { value: '4500' } });
    fireEvent.change(screen.getByLabelText(/PM \(0–1\)/), { target: { value: '1' } });

    await waitFor(() => expect(commands.previewRoleCost).toHaveBeenCalled(), { timeout: 2000 });
    expect(commands.previewRoleCost).toHaveBeenLastCalledWith(
      expect.objectContaining({ current_monthly_salary_try: '4500', salary_currency: 'EUR' }),
    );

    fireEvent.click(screen.getByText('Add Role'));
    await waitFor(() => expect(commands.addPersonnelRole).toHaveBeenCalled());
    expect(commands.addPersonnelRole).toHaveBeenCalledWith(
      expect.objectContaining({ role_label: 'Postdoc', current_monthly_salary_try: '4500', salary_currency: 'EUR' }),
    );
  });

  it('Actual Cost shows no per-Work-Package rows in the role cost preview', async () => {
    useProjectStore.setState({ projectConfig: actualCostConfig() });
    vi.mocked(commands.previewRoleCost).mockResolvedValue({
      base_monthly_eur: '1000',
      cost_lines: [],
      total_cost_eur: '12000',
      wp_breakdown: [{ work_package_id: 1, amount_eur: '12000' }],
    });
    render(<Personnel onNext={vi.fn()} onBack={vi.fn()} />);
    fireEvent.click(screen.getByText('+ Add Role'));
    fireEvent.change(screen.getByLabelText('Current Monthly Salary (TRY)'), { target: { value: '50000' } });
    fireEvent.change(screen.getByLabelText(/PM \(0–1\)/), { target: { value: '1' } });

    expect(await screen.findByText('Total cost', {}, { timeout: 2000 })).toBeTruthy();
    expect(screen.queryByText(/^→/)).toBeNull();
  });
});
