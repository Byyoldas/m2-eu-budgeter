/**
 * CSV export engine.
 * Exports the budget summary as a flat CSV file for pasting into other tools.
 */

import type { BudgetSummaryDto, ProjectConfigInput, WpBudgetDto } from '../types';

function wpLabel(wp: WpBudgetDto): string {
  return wp.work_package_name || `WP${wp.work_package_id}`;
}

function n(v: string | undefined): string {
  const val = parseFloat(v ?? '0') || 0;
  return val.toFixed(2);
}

function row(...cells: (string | number)[]): string {
  return cells.map((c) => `"${String(c).replace(/"/g, '""')}"`).join(',');
}

function download(content: string, filename: string): void {
  const blob = new Blob([content], { type: 'text/csv;charset=utf-8;' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

export async function exportToCsv(
  summary: BudgetSummaryDto,
  config: ProjectConfigInput | null,
): Promise<void> {
  const wpBudgets = summary.wp_budgets;
  const lines: string[] = [];

  // Project header
  lines.push(row('M2-EU Budgeter Export'));
  lines.push(row('Project', config?.project_title ?? ''));
  lines.push(row('PI', config?.pi_name ?? ''));
  lines.push(row('Call', config?.call_reference ?? ''));
  lines.push('');

  // Category summary header
  lines.push(row('Category', ...wpBudgets.map(wpLabel), 'Total (EUR)'));

  const cats: { label: string; total: string; key: keyof WpBudgetDto | null }[] = [
    { label: 'A  Personnel',      total: summary.category_a_total, key: 'personnel_eur' },
    { label: 'B  Subcontracting', total: summary.category_b_total, key: 'subcontracting_eur' },
    { label: 'C1 Travel',         total: summary.category_c1_total, key: 'travel_eur' },
    { label: 'C2 Equipment',      total: summary.category_c2_total, key: 'equipment_eur' },
    { label: 'C3 Other Direct',   total: summary.category_c3_total, key: 'other_costs_eur' },
    { label: 'E  Indirect',       total: summary.category_e_total, key: null },
  ];

  for (const { label, total, key } of cats) {
    lines.push(row(
      label,
      ...wpBudgets.map((wp) => (key ? n(wp[key] as string) : '')),
      n(total),
    ));
  }

  lines.push('');
  lines.push(row('Total Direct Costs', '', n(summary.total_direct_costs)));
  lines.push(row('Total Eligible Costs', '', n(summary.total_eligible_costs)));
  lines.push(row('EU Contribution Requested', '', n(summary.requested_eu_contribution)));

  // Personnel
  if (summary.role_detail.length > 0) {
    lines.push('');
    lines.push(row('PERSONNEL DETAIL'));
    lines.push(row('Role', 'Type', 'FTE', 'Start Month', 'End Month', 'Total'));
    for (const role of summary.role_detail) {
      lines.push(row(
        role.role_label,
        role.role_type,
        parseFloat(role.fte_fraction).toFixed(2),
        role.start_month,
        role.end_month,
        n(role.total_cost_eur),
      ));
    }
  }

  // Equipment
  if (summary.equipment_detail.length > 0) {
    lines.push('');
    lines.push(row('EQUIPMENT DETAIL'));
    lines.push(row('Item', 'Eligible Depreciation', 'Capped'));
    for (const item of summary.equipment_detail) {
      lines.push(row(item.name, n(item.eligible_depreciation_eur), item.is_capped ? 'Yes' : 'No'));
    }
  }

  // Travel
  if (summary.trip_detail.length > 0) {
    lines.push('');
    lines.push(row('TRAVEL DETAIL'));
    lines.push(row('Trip', 'Work Package(s)', 'Instances', 'Per Instance', 'Total'));
    for (const trip of summary.trip_detail) {
      lines.push(row(
        trip.name,
        trip.work_package_ids.map((id) => wpBudgets.find((w) => w.work_package_id === id)?.work_package_name || `WP${id}`).join(', '),
        trip.number_of_instances,
        n(trip.per_instance_total_eur),
        n(trip.total_trip_cost_eur),
      ));
    }
  }

  const filename = `${(config?.project_title ?? 'M2-EU-Budgeter').replace(/\s+/g, '_')}_Budget.csv`;
  download(lines.join('\n'), filename);
}
