/**
 * Tests for the Excel export engine.
 *
 * jsdom doesn't implement Blob/URL.createObjectURL/anchor clicks, so those are
 * mocked here; the mocked Blob capture is fed back into ExcelJS to assert on
 * the actual generated workbook contents (sheet names, rows, values).
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import ExcelJS from 'exceljs';
import { exportToExcel } from './excelExporter';
import type { BudgetSummaryDto, ProjectConfigInput } from '../types';

function makeZeroByYear(years = 1) {
  return Array.from({ length: years }, (_, i) => ({ year: i + 1, amount_eur: '0' }));
}

function makeSummary(overrides: Partial<BudgetSummaryDto> = {}): BudgetSummaryDto {
  return {
    category_a_by_year: makeZeroByYear(),
    category_a_total: '0',
    category_b_total: '0',
    category_c1_by_year: makeZeroByYear(),
    category_c1_total: '0',
    category_c2_total: '0',
    category_c3_by_year: makeZeroByYear(),
    category_c3_total: '0',
    indirect_base_total: '0',
    category_e_by_year: makeZeroByYear(),
    category_e_total: '0',
    total_direct_costs: '0',
    total_eligible_costs: '0',
    requested_eu_contribution: '0',
    cfs_status: 'NOT_REQUIRED',
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

const config: ProjectConfigInput = {
  project_title: 'Test Project',
  pi_name: 'Prof. Test',
  call_reference: 'ERC-2025-CoG',
  duration_years: 1,
  work_package_count: 1,
  work_package_names: [null],
  work_package_start_years: [1],
  work_package_end_years: [1],
  default_inflation_rate_pct: '0',
  try_eur_rate: '50',
  indirect_cost_rate_pct: '25',
  rate_version_id: 'v_from_2025_05_13',
  call_opening_date: null,
};

describe('exportToExcel', () => {
  let capturedBuffer: ArrayBuffer | null = null;

  beforeEach(() => {
    capturedBuffer = null;
    vi.stubGlobal('Blob', class {
      parts: BlobPart[];
      constructor(parts: BlobPart[]) {
        this.parts = parts;
        // ExcelJS writeBuffer() returns a Buffer/ArrayBuffer; capture it directly.
        capturedBuffer = parts[0] as ArrayBuffer;
      }
    });
    vi.stubGlobal('URL', { createObjectURL: vi.fn(() => 'blob:mock'), revokeObjectURL: vi.fn() });
    const realCreateElement = document.createElement.bind(document);
    vi.spyOn(document, 'createElement').mockImplementation((tag: string) =>
      tag === 'a'
        ? ({ click: vi.fn(), href: '', download: '' } as unknown as HTMLElement)
        : realCreateElement(tag)
    );
  });

  it('includes an Other Direct Costs sheet with the item rows when items exist', async () => {
    const summary = makeSummary({
      category_c3_total: '14870',
      other_cost_detail: [
        { id: '1', name: 'MAXQDA License', amount_eur: '9870', project_year: 1, is_cfs_item: false, notes: null, work_package_id: null },
        { id: '2', name: 'CFS Fee', amount_eur: '5000', project_year: 1, is_cfs_item: true, notes: 'Auto-added', work_package_id: null },
      ],
    });

    await exportToExcel(summary, config);

    expect(capturedBuffer).not.toBeNull();
    const wb = new ExcelJS.Workbook();
    await wb.xlsx.load(capturedBuffer as ArrayBuffer);

    const sheet = wb.getWorksheet('Other Direct Costs');
    expect(sheet).toBeDefined();
    expect(sheet!.getRow(1).values).toEqual([, 'Item', 'Year', 'Amount (€)', 'CFS?', 'Notes']);
    expect(sheet!.getRow(2).values).toEqual([, 'MAXQDA License', 'Year 1', 9870, 'No', '']);
    expect(sheet!.getRow(3).values).toEqual([, 'CFS Fee', 'Year 1', 5000, 'Yes', 'Auto-added']);
  });

  it('omits the Other Direct Costs sheet when there are no items', async () => {
    const summary = makeSummary();
    await exportToExcel(summary, config);

    const wb = new ExcelJS.Workbook();
    await wb.xlsx.load(capturedBuffer as ArrayBuffer);
    expect(wb.getWorksheet('Other Direct Costs')).toBeUndefined();
  });
});
