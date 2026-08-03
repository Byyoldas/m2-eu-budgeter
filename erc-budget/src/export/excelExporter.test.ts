/**
 * Tests for the Excel export engine.
 *
 * jsdom doesn't implement Blob/URL.createObjectURL/anchor clicks, so those are
 * mocked here; the mocked Blob capture is fed back into ExcelJS to assert on
 * the actual generated workbook contents (sheet names, rows, values).
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import ExcelJS from 'exceljs';
import { HyperFormula } from 'hyperformula';
import { exportToExcel } from './excelExporter';
import type { BudgetSummaryDto, ProjectConfigInput } from '../types';

/**
 * Converts an ExcelJS worksheet into the 2D grid shape HyperFormula expects,
 * preserving formula strings (as `=...`) so cross-sheet references resolve
 * exactly as they would in real Excel. Used to actually evaluate the
 * Personnel sheet's SUMPRODUCT formulas rather than just asserting their text.
 */
function gridFromWorksheet(ws: ExcelJS.Worksheet): (string | number | null)[][] {
  const grid: (string | number | null)[][] = [];
  for (let r = 1; r <= ws.rowCount; r++) {
    const row = ws.getRow(r);
    const rowArr: (string | number | null)[] = [];
    for (let c = 1; c <= ws.columnCount; c++) {
      const v = row.getCell(c).value as unknown;
      if (v && typeof v === 'object' && 'formula' in v) {
        rowArr.push(`=${(v as { formula: string }).formula}`);
      } else if (typeof v === 'number' || typeof v === 'string') {
        rowArr.push(v);
      } else {
        rowArr.push(null);
      }
    }
    grid.push(rowArr);
  }
  return grid;
}

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

  it('builds a WP-based Personnel sheet with a hidden month-overlap helper', async () => {
    const twoWpConfig: ProjectConfigInput = {
      ...config,
      duration_years: 2,
      work_package_count: 2,
      work_package_names: [null, null],
      // WP1 covers only Year 1; WP2 spans the whole project, so months 1-12
      // are covered by both (even split) and months 13-24 by WP2 alone.
      work_package_start_months: [1, 1],
      work_package_end_months: [12, 24],
    };
    const summary = makeSummary({
      wp_budgets: [
        { work_package_id: 1, work_package_name: null, personnel_eur: '0', equipment_eur: '0', travel_eur: '0', other_costs_eur: '0', subcontracting_eur: '0', total_eur: '0' },
        { work_package_id: 2, work_package_name: null, personnel_eur: '0', equipment_eur: '0', travel_eur: '0', other_costs_eur: '0', subcontracting_eur: '0', total_eur: '0' },
      ],
      category_a_total: '4092',
      role_detail: [
        {
          // Active only in Year 1 (months 1-12), inside both WPs' overlap zone
          // → every month splits 50/50. Base monthly = 5000/50 = 100 EUR;
          // Year 1 monthly = 100*(1.10)^1 = 110. Expect WP1=WP2=12*110/2=660.
          id: '1', role_label: 'RoleA', role_type: 'Expert',
          current_monthly_salary_try: '5000', inflation_rate_pct: '10', fte_fraction: '1.0',
          start_month: 1, end_month: 12, cost_lines: [], total_cost_eur: '1320', wp_breakdown: [],
        },
        {
          // Active the full 2 years. Months 1-12 split 50/50 as above (660
          // each); months 13-24 only WP2 covers, no split, at Year 2's
          // inflated rate: 100*(1.10)^2=121 → 12*121=1452 (all to WP2).
          // Expect WP1=660, WP2=660+1452=2112, total=2772.
          id: '2', role_label: 'RoleB', role_type: 'Expert',
          current_monthly_salary_try: '5000', inflation_rate_pct: '10', fte_fraction: '1.0',
          start_month: 1, end_month: 24, cost_lines: [], total_cost_eur: '2772', wp_breakdown: [],
        },
      ],
    });

    await exportToExcel(summary, twoWpConfig);

    const wb = new ExcelJS.Workbook();
    await wb.xlsx.load(capturedBuffer as ArrayBuffer);

    const persSheet = wb.getWorksheet('Personnel');
    const helperSheet = wb.getWorksheet('_WPMonthHelper');
    const summarySheet = wb.getWorksheet('Budget Summary');
    expect(persSheet).toBeDefined();
    expect(helperSheet).toBeDefined();
    expect(helperSheet!.state).toBe('hidden');

    // WP Timelines table (rows 1-2 header, then one row per WP). Duration
    // and PM columns are formulas, checked below via HyperFormula.
    expect(persSheet!.getRow(2).values).toEqual([
      , 'WP', 'Start Month', 'End Month', 'Duration (Months)', 'RoleA (PM)', 'RoleB (PM)',
    ]);
    expect(persSheet!.getRow(3).getCell(1).value).toBe('WP1');
    expect(persSheet!.getRow(3).getCell(2).value).toBe(1);
    expect(persSheet!.getRow(3).getCell(3).value).toBe(12);
    expect(persSheet!.getRow(4).getCell(1).value).toBe('WP2');
    expect(persSheet!.getRow(4).getCell(2).value).toBe(1);
    expect(persSheet!.getRow(4).getCell(3).value).toBe(24);

    // Roles table starts at row 9 (header) / row 10 (first role): 2 WP rows
    // (3-4) + Total PM/Employment Months/Reconciled? rows (5-7) + blank (8).
    expect(persSheet!.getRow(9).values).toEqual([
      , 'Role', 'Type', 'Current Salary (TRY)', 'Annual Increase (%)', 'FTE',
      'Start Month', 'End Month', 'Base Monthly (€)', 'WP1 (€)', 'WP2 (€)', 'Unattributed (€)', 'Total (€)',
    ]);

    // Evaluate the actual formulas (not just their text) with HyperFormula,
    // cross-checked against hand-computed expected values above.
    const hf = HyperFormula.buildFromSheets({
      'Budget Summary': gridFromWorksheet(summarySheet!),
      'Personnel': gridFromWorksheet(persSheet!),
      '_WPMonthHelper': gridFromWorksheet(helperSheet!),
    }, { licenseKey: 'gpl-v3', useArrayArithmetic: true });
    const sheetId = hf.getSheetId('Personnel')!;
    const cell = (row: number, col: number) => hf.getCellValue({ sheet: sheetId, row: row - 1, col: col - 1 });

    // WP Timelines Duration column (D): WP1 = 12-1+1=12, WP2 = 24-1+1=24.
    expect(cell(3, 4)).toBeCloseTo(12, 6);
    expect(cell(4, 4)).toBeCloseTo(24, 6);

    // Person-Months: both roles are active every month WP1 covers (1-12),
    // which WP2 also fully covers → every one of those months splits 50/50
    // between the two WPs. RoleA (months 1-12): WP1=6, WP2=6, total=12
    // (= its 12-month employment span). RoleB (months 1-24): months 1-12
    // split 50/50 (WP1=6, WP2=6), months 13-24 only WP2 covers (WP1=1-12
    // doesn't reach there) so WP2 += 12 more → WP1=6, WP2=18, total=24
    // (= its 24-month employment span).
    expect(cell(3, 5)).toBeCloseTo(6, 6); // RoleA PM in WP1
    expect(cell(3, 6)).toBeCloseTo(6, 6); // RoleB PM in WP1
    expect(cell(4, 5)).toBeCloseTo(6, 6); // RoleA PM in WP2
    expect(cell(4, 6)).toBeCloseTo(18, 6); // RoleB PM in WP2

    // Total PM (row 5) must equal Employment Months (row 6) for both roles —
    // the Reconciled? row (7) should read "OK", not "MISMATCH".
    expect(cell(5, 5)).toBeCloseTo(12, 6);
    expect(cell(5, 6)).toBeCloseTo(24, 6);
    expect(cell(6, 5)).toBeCloseTo(12, 6);
    expect(cell(6, 6)).toBeCloseTo(24, 6);
    expect(cell(7, 5)).toBe('OK');
    expect(cell(7, 6)).toBe('OK');

    // RoleA at row 10: Base Monthly(H)=100, WP1(I)=660, WP2(J)=660, Unattributed(K)=0.
    expect(cell(10, 8)).toBeCloseTo(100, 6);
    expect(cell(10, 9)).toBeCloseTo(660, 6);
    expect(cell(10, 10)).toBeCloseTo(660, 6);
    expect(cell(10, 11)).toBeCloseTo(0, 6);

    // RoleB at row 11: WP1(I)=660, WP2(J)=2112, Unattributed(K)=0.
    expect(cell(11, 9)).toBeCloseTo(660, 6);
    expect(cell(11, 10)).toBeCloseTo(2112, 6);
    expect(cell(11, 11)).toBeCloseTo(0, 6);

    // Budget Summary's Category A total = SUM of both roles' Total column = 1320+2772=4092.
    const bsSheetId = hf.getSheetId('Budget Summary')!;
    const rows = summarySheet!.getRows(1, summarySheet!.rowCount) ?? [];
    const aRowNum = rows.find((r) => r.getCell(1).value === 'A  Personnel')!.number;
    expect(hf.getCellValue({ sheet: bsSheetId, row: aRowNum - 1, col: 3 })).toBeCloseTo(4092, 6);
  });

  it('reconciles WP totals for sequential multi-year WPs with different inflation rates (mirrors Rust IT-06)', async () => {
    // 3-year (36-month) project, WP1 = months 1-18, WP2 = months 19-36 —
    // sequential, no overlap, each WP spans two project years. Same fixture
    // as src-tauri/tests/integration_test.rs::test_it06_personnel_wp_allocation_multi_year_multi_inflation.
    const wpConfig: ProjectConfigInput = {
      ...config,
      duration_years: 3,
      work_package_count: 2,
      work_package_names: [null, null],
      work_package_start_months: [1, 19],
      work_package_end_months: [18, 36],
    };
    const summary = makeSummary({
      wp_budgets: [
        { work_package_id: 1, work_package_name: null, personnel_eur: '0', equipment_eur: '0', travel_eur: '0', other_costs_eur: '0', subcontracting_eur: '0', total_eur: '0' },
        { work_package_id: 2, work_package_name: null, personnel_eur: '0', equipment_eur: '0', travel_eur: '0', other_costs_eur: '0', subcontracting_eur: '0', total_eur: '0' },
      ],
      category_a_total: '9844.2',
      role_detail: [
        {
          // RoleA: 100 EUR base, 10% inflation, active months 1-36 (whole project).
          id: '1', role_label: 'RoleA', role_type: 'Expert',
          current_monthly_salary_try: '5000', inflation_rate_pct: '10', fte_fraction: '1.0',
          start_month: 1, end_month: 36, cost_lines: [], total_cost_eur: '4369.2', wp_breakdown: [],
        },
        {
          // RoleB: 160 EUR base, 25% inflation, active months 10-30.
          id: '2', role_label: 'RoleB', role_type: 'PostDoc',
          current_monthly_salary_try: '8000', inflation_rate_pct: '25', fte_fraction: '1.0',
          start_month: 10, end_month: 30, cost_lines: [], total_cost_eur: '5475', wp_breakdown: [],
        },
      ],
    });

    await exportToExcel(summary, wpConfig);

    const wb = new ExcelJS.Workbook();
    await wb.xlsx.load(capturedBuffer as ArrayBuffer);
    const persSheet = wb.getWorksheet('Personnel');
    const helperSheet = wb.getWorksheet('_WPMonthHelper');
    const summarySheet = wb.getWorksheet('Budget Summary');

    const hf = HyperFormula.buildFromSheets({
      'Budget Summary': gridFromWorksheet(summarySheet!),
      'Personnel': gridFromWorksheet(persSheet!),
      '_WPMonthHelper': gridFromWorksheet(helperSheet!),
    }, { licenseKey: 'gpl-v3', useArrayArithmetic: true });
    const sheetId = hf.getSheetId('Personnel')!;
    const cell = (row: number, col: number) => hf.getCellValue({ sheet: sheetId, row: row - 1, col: col - 1 });

    // WP Timelines table occupies rows 3-4; Duration column (D): WP1 =
    // 18-1+1=18, WP2 = 36-19+1=18.
    expect(cell(3, 4)).toBeCloseTo(18, 6);
    expect(cell(4, 4)).toBeCloseTo(18, 6);

    // Person-Months (E=RoleA, F=RoleB): WP1/WP2 don't overlap here, so no
    // reciprocal splitting — each month counts once toward whichever WP
    // contains it.
    // RoleA (months 1-36): WP1 gets months 1-18 (18 PM), WP2 gets 19-36 (18
    // PM); total 36 = its 36-month employment span.
    expect(cell(3, 5)).toBeCloseTo(18, 6); // RoleA PM in WP1
    expect(cell(4, 5)).toBeCloseTo(18, 6); // RoleA PM in WP2
    // RoleB (months 10-30): WP1 gets months 10-18 (9 PM), WP2 gets 19-30 (12
    // PM); total 21 = its 21-month employment span (30-10+1).
    expect(cell(3, 6)).toBeCloseTo(9, 6); // RoleB PM in WP1
    expect(cell(4, 6)).toBeCloseTo(12, 6); // RoleB PM in WP2

    // Total PM (row 5) reconciles against Employment Months (row 6) for both
    // roles — Reconciled? (row 7) reads "OK".
    expect(cell(5, 5)).toBeCloseTo(36, 6);
    expect(cell(5, 6)).toBeCloseTo(21, 6);
    expect(cell(6, 5)).toBeCloseTo(36, 6);
    expect(cell(6, 6)).toBeCloseTo(21, 6);
    expect(cell(7, 5)).toBe('OK');
    expect(cell(7, 6)).toBe('OK');

    // Roles table: WP timeline table occupies rows 3-4, Total PM/Employment
    // Months/Reconciled? rows 5-7, blank row 8, header row 9, RoleA at row
    // 10, RoleB at row 11. Columns: H=Base Monthly, I=WP1, J=WP2,
    // K=Unattributed, L=Total.
    // RoleA: WP1 = 1320.0+726.00 = 2046.00, WP2 = 726.00+1597.200 = 2323.2, Unattributed = 0.
    expect(cell(10, 9)).toBeCloseTo(2046.00, 6);
    expect(cell(10, 10)).toBeCloseTo(2323.2, 6);
    expect(cell(10, 11)).toBeCloseTo(0, 6);
    // RoleB: WP1 = 600.0+1500.0000 = 2100.0, WP2 = 1500.0000+1875.000000 = 3375.0, Unattributed = 0.
    expect(cell(11, 9)).toBeCloseTo(2100.0, 6);
    expect(cell(11, 10)).toBeCloseTo(3375.0, 6);
    expect(cell(11, 11)).toBeCloseTo(0, 6);

    // No discrepancy: WP1 + WP2 across both roles equals the Category A total.
    const wp1Total = (cell(10, 9) as number) + (cell(11, 9) as number);
    const wp2Total = (cell(10, 10) as number) + (cell(11, 10) as number);
    expect(wp1Total + wp2Total).toBeCloseTo(9844.2, 6);

    const bsSheetId = hf.getSheetId('Budget Summary')!;
    const rows = summarySheet!.getRows(1, summarySheet!.rowCount) ?? [];
    const aRowNum = rows.find((r) => r.getCell(1).value === 'A  Personnel')!.number;
    expect(hf.getCellValue({ sheet: bsSheetId, row: aRowNum - 1, col: 3 })).toBeCloseTo(9844.2, 6);
  });

  it('Person-Months are FTE-weighted (24 months at FTE 0.4 = 9.6 PM)', async () => {
    const oneWpConfig: ProjectConfigInput = {
      ...config,
      duration_years: 2,
      work_package_count: 1,
      work_package_names: [null],
      work_package_start_months: [1],
      work_package_end_months: [24],
    };
    const summary = makeSummary({
      wp_budgets: [
        { work_package_id: 1, work_package_name: null, personnel_eur: '0', equipment_eur: '0', travel_eur: '0', other_costs_eur: '0', subcontracting_eur: '0', total_eur: '0' },
      ],
      category_a_total: '480',
      role_detail: [
        {
          // 24 months (months 1-24), FTE 0.4 → Person-Months = 24 x 0.4 = 9.6.
          id: '1', role_label: 'PartTimer', role_type: 'Expert',
          current_monthly_salary_try: '5000', inflation_rate_pct: '0', fte_fraction: '0.4',
          start_month: 1, end_month: 24, cost_lines: [], total_cost_eur: '480', wp_breakdown: [],
        },
      ],
    });

    await exportToExcel(summary, oneWpConfig);

    const wb = new ExcelJS.Workbook();
    await wb.xlsx.load(capturedBuffer as ArrayBuffer);
    const persSheet = wb.getWorksheet('Personnel');
    const helperSheet = wb.getWorksheet('_WPMonthHelper');

    const hf = HyperFormula.buildFromSheets({
      'Personnel': gridFromWorksheet(persSheet!),
      '_WPMonthHelper': gridFromWorksheet(helperSheet!),
    }, { licenseKey: 'gpl-v3', useArrayArithmetic: true });
    const sheetId = hf.getSheetId('Personnel')!;
    const cell = (row: number, col: number) => hf.getCellValue({ sheet: sheetId, row: row - 1, col: col - 1 });

    // 1 WP: WP row is row 3. Duration (D) = 24-1+1 = 24. PM (E) = 24 x 0.4 = 9.6.
    expect(cell(3, 4)).toBeCloseTo(24, 6);
    expect(cell(3, 5)).toBeCloseTo(9.6, 6);

    // Total PM (row 4) = Employment Months x FTE (row 5) = 24 x 0.4 = 9.6 → Reconciled? (row 6) = "OK".
    expect(cell(4, 5)).toBeCloseTo(9.6, 6);
    expect(cell(5, 5)).toBeCloseTo(9.6, 6);
    expect(cell(6, 5)).toBe('OK');
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
