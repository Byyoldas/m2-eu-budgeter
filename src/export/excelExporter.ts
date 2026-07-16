/**
 * Excel export engine.
 *
 * Produces a multi-sheet workbook:
 *   Sheet 1: Budget Summary (category totals + Work Package breakdown; totals
 *            are formulas that SUM the detail sheets rather than static copies)
 *   Sheet 2: Gantt Chart (rendered PNG of the Work Package timeline)
 *   Sheet 3: Personnel Detail (salary/inflation input cells + formula-built
 *            per-year and total costs)
 *   Sheet 4: Equipment Detail
 *   Sheet 5: Travel Detail
 *   Sheet 6: Other Direct Costs
 *
 * Uses ExcelJS. The file is downloaded via a data URL (browser-side).
 * On Tauri desktop the data URL approach works through the WebView.
 */

import ExcelJS from 'exceljs';
import type { BudgetSummaryDto, ProjectConfigInput, WpBudgetDto } from '../types';

function n(v: string | undefined): number {
  return parseFloat(v ?? '0') || 0;
}

function wpLabel(wp: WpBudgetDto): string {
  return wp.work_package_name || `WP${wp.work_package_id}`;
}

/** 1 → A, 2 → B, ..., 27 → AA. Column counts in this workbook never exceed 26. */
function colLetter(index: number): string {
  let s = '';
  let i = index;
  while (i > 0) {
    const rem = (i - 1) % 26;
    s = String.fromCharCode(65 + rem) + s;
    i = Math.floor((i - 1) / 26);
  }
  return s;
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

/**
 * Renders a simple Gantt-style PNG of the Work Package timeline using a 2D
 * canvas. Returns null if canvas isn't usable in the current environment
 * (e.g. jsdom in tests) so callers can skip embedding it.
 */
function renderGanttChartPng(config: ProjectConfigInput): { dataUrl: string; width: number; height: number } | null {
  try {
    const durationMonths = config.duration_years * 12;
    const wpCount = config.work_package_count;
    const rowHeight = 28;
    const rowGap = 8;
    const marginLeft = 160;
    const marginRight = 30;
    const top = 46;
    const chartWidth = 760;
    const width = marginLeft + chartWidth + marginRight;
    const height = top + wpCount * (rowHeight + rowGap) + 20;

    const canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = height;
    const ctx = canvas.getContext('2d');
    if (!ctx) return null;

    ctx.fillStyle = '#ffffff';
    ctx.fillRect(0, 0, width, height);

    ctx.fillStyle = '#1e3a5f';
    ctx.font = 'bold 14px Arial';
    ctx.fillText('Work Package Timeline', 10, 20);

    // Year gridlines
    ctx.strokeStyle = '#cbd5e1';
    ctx.fillStyle = '#64748b';
    ctx.font = '10px Arial';
    for (let m = 0; m <= durationMonths; m += 12) {
      const x = marginLeft + (m / durationMonths) * chartWidth;
      ctx.beginPath();
      ctx.moveTo(x, top - 6);
      ctx.lineTo(x, top + wpCount * (rowHeight + rowGap));
      ctx.stroke();
      ctx.fillText(`Y${m / 12 + 1}`, x + 2, top - 10);
    }

    // WP bars
    for (let i = 0; i < wpCount; i++) {
      const name = (config.work_package_names[i] as string | null) ?? `WP${i + 1}`;
      const start = config.work_package_start_months[i] ?? 1;
      const end = config.work_package_end_months[i] ?? durationMonths;
      const y = top + i * (rowHeight + rowGap);

      ctx.fillStyle = '#334155';
      ctx.font = '11px Arial';
      ctx.fillText(name.length > 22 ? `${name.slice(0, 21)}…` : name, 4, y + rowHeight / 2 + 4);

      const x0 = marginLeft + ((start - 1) / durationMonths) * chartWidth;
      const x1 = marginLeft + (end / durationMonths) * chartWidth;
      ctx.fillStyle = '#3b82f6';
      ctx.fillRect(x0, y, Math.max(x1 - x0, 2), rowHeight);

      ctx.fillStyle = '#ffffff';
      ctx.font = '10px Arial';
      ctx.fillText(`M${start}-M${end}`, x0 + 4, y + rowHeight / 2 + 4);
    }

    return { dataUrl: canvas.toDataURL('image/png'), width, height };
  } catch {
    return null;
  }
}

export async function exportToExcel(
  summary: BudgetSummaryDto,
  config: ProjectConfigInput | null,
): Promise<void> {
  const wb = new ExcelJS.Workbook();
  wb.creator = 'M2-EU Budgeter';
  wb.created = new Date();

  const wpBudgets = summary.wp_budgets;
  const duration = config?.duration_years ?? 1;

  // ── Precompute detail-sheet row layouts (needed so Budget Summary formulas
  // can reference the right ranges before those sheets exist) ───────────────

  const personnelExists = summary.role_detail.length > 0;
  const personnelLastRow = 1 + summary.role_detail.length; // header at row 1
  const personnelFixedCols = 7; // Role, Type, Salary(TRY), Increase%, FTE, Start, End
  const personnelBaseMonthlyCol = personnelFixedCols + 1; // H
  const personnelYear1Col = personnelBaseMonthlyCol + 1; // I
  const personnelTotalCol = personnelYear1Col + duration; // right after the last Year column
  const personnelTotalColLetter = colLetter(personnelTotalCol);

  const equipmentExists = summary.equipment_detail.length > 0;
  const equipmentLastRow = 1 + summary.equipment_detail.length;
  const EQUIPMENT_ELIGIBLE_COL = 'D';

  const travelExists = summary.trip_detail.length > 0;
  const travelLastRow = 1 + summary.trip_detail.length;
  const TRAVEL_TOTAL_COL = 'E';

  const otherCostExists = summary.other_cost_detail.length > 0;
  const otherCostLastRow = 1 + summary.other_cost_detail.length;
  const OTHER_COST_AMOUNT_COL = 'C';

  // ── Sheet 1: Budget Summary ───────────────────────────────────────────────

  const summarySheet = wb.addWorksheet('Budget Summary');
  summarySheet.properties.defaultColWidth = 18;

  summarySheet.addRow([config?.project_title ?? 'M2-EU Budgeter']);
  summarySheet.addRow([`PI: ${config?.pi_name ?? ''}`, '', `Call: ${config?.call_reference ?? ''}`]);
  summarySheet.addRow([]);

  const tryRateRow = summarySheet.addRow(['TRY → EUR Rate:', n(config?.try_eur_rate)]);
  const indirectRateRow = summarySheet.addRow(['Indirect Cost Rate (%):', n(config?.indirect_cost_rate_pct)]);
  const tryRateCellRef = `'Budget Summary'!$B$${tryRateRow.number}`;
  const indirectRateCellRef = `$B$${indirectRateRow.number}`;
  summarySheet.addRow([]);

  const headerRow = summarySheet.addRow([
    'Category',
    ...wpBudgets.map(wpLabel),
    'Total (€)',
  ]);
  headerRow.font = { bold: true, color: { argb: 'FFFFFFFF' } };
  headerRow.fill = { type: 'pattern', pattern: 'solid', fgColor: { argb: 'FF1E3A5F' } };

  const totalColIndex = wpBudgets.length + 2;
  const totalColLetterBS = colLetter(totalColIndex);

  function addCategoryRow(label: string, wpKey: keyof WpBudgetDto | null, totalFormulaOrValue: number | string): ExcelJS.Row {
    const row = summarySheet.addRow([
      label,
      ...wpBudgets.map((wp) => (wpKey ? n(wp[wpKey] as string) : '')),
      typeof totalFormulaOrValue === 'string' ? { formula: totalFormulaOrValue } : totalFormulaOrValue,
    ]);
    return row;
  }

  const aRow = addCategoryRow(
    'A  Personnel', 'personnel_eur',
    personnelExists ? `SUM(Personnel!${personnelTotalColLetter}2:${personnelTotalColLetter}${personnelLastRow})` : 0,
  );
  const bRow = addCategoryRow('B  Subcontracting', 'subcontracting_eur', n(summary.category_b_total));
  const c1Row = addCategoryRow(
    'C1 Travel', 'travel_eur',
    travelExists ? `SUM(Travel!${TRAVEL_TOTAL_COL}2:${TRAVEL_TOTAL_COL}${travelLastRow})` : 0,
  );
  const c2Row = addCategoryRow(
    'C2 Equipment', 'equipment_eur',
    equipmentExists ? `SUM(Equipment!${EQUIPMENT_ELIGIBLE_COL}2:${EQUIPMENT_ELIGIBLE_COL}${equipmentLastRow})` : 0,
  );
  const c3Row = addCategoryRow(
    'C3 Other Direct', 'other_costs_eur',
    otherCostExists ? `SUM('Other Direct Costs'!${OTHER_COST_AMOUNT_COL}2:${OTHER_COST_AMOUNT_COL}${otherCostLastRow})` : 0,
  );
  const eRow = addCategoryRow(
    'E  Indirect', null,
    `(${totalColLetterBS}${aRow.number}+${totalColLetterBS}${c1Row.number}+${totalColLetterBS}${c2Row.number}+${totalColLetterBS}${c3Row.number})*${indirectRateCellRef}/100`,
  );

  summarySheet.addRow([]);

  const directRow = summarySheet.addRow([
    'Total Direct Costs',
    ...wpBudgets.map(() => ''),
    { formula: `${totalColLetterBS}${aRow.number}+${totalColLetterBS}${bRow.number}+${totalColLetterBS}${c1Row.number}+${totalColLetterBS}${c2Row.number}+${totalColLetterBS}${c3Row.number}` },
  ]);
  directRow.font = { bold: true };

  const eligibleRow = summarySheet.addRow([
    'Total Eligible Costs',
    ...wpBudgets.map(() => ''),
    { formula: `${totalColLetterBS}${directRow.number}+${totalColLetterBS}${eRow.number}` },
  ]);
  eligibleRow.font = { bold: true };

  const euRow = summarySheet.addRow([
    'EU Contribution Requested',
    ...wpBudgets.map(() => ''),
    { formula: `${totalColLetterBS}${eligibleRow.number}` },
  ]);
  euRow.font = { bold: true, color: { argb: 'FF0070C0' } };

  for (let col = 2; col <= totalColIndex; col++) {
    summarySheet.getColumn(col).numFmt = '#,##0.00';
  }

  // ── Sheet 2: Gantt Chart ──────────────────────────────────────────────────

  if (config) {
    const gantt = renderGanttChartPng(config);
    if (gantt) {
      const imageId = wb.addImage({ base64: gantt.dataUrl, extension: 'png' });
      const ganttSheet = wb.addWorksheet('Gantt Chart');
      ganttSheet.addImage(imageId, {
        tl: { col: 0, row: 0 },
        ext: { width: gantt.width, height: gantt.height },
      });
    }
  }

  // ── Sheet 3: Personnel ────────────────────────────────────────────────────

  if (personnelExists) {
    const persSheet = wb.addWorksheet('Personnel');
    persSheet.properties.defaultColWidth = 15;

    const headers = [
      'Role', 'Type', 'Current Salary (TRY)', 'Annual Increase (%)', 'FTE',
      'Start Month', 'End Month', 'Base Monthly (€)',
      ...Array.from({ length: duration }, (_, i) => `Year ${i + 1} (€)`),
      'Total (€)',
    ];
    const ph = persSheet.addRow(headers);
    ph.font = { bold: true };

    for (const role of summary.role_detail) {
      const row = persSheet.addRow([
        role.role_label,
        role.role_type,
        n(role.current_monthly_salary_try),
        n(role.inflation_rate_pct),
        n(role.fte_fraction),
        role.start_month,
        role.end_month,
      ]);
      const r = row.number;
      const salaryCell = `C${r}`;
      const increaseCell = `D${r}`;
      const fteCell = `E${r}`;
      const startCell = `F${r}`;
      const endCell = `G${r}`;
      const baseMonthlyCell = `${colLetter(personnelBaseMonthlyCol)}${r}`;

      row.getCell(personnelBaseMonthlyCol).value = { formula: `${salaryCell}/${tryRateCellRef}` };

      for (let y = 1; y <= duration; y++) {
        const col = personnelYear1Col + y - 1;
        const activeMonths = `MAX(0,MIN(${endCell},${y}*12)-MAX(${startCell},(${y}-1)*12+1)+1)`;
        row.getCell(col).value = {
          formula: `${baseMonthlyCell}*(1+${increaseCell}/100)^${y}*${activeMonths}*${fteCell}`,
        };
      }

      row.getCell(personnelTotalCol).value = {
        formula: `SUM(${colLetter(personnelYear1Col)}${r}:${colLetter(personnelYear1Col + duration - 1)}${r})`,
      };
    }

    persSheet.getColumn(3).numFmt = '#,##0.00';
    persSheet.getColumn(5).numFmt = '0.00';
    for (let col = personnelBaseMonthlyCol; col <= personnelTotalCol; col++) {
      persSheet.getColumn(col).numFmt = '#,##0.00';
    }
  }

  // ── Sheet 4: Equipment ────────────────────────────────────────────────────

  if (equipmentExists) {
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

  // ── Sheet 5: Travel ───────────────────────────────────────────────────────

  const wpTag = (ids: number[]) =>
    ids.map((id) => wpBudgets.find((w) => w.work_package_id === id)?.work_package_name || `WP${id}`).join(', ');

  if (travelExists) {
    const travelSheet = wb.addWorksheet('Travel');
    travelSheet.properties.defaultColWidth = 18;
    const th = travelSheet.addRow(['Trip', 'Work Package(s)', 'Instances', 'Per Instance (€)', 'Total (€)']);
    th.font = { bold: true };

    for (const trip of summary.trip_detail) {
      travelSheet.addRow([
        trip.name,
        wpTag(trip.work_package_ids),
        trip.number_of_instances,
        n(trip.per_instance_total_eur),
        n(trip.total_trip_cost_eur),
      ]);
    }
    for (const col of [4, 5]) {
      travelSheet.getColumn(col).numFmt = '#,##0.00';
    }
  }

  // ── Sheet 6: Other Direct Costs ───────────────────────────────────────────

  if (otherCostExists) {
    const ocSheet = wb.addWorksheet('Other Direct Costs');
    ocSheet.properties.defaultColWidth = 20;
    const oh = ocSheet.addRow(['Item', 'Work Package(s)', 'Amount (€)', 'CFS?', 'Notes']);
    oh.font = { bold: true };

    for (const item of summary.other_cost_detail) {
      ocSheet.addRow([
        item.name,
        wpTag(item.work_package_ids),
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
