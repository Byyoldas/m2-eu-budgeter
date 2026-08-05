/**
 * Tests for the M-20 Excel export engine. jsdom doesn't implement
 * Blob/URL.createObjectURL/anchor clicks, so those are mocked (same
 * pattern as erc-budget's excelExporter.test.ts) and the captured buffer is
 * fed back into ExcelJS to assert on the real generated workbook.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import ExcelJS from 'exceljs';
import {
  exportFinancialReport,
  exportTechnicalReportAnnex,
  exportRiskRegister,
  exportPersonMonthDeclaration,
} from './excelExporter';
import type { ExecutionProjectSummaryDto } from '../types';

function baseSummary(overrides: Partial<ExecutionProjectSummaryDto> = {}): ExecutionProjectSummaryDto {
  return {
    project_info: {
      project_title: 'Test Project',
      pi_name: 'Dr. Demir',
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
    current_project_month: 20,
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
    reporting_period_coverage: { gaps_detected: false, final_period_covers_project_end: true },
    risks: [],
    issues: [],
    warnings: [],
    ...overrides,
  };
}

describe('excelExporter', () => {
  let capturedBuffer: ArrayBuffer | null = null;

  beforeEach(() => {
    capturedBuffer = null;
    vi.stubGlobal(
      'Blob',
      class {
        parts: BlobPart[];
        constructor(parts: BlobPart[]) {
          this.parts = parts;
          capturedBuffer = parts[0] as ArrayBuffer;
        }
      },
    );
    vi.stubGlobal('URL', { createObjectURL: vi.fn(() => 'blob:mock'), revokeObjectURL: vi.fn() });
    const realCreateElement = document.createElement.bind(document);
    vi.spyOn(document, 'createElement').mockImplementation((tag: string) =>
      tag === 'a' ? ({ click: vi.fn(), href: '', download: '' } as unknown as HTMLElement) : realCreateElement(tag),
    );
  });

  async function loadWorkbook(): Promise<ExcelJS.Workbook> {
    expect(capturedBuffer).not.toBeNull();
    const wb = new ExcelJS.Workbook();
    await wb.xlsx.load(capturedBuffer as ArrayBuffer);
    return wb;
  }

  it('exportFinancialReport writes category planned/actual/variance rows', async () => {
    const summary = baseSummary({
      planned: {
        ...baseSummary().planned,
        category_a_total: '1000',
      },
      actuals: {
        ...baseSummary().actuals,
        a_actual: '1200',
        category_a_overrun: true,
      },
    });
    await exportFinancialReport(summary);
    const wb = await loadWorkbook();
    const sheet = wb.getWorksheet('Summary');
    expect(sheet).toBeDefined();
    expect(sheet!.getRow(1).getCell(1).value).toBe('Test Project');
    expect(sheet!.getRow(5).values).toEqual([, 'A (Personnel)', 1000, 1200, 200, 'Yes']);
  });

  it('exportFinancialReport writes a Work Packages sheet', async () => {
    const summary = baseSummary({
      work_packages: [
        {
          work_package_id: 1,
          work_package_name: 'Data Collection',
          leader_role_id: null,
          leader_role_label: null,
          notes: null,
          status: 'OnTrack',
          planned_eur: '5000',
          actual_eur: '2000',
          overspend_warning: false,
        },
      ],
    });
    await exportFinancialReport(summary);
    const wb = await loadWorkbook();
    const sheet = wb.getWorksheet('Work Packages');
    expect(sheet!.getRow(5).values).toEqual([, 'WP1', 'Data Collection', 5000, 2000, 'OnTrack', 'No']);
  });

  it('exportTechnicalReportAnnex writes deliverable and milestone sheets', async () => {
    const summary = baseSummary({
      deliverables: [
        {
          id: 'd1',
          deliverable_number: 'D1.1',
          title: 'Protocol',
          deliverable_type: 'Dataset',
          work_package_id: 1,
          planned_month: 6,
          responsible_role_id: 'r1',
          responsible_role_label: 'PostDoc-1',
          dissemination_level: 'Public',
          status: 'Accepted',
          actual_submission_date: '2026-06-01',
          revision_note: null,
          revised_planned_month: null,
          cordis_registered: true,
          notes: null,
          is_overdue: false,
          cordis_warning: false,
          reporting_period_number: 1,
        },
      ],
      milestones: [
        {
          id: 'm1',
          title: 'Prototype ready',
          work_package_id: 1,
          planned_month: 6,
          status: 'OnTrack',
          effective_status: 'OnTrack',
          actual_completion_month: null,
          linked_deliverable_ids: ['d1'],
        },
      ],
    });
    await exportTechnicalReportAnnex(summary);
    const wb = await loadWorkbook();
    const delSheet = wb.getWorksheet('Deliverables');
    expect(delSheet!.getRow(5).values).toEqual([
      ,
      'D1.1',
      'Protocol',
      'WP1',
      'Dataset',
      6,
      'Accepted',
      '2026-06-01',
      'No',
    ]);
    const msSheet = wb.getWorksheet('Milestones');
    expect(msSheet!.getRow(5).values).toEqual([, 'Prototype ready', 'WP1', 6, 'OnTrack', 'OnTrack', '', 1]);
  });

  it('exportRiskRegister sorts by risk score descending', async () => {
    const summary = baseSummary({
      risks: [
        {
          id: 'low',
          title: 'Low risk',
          description: 'D',
          work_package_id: null,
          probability: 'Low',
          impact: 'Low',
          mitigation: null,
          status: 'Open',
          owner_role_id: null,
          owner_role_label: null,
          identified_date: '2026-01-01',
          review_date: null,
          closed_date: null,
          risk_score: 1,
          priority: 'Low',
        },
        {
          id: 'high',
          title: 'High risk',
          description: 'D',
          work_package_id: 1,
          probability: 'High',
          impact: 'High',
          mitigation: null,
          status: 'Open',
          owner_role_id: null,
          owner_role_label: 'PI',
          identified_date: '2026-01-01',
          review_date: '2026-06-15',
          closed_date: null,
          risk_score: 9,
          priority: 'High',
        },
      ],
    });
    await exportRiskRegister(summary);
    const wb = await loadWorkbook();
    const sheet = wb.getWorksheet('Risks');
    expect(sheet!.getRow(5).values).toEqual([, 9, 'High', 'High risk', 'WP1', 'High', 'High', 'Open', 'PI', '2026-06-15']);
    expect(sheet!.getRow(6).values).toEqual([, 1, 'Low', 'Low risk', '', 'Low', 'Low', 'Open', '', '']);
  });

  it('exportPersonMonthDeclaration groups records by reporting period', async () => {
    const summary = baseSummary({
      persons: [
        {
          id: 'p1',
          full_name: 'Ada Lovelace',
          email: null,
          institution: null,
          orcid: null,
          linked_role_id: 'r1',
          linked_role_label: 'PostDoc-1',
          actual_start_date: '2026-01-01',
          actual_end_date: null,
        },
      ],
      person_months: [
        { id: 'pm1', person_id: 'p1', project_month: 5, reported_months: '1', approved_months: '1', salary_cost_estimate_eur: '1000', calendar_year: 2026, calendar_month: 5 },
        { id: 'pm2', person_id: 'p1', project_month: 25, reported_months: '1', approved_months: null, salary_cost_estimate_eur: null, calendar_year: 2028, calendar_month: 1 },
      ],
      reporting_periods: [
        {
          id: 'per1',
          period_number: 1,
          start_month: 1,
          end_month: 18,
          submission_deadline: null,
          technical_report_submitted: false,
          financial_report_submitted: false,
          status: 'Open',
          deliverables_due: 0,
          deliverables_submitted: 0,
        },
        {
          id: 'per2',
          period_number: 2,
          start_month: 19,
          end_month: 36,
          submission_deadline: null,
          technical_report_submitted: false,
          financial_report_submitted: false,
          status: 'Open',
          deliverables_due: 0,
          deliverables_submitted: 0,
        },
      ],
    });
    await exportPersonMonthDeclaration(summary);
    const wb = await loadWorkbook();
    const p1Sheet = wb.getWorksheet('P1');
    expect(p1Sheet!.getRow(5).values).toEqual([, 'Ada Lovelace', 'PostDoc-1', 5, 1, 1, 1000]);
    const p2Sheet = wb.getWorksheet('P2');
    expect(p2Sheet!.getRow(5).values).toEqual([, 'Ada Lovelace', 'PostDoc-1', 25, 1, '', '']);
  });

  it('exportPersonMonthDeclaration falls back to a single sheet when no periods exist', async () => {
    const summary = baseSummary({ reporting_periods: [] });
    await exportPersonMonthDeclaration(summary);
    const wb = await loadWorkbook();
    expect(wb.getWorksheet('All Periods')).toBeDefined();
  });

  it('every sheet ends with a "Generated by ERC Execution" footer row', async () => {
    const summary = baseSummary();
    await exportFinancialReport(summary);
    const wb = await loadWorkbook();
    const sheet = wb.getWorksheet('Summary')!;
    const lastRow = sheet.getRow(sheet.rowCount);
    expect(lastRow.getCell(1).value).toBe('Generated by ERC Execution');
  });
});
