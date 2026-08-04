import { describe, it, expect } from 'vitest';
import { buildProjectStatusReportHtml } from './pdfExporter';
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
      category_a_total: '1000',
      category_b_total: '0',
      category_c1_total: '0',
      category_c2_total: '0',
      category_c3_total: '0',
      indirect_base_total: '1000',
      category_e_total: '250',
      total_direct_costs: '1000',
      total_eligible_costs: '1250',
      requested_eu_contribution: '1250',
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

describe('buildProjectStatusReportHtml', () => {
  it('includes the project title, PI, and current month', () => {
    const html = buildProjectStatusReportHtml(baseSummary());
    expect(html).toContain('Test Project');
    expect(html).toContain('Dr. Demir');
    expect(html).toContain('Month 20 of 60');
  });

  it('includes planned and actual category values', () => {
    const html = buildProjectStatusReportHtml(baseSummary());
    expect(html).toContain('1,000');
    expect(html).toContain('1,250');
  });

  it('lists warnings when present', () => {
    const html = buildProjectStatusReportHtml(
      baseSummary({
        warnings: [
          {
            code: 'W-01',
            severity: 'Error',
            message: 'D1.1 is overdue',
            navigation_target: 'Deliverables',
            entity_id: null,
          },
        ],
      }),
    );
    expect(html).toContain('D1.1 is overdue');
    expect(html).toContain('Warnings (1)');
  });

  it('shows "None" when there are no warnings', () => {
    const html = buildProjectStatusReportHtml(baseSummary());
    expect(html).toContain('Warnings (0)');
    expect(html).toMatch(/<li>None<\/li>/);
  });

  it('lists upcoming reporting deadlines, excluding submitted periods', () => {
    const html = buildProjectStatusReportHtml(
      baseSummary({
        reporting_periods: [
          {
            id: 'p1',
            period_number: 1,
            start_month: 1,
            end_month: 18,
            submission_deadline: '2027-04-01',
            technical_report_submitted: true,
            financial_report_submitted: true,
            status: 'Submitted',
            deliverables_due: 0,
            deliverables_submitted: 0,
          },
          {
            id: 'p2',
            period_number: 2,
            start_month: 19,
            end_month: 36,
            submission_deadline: '2028-10-01',
            technical_report_submitted: false,
            financial_report_submitted: false,
            status: 'Open',
            deliverables_due: 0,
            deliverables_submitted: 0,
          },
        ],
      }),
    );
    expect(html).not.toContain('P1: 2027-04-01');
    expect(html).toContain('P2: 2028-10-01');
  });

  it('escapes HTML in user-entered text', () => {
    const html = buildProjectStatusReportHtml(
      baseSummary({
        project_info: {
          ...baseSummary().project_info,
          project_title: '<script>alert(1)</script>',
        },
      }),
    );
    expect(html).not.toContain('<script>alert(1)</script>');
    expect(html).toContain('&lt;script&gt;');
  });

  it('escapes HTML in a reporting period submission_deadline (defense-in-depth against a hand-edited file)', () => {
    const html = buildProjectStatusReportHtml(
      baseSummary({
        reporting_periods: [
          {
            id: 'p1',
            period_number: 1,
            start_month: 1,
            end_month: 18,
            submission_deadline: '<img src=x onerror=alert(1)>',
            technical_report_submitted: false,
            financial_report_submitted: false,
            status: 'Open',
            deliverables_due: 0,
            deliverables_submitted: 0,
          },
        ],
      }),
    );
    expect(html).not.toContain('<img src=x onerror=alert(1)>');
    expect(html).toContain('&lt;img src=x onerror=alert(1)&gt;');
  });
});
