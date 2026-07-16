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

function makeWpBudgets(): BudgetSummaryDto['wp_budgets'] {
  return [{
    work_package_id: 1,
    work_package_name: null,
    personnel_eur: '0',
    equipment_eur: '0',
    travel_eur: '0',
    other_costs_eur: '0',
    subcontracting_eur: '0',
    total_eur: '0',
  }];
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
  work_package_start_months: [1],
  work_package_end_months: [12],
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
    // jsdom doesn't implement canvas 2D context; stub it to return null quietly
    // instead of letting jsdom log its noisy "not implemented" error.
    vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue(null);
  });

  it('includes an Other Direct Costs sheet with the item rows when items exist', async () => {
    const summary = makeSummary({
      category_c3_total: '14870',
      other_cost_detail: [
        { id: '1', name: 'MAXQDA License', amount_eur: '9870', is_cfs_item: false, notes: null, work_package_ids: [1] },
        { id: '2', name: 'CFS Fee', amount_eur: '5000', is_cfs_item: true, notes: 'Auto-added', work_package_ids: [] },
      ],
    });

    await exportToExcel(summary, config);

    expect(capturedBuffer).not.toBeNull();
    const wb = new ExcelJS.Workbook();
    await wb.xlsx.load(capturedBuffer as ArrayBuffer);

    const sheet = wb.getWorksheet('Other Direct Costs');
    expect(sheet).toBeDefined();
    expect(sheet!.getRow(1).values).toEqual([, 'Item', 'Work Package(s)', 'Amount (€)', 'CFS?', 'Notes']);
    expect(sheet!.getRow(2).values).toEqual([, 'MAXQDA License', 'WP1', 9870, 'No', '']);
    expect(sheet!.getRow(3).values).toEqual([, 'CFS Fee', '', 5000, 'Yes', 'Auto-added']);
  });

  it('omits the Other Direct Costs sheet when there are no items', async () => {
    const summary = makeSummary();
    await exportToExcel(summary, config);

    const wb = new ExcelJS.Workbook();
    await wb.xlsx.load(capturedBuffer as ArrayBuffer);
    expect(wb.getWorksheet('Other Direct Costs')).toBeUndefined();
  });

  it('builds a formula-driven Personnel sheet and links it from Budget Summary', async () => {
    const twoYearConfig: ProjectConfigInput = { ...config, duration_years: 2 };
    const summary = makeSummary({
      category_a_total: '45000',
      role_detail: [{
        id: '1',
        role_label: 'PI',
        role_type: 'Pi',
        current_monthly_salary_try: '150000',
        inflation_rate_pct: '20',
        fte_fraction: '0.5',
        start_month: 1,
        end_month: 24,
        cost_lines: [],
        total_cost_eur: '45000',
        wp_breakdown: [],
      }],
    });

    await exportToExcel(summary, twoYearConfig);

    const wb = new ExcelJS.Workbook();
    await wb.xlsx.load(capturedBuffer as ArrayBuffer);

    const persSheet = wb.getWorksheet('Personnel');
    expect(persSheet).toBeDefined();
    expect(persSheet!.getRow(1).values).toEqual([
      , 'Role', 'Type', 'Current Salary (TRY)', 'Annual Increase (%)', 'FTE',
      'Start Month', 'End Month', 'Base Monthly (€)', 'Year 1 (€)', 'Year 2 (€)', 'Total (€)',
    ]);
    const roleRow = persSheet!.getRow(2);
    expect(roleRow.getCell(3).value).toBe(150000); // Current Salary (TRY)
    expect((roleRow.getCell(8).value as { formula: string }).formula).toBe(`C2/'Budget Summary'!$B$4`);
    expect((roleRow.getCell(9).value as { formula: string }).formula).toBe(
      'H2*(1+D2/100)^1*MAX(0,MIN(G2,1*12)-MAX(F2,(1-1)*12+1)+1)*E2',
    );
    expect((roleRow.getCell(11).value as { formula: string }).formula).toBe('SUM(I2:J2)');

    const summarySheet = wb.getWorksheet('Budget Summary');
    const rows = summarySheet!.getRows(1, summarySheet!.rowCount) ?? [];
    const aRow = rows.find((r) => r.getCell(1).value === 'A  Personnel');
    expect((aRow!.getCell(3).value as { formula: string }).formula).toBe('SUM(Personnel!K2:K2)');
  });

  it('embeds a Gantt chart image sheet when canvas rendering succeeds', async () => {
    // jsdom doesn't implement canvas 2D context, so the exporter's try/catch
    // guard should skip the sheet rather than throw.
    const summary = makeSummary();
    await exportToExcel(summary, config);
    const wb = new ExcelJS.Workbook();
    await wb.xlsx.load(capturedBuffer as ArrayBuffer);
    expect(wb.getWorksheet('Gantt Chart')).toBeUndefined();
  });
});
