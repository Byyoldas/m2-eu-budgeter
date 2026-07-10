/**
 * CSV export engine.
 * Exports the budget summary as a flat CSV file for pasting into other tools.
 */

import type { BudgetSummaryDto, ProjectConfigInput } from '../types';

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
  const years = summary.category_a_by_year.map((y) => y.year);
  const lines: string[] = [];

  // Project header
  lines.push(row('M2-EU Budgeter Export'));
  lines.push(row('Project', config?.project_title ?? ''));
  lines.push(row('PI', config?.pi_name ?? ''));
  lines.push(row('Call', config?.call_reference ?? ''));
  lines.push('');

  // Category summary header
  lines.push(row('Category', ...years.map((y) => `Year ${y}`), 'Total (EUR)'));

  const cats = [
    { label: 'A  Personnel',      total: summary.category_a_total, byYear: summary.category_a_by_year },
    { label: 'B  Subcontracting', total: summary.category_b_total, byYear: [] as typeof summary.category_a_by_year },
    { label: 'C1 Travel',         total: summary.category_c1_total, byYear: summary.category_c1_by_year },
    { label: 'C2 Equipment',      total: summary.category_c2_total, byYear: [] as typeof summary.category_a_by_year },
    { label: 'C3 Other Direct',   total: summary.category_c3_total, byYear: summary.category_c3_by_year },
    { label: 'E  Indirect',       total: summary.category_e_total, byYear: summary.category_e_by_year },
  ];

  for (const { label, total, byYear } of cats) {
    lines.push(row(
      label,
      ...years.map((yr) => {
        const entry = byYear.find((y) => y.year === yr);
        return entry ? n(entry.amount_eur) : '';
      }),
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
    lines.push(row('Role', 'Type', 'FTE', ...years.map((y) => `Year ${y}`), 'Total'));
    for (const role of summary.role_detail) {
      lines.push(row(
        role.role_label,
        role.role_type,
        parseFloat(role.fte_fraction).toFixed(2),
        ...years.map((yr) => {
          const line = role.cost_lines.find((l) => l.year === yr);
          return line?.is_active ? n(line.annual_cost_eur) : '0.00';
        }),
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
    lines.push(row('Trip', 'Year', 'Instances', 'Per Instance', 'Total'));
    for (const trip of summary.trip_detail) {
      lines.push(row(
        trip.name,
        `Year ${trip.project_year}`,
        trip.number_of_instances,
        n(trip.per_instance_total_eur),
        n(trip.total_trip_cost_eur),
      ));
    }
  }

  const filename = `${(config?.project_title ?? 'M2-EU-Budgeter').replace(/\s+/g, '_')}_Budget.csv`;
  download(lines.join('\n'), filename);
}
