/**
 * Excel export engine.
 *
 * Produces a multi-sheet workbook:
 *   Sheet 1: Budget Summary (category totals + year breakdown)
 *   Sheet 2: Personnel Detail
 *   Sheet 3: Equipment Detail
 *   Sheet 4: Travel Detail
 *
 * Uses ExcelJS. The file is downloaded via a data URL (browser-side).
 * On Tauri desktop the data URL approach works through the WebView.
 */

import ExcelJS from 'exceljs';
import type { BudgetSummaryDto, ProjectConfigInput } from '../types';

function n(v: string | undefined): number {
  return parseFloat(v ?? '0') || 0;
}

async function downloadWorkbook(wb: ExcelJS.Workbook, filename: string): Promise<void> {
  const buf = await wb.xlsx.writeBuffer();
  const blob = new Blob([buf], { type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

export async function exportToExcel(
  summary: BudgetSummaryDto,
  config: ProjectConfigInput | null,
): Promise<void> {
  const wb = new ExcelJS.Workbook();
  wb.creator = 'M2-EU Budgeter';
  wb.created = new Date();

  const years = summary.category_a_by_year.map((y) => y.year);

  // ── Sheet 1: Budget Summary ───────────────────────────────────────────────

  const summarySheet = wb.addWorksheet('Budget Summary');
  summarySheet.properties.defaultColWidth = 18;

  // Title block
  summarySheet.addRow([config?.project_title ?? 'M2-EU Budgeter']);
  summarySheet.addRow([`PI: ${config?.pi_name ?? ''}`, '', `Call: ${config?.call_reference ?? ''}`]);
  summarySheet.addRow([]);

  // Header row
  const headerRow = summarySheet.addRow([
    'Category',
    ...years.map((y) => `Year ${y}`),
    'Total (€)',
  ]);
  headerRow.font = { bold: true };
  headerRow.fill = { type: 'pattern', pattern: 'solid', fgColor: { argb: 'FF1E3A5F' } };
  headerRow.font = { bold: true, color: { argb: 'FFFFFFFF' } };

  const categoryRows = [
    { label: 'A  Personnel',         total: summary.category_a_total, byYear: summary.category_a_by_year },
    { label: 'B  Subcontracting',    total: summary.category_b_total, byYear: [] },
    { label: 'C1 Travel',            total: summary.category_c1_total, byYear: summary.category_c1_by_year },
    { label: 'C2 Equipment',         total: summary.category_c2_total, byYear: [] },
    { label: 'C3 Other Direct',      total: summary.category_c3_total, byYear: summary.category_c3_by_year },
    { label: 'E  Indirect (25%)',    total: summary.category_e_total, byYear: summary.category_e_by_year },
  ];

  for (const { label, total, byYear } of categoryRows) {
    summarySheet.addRow([
      label,
      ...years.map((yr) => {
        const entry = byYear.find((y) => y.year === yr);
        return entry ? n(entry.amount_eur) : '';
      }),
      n(total),
    ]);
  }

  // Totals
  summarySheet.addRow([]);
  const directRow = summarySheet.addRow(['Total Direct Costs', ...years.map(() => ''), n(summary.total_direct_costs)]);
  directRow.font = { bold: true };
  const eligibleRow = summarySheet.addRow(['Total Eligible Costs', ...years.map(() => ''), n(summary.total_eligible_costs)]);
  eligibleRow.font = { bold: true };
  const euRow = summarySheet.addRow(['EU Contribution Requested', ...years.map(() => ''), n(summary.requested_eu_contribution)]);
  euRow.font = { bold: true, color: { argb: 'FF0070C0' } };

  // Format number columns as EUR
  const numCols = years.length + 2;
  for (let col = 2; col <= numCols; col++) {
    summarySheet.getColumn(col).numFmt = '#,##0.00';
  }

  // ── Sheet 2: Personnel ────────────────────────────────────────────────────

  if (summary.role_detail.length > 0) {
    const persSheet = wb.addWorksheet('Personnel');
    persSheet.properties.defaultColWidth = 16;
    const ph = persSheet.addRow(['Role', 'Type', 'FTE', ...years.map((y) => `Year ${y} (€)`), 'Total (€)']);
    ph.font = { bold: true };

    for (const role of summary.role_detail) {
      persSheet.addRow([
        role.role_label,
        role.role_type,
        parseFloat(role.fte_fraction),
        ...years.map((yr) => {
          const line = role.cost_lines.find((l) => l.year === yr);
          return line?.is_active ? n(line.annual_cost_eur) : 0;
        }),
        n(role.total_cost_eur),
      ]);
    }
    persSheet.getColumn(3).numFmt = '0.00';
    for (let col = 4; col <= years.length + 4; col++) {
      persSheet.getColumn(col).numFmt = '#,##0.00';
    }
  }

  // ── Sheet 3: Equipment ────────────────────────────────────────────────────

  if (summary.equipment_detail.length > 0) {
    const eqSheet = wb.addWorksheet('Equipment');
    eqSheet.properties.defaultColWidth = 20;
    const eh = eqSheet.addRow(['Item', 'Theoretical (€)', 'Max (€)', 'Eligible Depreciation (€)', 'Capped?']);
    eh.font = { bold: true };

    for (const item of summary.equipment_detail) {
      eqSheet.addRow([
        item.name,
        n(item.theoretical_eligible_eur),
        n(item.maximum_eligible_eur),
        n(item.eligible_depreciation_eur),
        item.is_capped ? 'Yes' : 'No',
      ]);
    }
    for (let col = 2; col <= 4; col++) {
      eqSheet.getColumn(col).numFmt = '#,##0.00';
    }
  }

  // ── Sheet 4: Travel ───────────────────────────────────────────────────────

  if (summary.trip_detail.length > 0) {
    const travelSheet = wb.addWorksheet('Travel');
    travelSheet.properties.defaultColWidth = 18;
    const th = travelSheet.addRow(['Trip', 'Year', 'Instances', 'Per Instance (€)', 'Total (€)']);
    th.font = { bold: true };

    for (const trip of summary.trip_detail) {
      travelSheet.addRow([
        trip.name,
        `Year ${trip.project_year}`,
        trip.number_of_instances,
        n(trip.per_instance_total_eur),
        n(trip.total_trip_cost_eur),
      ]);
    }
    for (const col of [4, 5]) {
      travelSheet.getColumn(col).numFmt = '#,##0.00';
    }
  }

  // ── Sheet 5: Other Direct Costs ───────────────────────────────────────────

  if (summary.other_cost_detail.length > 0) {
    const ocSheet = wb.addWorksheet('Other Direct Costs');
    ocSheet.properties.defaultColWidth = 20;
    const oh = ocSheet.addRow(['Item', 'Year', 'Amount (€)', 'CFS?', 'Notes']);
    oh.font = { bold: true };

    for (const item of summary.other_cost_detail) {
      ocSheet.addRow([
        item.name,
        `Year ${item.project_year}`,
        n(item.amount_eur),
        item.is_cfs_item ? 'Yes' : 'No',
        item.notes ?? '',
      ]);
    }
    ocSheet.getColumn(3).numFmt = '#,##0.00';
  }

  const filename = `${(config?.project_title ?? 'M2-EU-Budgeter').replace(/\s+/g, '_')}_Budget.xlsx`;
  await downloadWorkbook(wb, filename);
}
