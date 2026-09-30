/**
 * Tests for the Travel screen's live cost preview — specifically that the
 * "Acc. rate/night" and "Subsistence rate/day" reference rates (returned by
 * the backend's `preview_trip_cost` command) both render, not just the
 * accommodation one. The subsistence rate was already computed by the
 * backend but silently dropped by the UI until this was fixed.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import { Travel } from './Travel';
import { useProjectStore } from '../store/projectStore';
import * as commands from '../ipc/commands';
import type { TripCostPreviewDto, ProjectConfigInput } from '../types';

vi.mock('../ipc/commands', () => ({
  addTrip: vi.fn(),
  updateTrip: vi.fn(),
  deleteTrip: vi.fn(),
  previewTripCost: vi.fn(),
  getCountries: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-shell', () => ({
  open: vi.fn(),
}));

const config: ProjectConfigInput = {
  project_title: 'Test Project',
  pi_name: 'Prof. Test',
  call_reference: 'ERC-2025-CoG',
  duration_years: 1,
  work_package_count: 1,
  work_package_names: [null],
  work_package_start_months: [1],
  work_package_end_months: [12],
  default_inflation_rate_pct: '0',
  try_eur_rate: '50',
  indirect_cost_rate_pct: '25',
  rate_version_id: 'v_from_2025_05_13',
  call_opening_date: null,
  project_start_date: null,
};

describe('Travel — cost preview rate display', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useProjectStore.setState({
      projectConfig: config,
      countries: [
        { country_code: 'IN', country_name: 'India', accommodation_eur_per_night: '195', subsistence_eur_per_day: '50' },
      ],
      summary: null,
    });
  });

  it('shows both the accommodation and subsistence reference rates once the preview resolves', async () => {
    const previewResult: TripCostPreviewDto = {
      flight_cost_per_instance: '857',
      accommodation_cost_per_instance: '780',
      subsistence_cost_per_instance: '250',
      domestic_transport_per_instance: '340',
      per_instance_total_eur: '2227',
      total_trip_cost_eur: '8908',
      flight_band_label: '4,501–6,000 km → €857',
      no_flight_applicable: false,
      accommodation_rate_eur: '195',
      subsistence_rate_eur: '50',
    };
    vi.mocked(commands.previewTripCost).mockResolvedValue(previewResult);

    render(<Travel onNext={vi.fn()} onBack={vi.fn()} />);

    fireEvent.click(screen.getByText('+ Add Trip'));

    fireEvent.change(screen.getByLabelText(/Trip Name/), { target: { value: 'Field work India' } });
    fireEvent.change(screen.getByLabelText(/Destination Country/), { target: { value: 'IN' } });
    fireEvent.change(screen.getByLabelText(/One-Way Distance/), { target: { value: '5800' } });
    fireEvent.change(screen.getByLabelText(/Nights/), { target: { value: '4' } });
    fireEvent.change(screen.getByLabelText(/Days/), { target: { value: '5' } });

    await waitFor(() => expect(commands.previewTripCost).toHaveBeenCalled(), { timeout: 2000 });

    expect(await screen.findByText('Acc. rate/night')).toBeTruthy();
    expect(await screen.findByText('Subsistence rate/day')).toBeTruthy();

    const accRow = screen.getByText('Acc. rate/night').closest('.preview-row, div, li') ?? document.body;
    expect(accRow.textContent).toContain('195.00');
    const subRow = screen.getByText('Subsistence rate/day').closest('.preview-row, div, li') ?? document.body;
    expect(subRow.textContent).toContain('50.00');
  });

  it('omits both rate rows while there is no preview result yet', () => {
    render(<Travel onNext={vi.fn()} onBack={vi.fn()} />);
    fireEvent.click(screen.getByText('+ Add Trip'));

    expect(screen.queryByText('Acc. rate/night')).toBeNull();
    expect(screen.queryByText('Subsistence rate/day')).toBeNull();
  });
});
