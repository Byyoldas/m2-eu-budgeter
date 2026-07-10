/**
 * PDF export engine.
 *
 * Generates a print-ready PDF budget summary using the browser's
 * window.print() API with a dedicated print stylesheet.
 * This avoids a heavy PDF library dependency while producing clean output.
 *
 * For production, this can be upgraded to @react-pdf/renderer for
 * richer control over the output format.
 */

import type { BudgetSummaryDto, ProjectConfigInput } from '../types';

function n(v: string | undefined): string {
  const val = parseFloat(v ?? '0') || 0;
  return `€ ${val.toLocaleString('en-GB', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

function buildHtml(summary: BudgetSummaryDto, config: ProjectConfigInput | null): string {
  const years = summary.category_a_by_year.map((y) => y.year);

  const cats = [
    { label: 'A  Personnel',      total: summary.category_a_total, byYear: summary.category_a_by_year },
    { label: 'B  Subcontracting', total: summary.category_b_total, byYear: [] as typeof summary.category_a_by_year },
    { label: 'C1 Travel',         total: summary.category_c1_total, byYear: summary.category_c1_by_year },
    { label: 'C2 Equipment',      total: summary.category_c2_total, byYear: [] as typeof summary.category_a_by_year },
    { label: 'C3 Other Direct',   total: summary.category_c3_total, byYear: summary.category_c3_by_year },
    { label: 'E  Indirect (25%)', total: summary.category_e_total, byYear: summary.category_e_by_year },
  ];

  const catRows = cats.map(({ label, total, byYear }) => `
    <tr>
      <td>${label}</td>
      ${years.map((yr) => {
        const entry = byYear.find((y) => y.year === yr);
        return `<td class="num">${entry ? n(entry.amount_eur) : ''}</td>`;
      }).join('')}
      <td class="num bold">${n(total)}</td>
    </tr>
  `).join('');

  return `
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8"/>
  <title>M2-EU Budgeter — ${config?.project_title ?? 'Export'}</title>
  <style>
    body { font-family: Arial, sans-serif; font-size: 11pt; color: #111; margin: 20mm; }
    h1 { font-size: 16pt; color: #1e3a5f; margin-bottom: 4px; }
    .meta { font-size: 10pt; color: #555; margin-bottom: 16px; }
    table { width: 100%; border-collapse: collapse; margin-top: 12px; }
    th { background: #1e3a5f; color: #fff; padding: 6px 8px; text-align: left; }
    td { padding: 5px 8px; border-bottom: 1px solid #e0e0e0; }
    td.num { text-align: right; font-variant-numeric: tabular-nums; }
    td.bold { font-weight: bold; }
    tr.total td { border-top: 2px solid #1e3a5f; font-weight: bold; }
    tr.grand td { background: #eef4ff; font-weight: bold; font-size: 12pt; }
    .section { margin-top: 24px; page-break-inside: avoid; }
    h2 { font-size: 13pt; color: #1e3a5f; margin-bottom: 4px; }
  </style>
</head>
<body>
  <h1>${config?.project_title ?? 'M2-EU Budgeter'}</h1>
  <div class="meta">
    PI: ${config?.pi_name ?? '—'} &nbsp;|&nbsp;
    Call: ${config?.call_reference ?? '—'} &nbsp;|&nbsp;
    Duration: ${config?.duration_years ?? '—'} years
  </div>

  <div class="section">
    <h2>Budget Summary</h2>
    <table>
      <thead>
        <tr>
          <th>Category</th>
          ${years.map((y) => `<th>Year ${y}</th>`).join('')}
          <th>Total</th>
        </tr>
      </thead>
      <tbody>
        ${catRows}
      </tbody>
      <tfoot>
        <tr class="total">
          <td>Total Direct Costs</td>
          ${years.map(() => '<td></td>').join('')}
          <td class="num">${n(summary.total_direct_costs)}</td>
        </tr>
        <tr class="total">
          <td>Total Eligible Costs</td>
          ${years.map(() => '<td></td>').join('')}
          <td class="num">${n(summary.total_eligible_costs)}</td>
        </tr>
        <tr class="grand">
          <td>EU Contribution Requested</td>
          ${years.map(() => '<td></td>').join('')}
          <td class="num">${n(summary.requested_eu_contribution)}</td>
        </tr>
      </tfoot>
    </table>
  </div>

  ${summary.role_detail.length > 0 ? `
  <div class="section">
    <h2>Personnel Detail</h2>
    <table>
      <thead><tr><th>Role</th><th>Type</th><th>FTE</th><th>Total Cost</th></tr></thead>
      <tbody>
        ${summary.role_detail.map((r) => `
          <tr>
            <td>${r.role_label}</td>
            <td>${r.role_type}</td>
            <td class="num">${parseFloat(r.fte_fraction).toFixed(2)}</td>
            <td class="num">${n(r.total_cost_eur)}</td>
          </tr>
        `).join('')}
      </tbody>
    </table>
  </div>` : ''}

  ${summary.trip_detail.length > 0 ? `
  <div class="section">
    <h2>Travel Detail</h2>
    <table>
      <thead><tr><th>Trip</th><th>Year</th><th>×</th><th>Total</th></tr></thead>
      <tbody>
        ${summary.trip_detail.map((t) => `
          <tr>
            <td>${t.name}</td>
            <td>Year ${t.project_year}</td>
            <td class="num">${t.number_of_instances}</td>
            <td class="num">${n(t.total_trip_cost_eur)}</td>
          </tr>
        `).join('')}
      </tbody>
    </table>
  </div>` : ''}
</body>
</html>`;
}

export function exportToPdf(
  summary: BudgetSummaryDto,
  config: ProjectConfigInput | null,
): void {
  const html = buildHtml(summary, config);
  const win = window.open('', '_blank');
  if (!win) return;
  win.document.write(html);
  win.document.close();
  win.focus();
  setTimeout(() => {
    win.print();
    win.close();
  }, 500);
}
