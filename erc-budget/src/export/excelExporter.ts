/**
 * Excel export engine.
 *
 * Produces a multi-sheet workbook:
 *   Sheet 1: Budget Summary (category totals + Work Package breakdown; totals
 *            are formulas that SUM the detail sheets rather than static copies)
 *   Sheet 2: Gantt Chart (rendered PNG of the Work Package timeline)
 *   Sheet 3: Personnel Detail — a "WP Timelines" table (WP Start/End Month,
 *            an inclusive Duration column, and a Person-Months column per
 *            role reconciled against each role's raw employment length) sits
 *            above the roles table (salary/inflation input cells + formula-
 *            built Base Monthly cost; per-Work-Package cost is a genuine
 *            formula — SUMPRODUCT over a hidden per-month helper sheet that
 *            replicates the backend's month-by-month WP-overlap allocation,
 *            with yearly inflation compounding applied per month — plus a
 *            formula-derived Unattributed column that reconciles against the
 *            backend's total)
 *   Sheet 4: Equipment Detail
 *   Sheet 5: Travel Detail
 *   Sheet 6: Other Direct Costs
 *
 * Uses ExcelJS. The file is downloaded via a data URL (browser-side).
 * On Tauri desktop the data URL approach works through the WebView.
 */

import ExcelJS from 'exceljs';
import type { BudgetSummaryDto, ProjectConfigInput, WpBudgetDto, CountrySummary } from '../types';

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
  countries: CountrySummary[] = [],
): Promise<void> {
  const wb = new ExcelJS.Workbook();
  wb.creator = 'M2-EU Budgeter';
  wb.created = new Date();

  const wpBudgets = summary.wp_budgets;
  const countryName = (code: string | null) => {
    if (!code) return '';
    return countries.find((c) => c.country_code === code)?.country_name ?? code;
  };

  // ── Precompute detail-sheet row layouts (needed so Budget Summary formulas
  // can reference the right ranges before those sheets exist) ───────────────

  const personnelExists = summary.role_detail.length > 0;
  const maxMonths = (config?.duration_years ?? 1) * 12;
  // Personnel sheet layout: a "WP Timelines" reference table at the top
  // (rows 1-2 + one row per WP + a 3-row PM reconciliation block), then a
  // blank row, then the roles table. The WP Timelines table now carries a
  // Duration column plus one Person-Months (PM) column per role — each
  // role's PM in a WP is that role's month-count allocation to the WP
  // (same month-overlap/reciprocal-split logic as the cost formulas below,
  // just without the salary/inflation/FTE factors), so the table doubles as
  // a visible audit of the allocation rather than only the cost outcome.
  const personnelWpTimelineFirstRow = 3;
  const personnelWpTimelineLastRow = 2 + wpBudgets.length;
  const personnelWpTimelineDurationCol = 4; // D
  const personnelWpTimelinePmStartCol = 5; // E.. — one column per role, in role_detail order
  const personnelWpTimelineTotalPmRow = personnelWpTimelineLastRow + 1;
  const personnelWpTimelineEmploymentRow = personnelWpTimelineLastRow + 2;
  // Row 3 below (personnelWpTimelineLastRow + 3) is the "Reconciled?" row;
  // + 1 blank row, + 1 header row brings us to personnelHeaderRow.
  const personnelHeaderRow = personnelWpTimelineLastRow + 5;
  const personnelFirstDataRow = personnelHeaderRow + 1;
  const personnelLastRow = personnelFirstDataRow + summary.role_detail.length - 1;
  const personnelFixedCols = 7; // Role, Type, Salary(TRY), Increase%, FTE, Start, End
  const personnelBaseMonthlyCol = personnelFixedCols + 1; // H
  const personnelWpStartCol = personnelBaseMonthlyCol + 1; // I
  const personnelUnattributedCol = personnelWpStartCol + wpBudgets.length; // right after the last WP column
  const personnelTotalCol = personnelUnattributedCol + 1;
  const personnelTotalColLetter = colLetter(personnelTotalCol);
  // Hidden helper sheet columns: A = row label, B.. = one column per project month.
  const helperMonthRange = `_WPMonthHelper!$B$1:$${colLetter(1 + maxMonths)}$1`;
  const helperReciprocalRange = `_WPMonthHelper!$B$3:$${colLetter(1 + maxMonths)}$3`;

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
    personnelExists ? `SUM(Personnel!${personnelTotalColLetter}${personnelFirstDataRow}:${personnelTotalColLetter}${personnelLastRow})` : 0,
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
  //
  // A hidden helper sheet drives the per-Work-Package formulas: row 1 lists
  // every project month (1..duration*12), row 2 counts how many WPs cover
  // that month (from the Personnel sheet's own WP Timelines table), and row
  // 3 is the safe reciprocal (0 for months no WP covers, avoiding #DIV/0!
  // instead of relying on the WP-membership term to zero it out afterwards).

  if (personnelExists) {
    const helperSheet = wb.addWorksheet('_WPMonthHelper');
    helperSheet.state = 'hidden';

    const monthRow = helperSheet.addRow(['Month']);
    for (let m = 1; m <= maxMonths; m++) {
      monthRow.getCell(1 + m).value = m;
    }
    const countRow = helperSheet.addRow(['WP overlap count']);
    const reciprocalRow = helperSheet.addRow(['1 / count (0 if uncovered)']);
    for (let m = 1; m <= maxMonths; m++) {
      const col = 1 + m;
      const monthCellRef = `${colLetter(col)}$1`;
      const countFormula = wpBudgets.length > 0
        ? `SUMPRODUCT((Personnel!$B$${personnelWpTimelineFirstRow}:$B$${personnelWpTimelineLastRow}<=${monthCellRef})*(Personnel!$C$${personnelWpTimelineFirstRow}:$C$${personnelWpTimelineLastRow}>=${monthCellRef}))`
        : '0';
      countRow.getCell(col).value = { formula: countFormula };
      const countCellRef = `${colLetter(col)}2`;
      reciprocalRow.getCell(col).value = { formula: `IF(${countCellRef}=0,0,1/${countCellRef})` };
    }

    const persSheet = wb.addWorksheet('Personnel');
    persSheet.properties.defaultColWidth = 15;

    // WP Timelines reference table (rows 1-2 + one row per WP), extended with
    // a Duration column (End - Start + 1, inclusive of both ends — the same
    // inclusive convention the month-overlap formulas below use, so there's
    // no off-by-one drift between "how long is this WP" and "how many months
    // of it does each role get") and one Person-Months (PM) column per role.
    persSheet.addRow(['Work Package Timelines']).font = { bold: true };
    const wpTimelineHeader = persSheet.addRow([
      'WP', 'Start Month', 'End Month', 'Duration (Months)',
      ...summary.role_detail.map((r) => `${r.role_label} (PM)`),
    ]);
    wpTimelineHeader.font = { bold: true };
    wpBudgets.forEach((wp, i) => {
      const wpTableRow = personnelWpTimelineFirstRow + i;
      const row = persSheet.addRow([
        wpLabel(wp),
        config?.work_package_start_months[i] ?? 1,
        config?.work_package_end_months[i] ?? maxMonths,
      ]);
      const durationCell = row.getCell(personnelWpTimelineDurationCol);
      durationCell.value = { formula: `C${wpTableRow}-B${wpTableRow}+1` };
      durationCell.numFmt = '0';

      // Each role's Person-Months (PM) in this WP is an FTE-weighted month
      // allocation — Person-Months = calendar months × FTE (e.g. 24 months
      // at FTE 0.4 = 9.6 PM) — via SUMPRODUCT over every project month of
      //   [month in role's Start-End] * [month in this WP's Start-End] *
      //   [1/overlap-count that month] * [FTE]
      // — the same month-overlap/reciprocal-split term the per-WP cost
      // formula below uses (minus the salary/inflation factors), so a role
      // split across simultaneously-active WPs gets its PM split the same
      // way its cost is split.
      summary.role_detail.forEach((_role, k) => {
        const roleRow = personnelFirstDataRow + k;
        const roleStartCell = `$F$${roleRow}`;
        const roleEndCell = `$G$${roleRow}`;
        const roleFteCell = `$E$${roleRow}`;
        const wpStartCellSelf = `$B$${wpTableRow}`;
        const wpEndCellSelf = `$C$${wpTableRow}`;
        const col = personnelWpTimelinePmStartCol + k;
        const cell = row.getCell(col);
        cell.value = {
          formula: `SUMPRODUCT((${helperMonthRange}>=${roleStartCell})*(${helperMonthRange}<=${roleEndCell})*(${helperMonthRange}>=${wpStartCellSelf})*(${helperMonthRange}<=${wpEndCellSelf})*${helperReciprocalRange})*${roleFteCell}`,
        };
        cell.numFmt = '0.00';
      });
    });

    // Reconciliation block: for each role, Total PM (summed down its column
    // across every WP row above) must equal its FTE-weighted Employment
    // Months ((End - Start + 1) × FTE) — proving no month was double-counted
    // or dropped by the WP split, independent of the cost/inflation math.
    const totalPmRow = persSheet.addRow(['Total PM (check)']);
    totalPmRow.font = { italic: true };
    const employmentRow = persSheet.addRow(['Employment Months']);
    employmentRow.font = { italic: true };
    const reconciledRow = persSheet.addRow(['Reconciled?']);
    reconciledRow.font = { italic: true, bold: true };

    summary.role_detail.forEach((_role, k) => {
      const roleRow = personnelFirstDataRow + k;
      const col = personnelWpTimelinePmStartCol + k;
      const colLetterStr = colLetter(col);

      const totalPmCell = totalPmRow.getCell(col);
      totalPmCell.value = {
        formula: `SUM(${colLetterStr}${personnelWpTimelineFirstRow}:${colLetterStr}${personnelWpTimelineLastRow})`,
      };
      totalPmCell.numFmt = '0.00';

      const employmentCell = employmentRow.getCell(col);
      employmentCell.value = { formula: `(G${roleRow}-F${roleRow}+1)*E${roleRow}` };
      employmentCell.numFmt = '0.00';

      reconciledRow.getCell(col).value = {
        formula: `IF(${colLetterStr}${personnelWpTimelineTotalPmRow}=${colLetterStr}${personnelWpTimelineEmploymentRow},"OK","MISMATCH")`,
      };
    });

    persSheet.addRow([]);

    const headers = [
      'Role', 'Type', 'Current Salary (TRY)', 'Annual Increase (%)', 'FTE',
      'Start Month', 'End Month', 'Base Monthly (€)',
      ...wpBudgets.map((wp) => `${wpLabel(wp)} (€)`),
      'Unattributed (€)',
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

      // Per-WP cost: SUMPRODUCT over every project month of
      //   [month in role's Start-End] * [month in this WP's Start-End] *
      //   [that month's inflated salary] * [1/overlap-count that month] * FTE
      // This mirrors allocate_personnel_cost_by_wp exactly, including the
      // even split across WPs that are simultaneously active in a month.
      wpBudgets.forEach((_wp, i) => {
        const wpTableRow = personnelWpTimelineFirstRow + i;
        const wpStartCell = `$B$${wpTableRow}`;
        const wpEndCell = `$C$${wpTableRow}`;
        const col = personnelWpStartCol + i;
        row.getCell(col).value = {
          formula: `SUMPRODUCT((${helperMonthRange}>=${startCell})*(${helperMonthRange}<=${endCell})*(${helperMonthRange}>=${wpStartCell})*(${helperMonthRange}<=${wpEndCell})*(${baseMonthlyCell}*(1+${increaseCell}/100)^ROUNDUP(${helperMonthRange}/12,0))*${helperReciprocalRange}*${fteCell})`,
        };
      });

      row.getCell(personnelTotalCol).value = n(role.total_cost_eur);

      // Any months of the role's Start/End period outside every WP timeline
      // aren't attributed to a WP bucket — surfaced here so the row still
      // reconciles to the authoritative Total. In the normal case (every
      // active month falls inside some WP) this is 0.
      const totalCellRef = `${colLetter(personnelTotalCol)}${r}`;
      if (wpBudgets.length > 0) {
        const wpRangeStart = `${colLetter(personnelWpStartCol)}${r}`;
        const wpRangeEnd = `${colLetter(personnelWpStartCol + wpBudgets.length - 1)}${r}`;
        row.getCell(personnelUnattributedCol).value = {
          formula: `${totalCellRef}-SUM(${wpRangeStart}:${wpRangeEnd})`,
        };
      } else {
        row.getCell(personnelUnattributedCol).value = { formula: totalCellRef };
      }
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
    const eh = eqSheet.addRow([
      'Item', 'Theoretical (€)', 'Max (€)', 'Eligible Depreciation (€)', 'Capped?',
      'Purchase Cost (€)', 'Total Need an External Funding (€)',
    ]);
    eh.font = { bold: true };

    for (const item of summary.equipment_detail) {
      const row = eqSheet.addRow([
        item.name,
        n(item.theoretical_eligible_eur),
        n(item.maximum_eligible_eur),
        n(item.eligible_depreciation_eur),
        item.is_capped ? 'Yes' : 'No',
        n(item.purchase_cost_eur),
      ]);
      // Total Need an External Funding = Purchase Cost − Eligible Depreciation
      row.getCell(7).value = { formula: `F${row.number}-D${row.number}` };
    }
    for (const col of [2, 3, 4, 6, 7]) {
      eqSheet.getColumn(col).numFmt = '#,##0.00';
    }
  }

  // ── Sheet 5: Travel ───────────────────────────────────────────────────────

  const wpTag = (ids: number[]) =>
    ids.map((id) => wpBudgets.find((w) => w.work_package_id === id)?.work_package_name || `WP${id}`).join(', ');

  if (travelExists) {
    const travelSheet = wb.addWorksheet('Travel');
    travelSheet.properties.defaultColWidth = 18;
    const th = travelSheet.addRow([
      'Trip', 'Work Package(s)', 'Instances', 'Per Instance (€)', 'Total (€)',
      'Destination Country', 'Flight Cost (€)', 'Accommodation Cost (€)',
      'Subsistence Cost (€)', 'Domestic Transport Cost (€)',
    ]);
    th.font = { bold: true };

    for (const trip of summary.trip_detail) {
      travelSheet.addRow([
        trip.name,
        wpTag(trip.work_package_ids),
        trip.number_of_instances,
        n(trip.per_instance_total_eur),
        n(trip.total_trip_cost_eur),
        countryName(trip.destination_country_code),
        trip.flight_cost_per_instance !== null ? n(trip.flight_cost_per_instance) : '',
        trip.accommodation_cost_per_instance !== null ? n(trip.accommodation_cost_per_instance) : '',
        trip.subsistence_cost_per_instance !== null ? n(trip.subsistence_cost_per_instance) : '',
        trip.domestic_transport_per_instance !== null ? n(trip.domestic_transport_per_instance) : '',
      ]);
    }
    for (const col of [4, 5, 7, 8, 9, 10]) {
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
