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
 *            built Base Monthly cost (Actual Cost projects also get a Total PM
 *            column and a totals row — they have no WP Timelines table), plus a display-only Average Monthly
 *            column — the plain average of the role's inflated salary
 *            across every project year, not used by any cost formula;
 *            per-Work-Package cost and the row Total are genuine formulas
 *            driven by two hidden helper sheets: _WPMonthHelper
 *            (month-by-month WP-overlap counts, unchanged) and
 *            _WPYearHelper (one row per role per project year, which
 *            reproduces CALC-03/CALC-20a's Person-Months rounding — round
 *            each role-year's PM to 1 decimal place before pricing it —
 *            entirely as formulas, so the sheet stays live if salary/FTE/
 *            dates are edited) plus a formula-derived Unattributed column
 *            that reconciles against the row's Total)
 *   Sheet 4: Equipment Detail
 *   Sheet 5: Travel Detail
 *   Sheet 6: Other Direct Costs
 *   Sheet 7: Subcontracting Detail
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

  // Actual Cost has no Work Packages: the workbook shows category totals only,
  // with no Work-Package columns, timelines, Gantt chart, or per-WP formulas.
  // An empty `wpBudgets` makes every WP-column loop below a natural no-op;
  // the places that need more than that branch on `isActualCost` explicitly.
  const isActualCost = config?.funding_model === 'ActualCost';
  const wpBudgets = isActualCost ? [] : summary.wp_budgets;
  const countryName = (code: string | null) => {
    if (!code) return '';
    return countries.find((c) => c.country_code === code)?.country_name ?? code;
  };

  // ── Precompute detail-sheet row layouts (needed so Budget Summary formulas
  // can reference the right ranges before those sheets exist) ───────────────

  const personnelExists = summary.role_detail.length > 0;
  const durationYears = config?.duration_years ?? 1;
  const maxMonths = durationYears * 12;
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
  // (Actual Cost has no WP Timelines table, so its roles table starts at row 1.)
  const personnelHeaderRow = isActualCost ? 1 : personnelWpTimelineLastRow + 5;
  const personnelFirstDataRow = personnelHeaderRow + 1;
  const personnelLastRow = personnelFirstDataRow + summary.role_detail.length - 1;
  // _WPYearHelper layout: one row per (role, project year), in role_detail
  // order, years 1..durationYears — a role's block is always contiguous, so
  // its Total (CALC-03) is a single SUM over a row range.
  const yearHelperFirstRow = 2; // row 1 is the header
  const yearHelperRowFor = (roleIdx: number, year: number) =>
    yearHelperFirstRow + roleIdx * durationYears + (year - 1);
  const personnelFixedCols = 7; // Role, Type, Salary(TRY), Increase%, FTE, Start, End
  const personnelBaseMonthlyCol = personnelFixedCols + 1; // H
  const personnelAverageMonthlyCol = personnelBaseMonthlyCol + 1; // I
  // Actual Cost has no WP Timelines table (which is where Lump Sum shows each
  // role's Person-Months), so its roles table carries a Total PM column
  // itself, right after Average Monthly — the figure entered in the portal.
  const personnelTotalPmCol = isActualCost ? personnelAverageMonthlyCol + 1 : null; // J (Actual Cost only)
  const personnelWpStartCol = personnelAverageMonthlyCol + 1 + (isActualCost ? 1 : 0); // J (Lump Sum) / K (Actual Cost)
  const personnelUnattributedCol = personnelWpStartCol + wpBudgets.length; // right after the last WP column (Lump Sum only)
  const personnelTotalCol = isActualCost ? personnelWpStartCol : personnelUnattributedCol + 1;
  const personnelTotalColLetter = colLetter(personnelTotalCol);
  // Hidden helper sheet columns: A = row label, B.. = one column per project month.
  const helperMonthRange = `_WPMonthHelper!$B$1:$${colLetter(1 + maxMonths)}$1`;
  const helperReciprocalRange = `_WPMonthHelper!$B$3:$${colLetter(1 + maxMonths)}$3`;
  const helperCountRange = `_WPMonthHelper!$B$2:$${colLetter(1 + maxMonths)}$2`;

  const equipmentExists = summary.equipment_detail.length > 0;
  const equipmentLastRow = 1 + summary.equipment_detail.length;
  const EQUIPMENT_ELIGIBLE_COL = 'D';

  const travelExists = summary.trip_detail.length > 0;
  const travelLastRow = 1 + summary.trip_detail.length;
  // Detail sheets lose their "Work Package(s)" column (column B) under Actual
  // Cost, which shifts every column after it one to the left.
  const wpColShift = isActualCost ? 1 : 0;
  const TRAVEL_TOTAL_COL = colLetter(5 - wpColShift);

  const otherCostExists = summary.other_cost_detail.length > 0;
  const otherCostLastRow = 1 + summary.other_cost_detail.length;
  const OTHER_COST_AMOUNT_COL = colLetter(3 - wpColShift);

  const subcontractingExists = summary.subcontracting_detail.length > 0;
  const subcontractingLastRow = 1 + summary.subcontracting_detail.length;
  const SUBCONTRACTING_AMOUNT_COL = colLetter(3 - wpColShift);

  // Lump Sum only: the Budget Summary's per-WP cells are live formulas over the
  // detail sheets. Equipment items belong to one WP (a "Work Package" column,
  // column H); Travel / Other Direct Costs / Subcontracting items are split
  // evenly across the WPs they're charged to, so each of those sheets gets, after
  // its own columns, a "# WPs charged" count column followed by one 1/0 flag
  // column per WP. Editing an amount or a flag flows straight into the summary.
  const EQUIPMENT_WP_COL = 'H';
  const TRAVEL_WP_COUNT_COL = 11;        // K — after Travel's 10 columns
  const OTHER_COST_WP_COUNT_COL = 6;     // F — after its 5 columns
  const SUBCONTRACTING_WP_COUNT_COL = 5; // E — after its 4 columns
  /** `SUM(amount x flag / # WPs)` — an item's even share of its amount in one WP. */
  const wpShareFormula = (sheet: string, amountCol: string, countColIdx: number, wpIdx: number, lastRow: number) => {
    const rng = (col: string) => `${sheet}!${col}2:${col}${lastRow}`;
    const count = rng(colLetter(countColIdx));
    // (count=0) keeps a no-WP item (e.g. the CFS item) from dividing by zero; its flag is 0 anyway.
    return `SUMPRODUCT(${rng(amountCol)}*${rng(colLetter(countColIdx + 1 + wpIdx))}/(${count}+(${count}=0)))`;
  };

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

  function addCategoryRow(
    label: string,
    wpKey: keyof WpBudgetDto | null,
    totalFormulaOrValue: number | string,
    // Live per-WP formula; when absent (or null) the backend's static figure is used.
    wpFormula?: (wp: WpBudgetDto, wpIdx: number) => string | null,
  ): ExcelJS.Row {
    const row = summarySheet.addRow([
      label,
      ...wpBudgets.map((wp, i) => {
        const formula = wpFormula?.(wp, i);
        if (formula) return { formula };
        return wpKey ? n(wp[wpKey] as string) : '';
      }),
      typeof totalFormulaOrValue === 'string' ? { formula: totalFormulaOrValue } : totalFormulaOrValue,
    ]);
    return row;
  }

  const aRow = addCategoryRow(
    'A  Personnel', 'personnel_eur',
    personnelExists ? `SUM(Personnel!${personnelTotalColLetter}${personnelFirstDataRow}:${personnelTotalColLetter}${personnelLastRow})` : 0,
    // Each WP's cell sums that WP's column on the Personnel sheet.
    personnelExists
      ? (_wp, i) => {
          const col = colLetter(personnelWpStartCol + i);
          return `SUM(Personnel!${col}${personnelFirstDataRow}:${col}${personnelLastRow})`;
        }
      : undefined,
  );
  const bRow = addCategoryRow(
    'B  Subcontracting', 'subcontracting_eur',
    subcontractingExists ? `SUM('Subcontracting'!${SUBCONTRACTING_AMOUNT_COL}2:${SUBCONTRACTING_AMOUNT_COL}${subcontractingLastRow})` : 0,
    subcontractingExists
      ? (_wp, i) => wpShareFormula("'Subcontracting'", SUBCONTRACTING_AMOUNT_COL, SUBCONTRACTING_WP_COUNT_COL, i, subcontractingLastRow)
      : undefined,
  );
  const c1Row = addCategoryRow(
    'C1 Travel', 'travel_eur',
    travelExists ? `SUM(Travel!${TRAVEL_TOTAL_COL}2:${TRAVEL_TOTAL_COL}${travelLastRow})` : 0,
    travelExists
      ? (_wp, i) => wpShareFormula('Travel', TRAVEL_TOTAL_COL, TRAVEL_WP_COUNT_COL, i, travelLastRow)
      : undefined,
  );
  const c2Row = addCategoryRow(
    'C2 Equipment', 'equipment_eur',
    equipmentExists ? `SUM(Equipment!${EQUIPMENT_ELIGIBLE_COL}2:${EQUIPMENT_ELIGIBLE_COL}${equipmentLastRow})` : 0,
    equipmentExists
      ? (wp) => `SUMIF(Equipment!$${EQUIPMENT_WP_COL}$2:$${EQUIPMENT_WP_COL}$${equipmentLastRow},${wp.work_package_id},Equipment!$${EQUIPMENT_ELIGIBLE_COL}$2:$${EQUIPMENT_ELIGIBLE_COL}$${equipmentLastRow})`
      : undefined,
  );
  const c3Row = addCategoryRow(
    'C3 Other Direct', 'other_costs_eur',
    otherCostExists ? `SUM('Other Direct Costs'!${OTHER_COST_AMOUNT_COL}2:${OTHER_COST_AMOUNT_COL}${otherCostLastRow})` : 0,
    otherCostExists
      ? (_wp, i) => wpShareFormula("'Other Direct Costs'", OTHER_COST_AMOUNT_COL, OTHER_COST_WP_COUNT_COL, i, otherCostLastRow)
      : undefined,
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

  if (config && !isActualCost) {
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
    // The month-overlap helper only exists to split cost across Work Packages.
    if (!isActualCost) {
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
    }

    // Second hidden helper sheet: one row per (role, project year). This is
    // where CALC-03/CALC-20a's Person-Months rounding rule — round each
    // role-year's Person-Months to 1 decimal (standard round-half-up,
    // matching the EU Funding & Tenders Portal) *before* pricing it — is
    // reproduced as formulas, referencing each role's own input cells on the
    // Personnel sheet (built below) by address.
    const yearHelperSheet = wb.addWorksheet('_WPYearHelper');
    yearHelperSheet.state = 'hidden';
    const yearHelperHeaders = [
      'Role', 'Year', 'Active Months', 'PM (rounded)', 'Salary (€)',
      'Annual Cost — CALC-03', 'Covered PM (unrounded)', 'Covered PM (rounded)',
      'Annual Cost from WPs — CALC-20a basis',
    ];
    // Columns G-I (the Work-Package split basis) don't exist under Actual Cost.
    yearHelperSheet.addRow(isActualCost ? yearHelperHeaders.slice(0, 6) : yearHelperHeaders);

    summary.role_detail.forEach((role, k) => {
      const roleRow = personnelFirstDataRow + k;
      const pStartCell = `Personnel!$F$${roleRow}`;
      const pEndCell = `Personnel!$G$${roleRow}`;
      const pFteCell = `Personnel!$E$${roleRow}`;
      const pIncreaseCell = `Personnel!$D$${roleRow}`;
      const pBaseMonthlyCell = `Personnel!$${colLetter(personnelBaseMonthlyCol)}$${roleRow}`;

      for (let year = 1; year <= durationYears; year++) {
        const yearStartMonth = (year - 1) * 12 + 1;
        const yearEndMonth = year * 12;
        const row = yearHelperSheet.addRow([role.role_label, year]);
        const rn = row.number;
        // Active months this year (CALC-03): overlap of [start,end] with the
        // year's [yearStart,yearEnd] month range, clamped to >= 0.
        row.getCell(3).value = {
          formula: `MAX(0,MIN(${pEndCell},${yearEndMonth})-MAX(${pStartCell},${yearStartMonth})+1)`,
        };
        row.getCell(4).value = { formula: `ROUND(C${rn}*${pFteCell},1)` };
        row.getCell(5).value = { formula: `${pBaseMonthlyCell}*(1+${pIncreaseCell}/100)^${year}` };
        row.getCell(6).value = { formula: `D${rn}*E${rn}` };
        if (!isActualCost) {
          // Covered PM (unrounded): the role's PM in months that at least one
          // WP covers — the basis CALC-20a rounds and splits across WPs. It is
          // less than the role's full active-month PM above only when some
          // months fall outside every WP (see the Unattributed column).
          // Each covered month counts ONCE (count row > 0), not weighted by
          // 1/overlap: that weighting is only for dividing a month between
          // WPs, and using it here under-counted every month two WPs share
          // (e.g. 4.25 PM instead of 6.0), which shrank the priced pool and
          // made the WP columns sum to more than the role's Total.
          row.getCell(7).value = {
            formula: `${pFteCell}*SUMPRODUCT((${helperMonthRange}>=${pStartCell})*(${helperMonthRange}<=${pEndCell})*(${helperMonthRange}>=${yearStartMonth})*(${helperMonthRange}<=${yearEndMonth})*(${helperCountRange}>0))`,
          };
          row.getCell(8).value = { formula: `ROUND(G${rn},1)` };
          row.getCell(9).value = { formula: `H${rn}*E${rn}` };
        }
      }
    });

    const persSheet = wb.addWorksheet('Personnel');
    persSheet.properties.defaultColWidth = 15;

    // WP Timelines reference table (rows 1-2 + one row per WP), extended with
    // a Duration column (End - Start + 1, inclusive of both ends — the same
    // inclusive convention the month-overlap formulas below use, so there's
    // no off-by-one drift between "how long is this WP" and "how many months
    // of it does each role get") and one Person-Months (PM) column per role.
    // The WP Timelines table, its Person-Months audit columns and the
    // reconciliation block are Lump-Sum-only: Actual Cost has no Work Packages.
    if (!isActualCost) {
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
      //
      // Displayed to 1 decimal place, matching the EU Funding & Tenders
      // Portal's own Person-Months display (standard round-half-up: a
      // second decimal of 5 or higher rounds the first decimal up, e.g.
      // 9.65 -> 9.7). This is display-only — the underlying formula keeps
      // full precision, so the Total PM / Employment Months reconciliation
      // below compares exact values and can't drift from per-WP rounding.
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
        cell.numFmt = '0.0';
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
      totalPmCell.numFmt = '0.0';

      const employmentCell = employmentRow.getCell(col);
      employmentCell.value = { formula: `(G${roleRow}-F${roleRow}+1)*E${roleRow}` };
      employmentCell.numFmt = '0.0';

      reconciledRow.getCell(col).value = {
        formula: `IF(${colLetterStr}${personnelWpTimelineTotalPmRow}=${colLetterStr}${personnelWpTimelineEmploymentRow},"OK","MISMATCH")`,
      };
    });

    persSheet.addRow([]);
    }

    // Each role's salary cell shows its own currency (a per-cell number
    // format), so one column can mix TRY and EUR roles.
    const headers = [
      'Role', 'Type', 'Current Salary', 'Annual Increase (%)', 'PM',
      'Start Month', 'End Month', 'Base Monthly (€)', 'Average Monthly (€)',
      ...(isActualCost ? ['Total PM'] : []),
      ...wpBudgets.map((wp) => `${wpLabel(wp)} (€)`),
      ...(isActualCost ? [] : ['Unattributed (€)']),
      'Total (€)',
    ];
    const ph = persSheet.addRow(headers);
    ph.font = { bold: true };

    summary.role_detail.forEach((role, k) => {
      const row = persSheet.addRow([
        role.role_label,
        role.role_type,
        n(role.current_monthly_salary_try),
        n(role.inflation_rate_pct),
        n(role.fte_fraction),
        role.start_month,
        role.end_month,
      ]);
      // Set per-cell, not via getColumn(5) below — column E is also the WP
      // Timelines table's first role's PM column further up this same
      // sheet, and a column-wide numFmt would silently overwrite that
      // cell's own '0.0' format with this one.
      row.getCell(5).numFmt = '0.00';
      const r = row.number;
      const salaryCell = `C${r}`;
      const fteCell = `E${r}`;
      const startCell = `F${r}`;
      const endCell = `G${r}`;

      // A TRY salary is converted at the project's TRY→EUR rate; a EUR salary
      // is already the calculation currency and is used as entered.
      const isEurSalary = role.salary_currency === 'EUR';
      row.getCell(3).numFmt = `#,##0.00 "${isEurSalary ? 'EUR' : 'TRY'}"`;
      row.getCell(personnelBaseMonthlyCol).value = {
        formula: isEurSalary ? salaryCell : `${salaryCell}/${tryRateCellRef}`,
      };

      const yearRoleFirstRow = yearHelperRowFor(k, 1);
      const yearRoleLastRow = yearHelperRowFor(k, durationYears);

      // Average Monthly (€): display only, doesn't feed any cost figure —
      // the plain average of this role's inflated monthly salary
      // (_WPYearHelper column E) across every project year, regardless of
      // which years the role is actually active in.
      row.getCell(personnelAverageMonthlyCol).value = {
        formula: `AVERAGE(_WPYearHelper!$E$${yearRoleFirstRow}:$E$${yearRoleLastRow})`,
      };

      // Total PM (Actual Cost only): the role's Person-Months over the whole
      // project — the sum of _WPYearHelper's per-year *rounded* PM (column D),
      // i.e. exactly the figure its cost is priced from, matching what the EU
      // portal shows (1 decimal, round-half-up per period).
      if (personnelTotalPmCol !== null) {
        row.getCell(personnelTotalPmCol).value = {
          formula: `SUM(_WPYearHelper!$D$${yearRoleFirstRow}:$D$${yearRoleLastRow})`,
        };
      }

      // Per-WP cost: a genuine formula, summed year by year against
      // _WPYearHelper so the Person-Months-rounding rule (CALC-03/CALC-20a)
      // is honored — for each project year, this role's WP-covered PM
      // (this WP's own month-overlap SUMPRODUCT, restricted to that year)
      // as a share of that year's rounded, priced pool
      // (_WPYearHelper column I). A year with no WP coverage contributes 0
      // rather than dividing by zero.
      wpBudgets.forEach((_wp, i) => {
        const wpTableRow = personnelWpTimelineFirstRow + i;
        const wpStartCell = `$B$${wpTableRow}`;
        const wpEndCell = `$C$${wpTableRow}`;
        const col = personnelWpStartCol + i;
        const terms: string[] = [];
        for (let year = 1; year <= durationYears; year++) {
          const hr = yearHelperRowFor(k, year);
          const yearStartMonth = (year - 1) * 12 + 1;
          const yearEndMonth = year * 12;
          const coveredPmThisWpYear =
            `(${fteCell}*SUMPRODUCT((${helperMonthRange}>=${startCell})*(${helperMonthRange}<=${endCell})` +
            `*(${helperMonthRange}>=${wpStartCell})*(${helperMonthRange}<=${wpEndCell})` +
            `*(${helperMonthRange}>=${yearStartMonth})*(${helperMonthRange}<=${yearEndMonth})*${helperReciprocalRange}))`;
          terms.push(
            `IF(_WPYearHelper!$G$${hr}=0,0,_WPYearHelper!$I$${hr}*${coveredPmThisWpYear}/_WPYearHelper!$G$${hr})`,
          );
        }
        row.getCell(col).value = { formula: terms.join('+') };
      });

      // Total (€): SUM of this role's per-year CALC-03 cost (_WPYearHelper
      // column F) — a genuine formula, independent of the WP split above,
      // matching the backend's own role-level total.
      row.getCell(personnelTotalCol).value = {
        formula: `SUM(_WPYearHelper!$F$${yearRoleFirstRow}:$F$${yearRoleLastRow})`,
      };

      // Any months of the role's Start/End period outside every WP timeline
      // aren't attributed to a WP bucket — surfaced here so the row still
      // reconciles to the authoritative Total. In the normal case (every
      // active month falls inside some WP) this is 0.
      const totalCellRef = `${colLetter(personnelTotalCol)}${r}`;
      if (isActualCost) {
        // No Work Packages, so nothing to attribute and no Unattributed column.
      } else if (wpBudgets.length > 0) {
        const wpRangeStart = `${colLetter(personnelWpStartCol)}${r}`;
        const wpRangeEnd = `${colLetter(personnelWpStartCol + wpBudgets.length - 1)}${r}`;
        row.getCell(personnelUnattributedCol).value = {
          formula: `${totalCellRef}-SUM(${wpRangeStart}:${wpRangeEnd})`,
        };
      } else {
        row.getCell(personnelUnattributedCol).value = { formula: totalCellRef };
      }
    });

    // Actual Cost: a project-level totals row, so the total Person-Months (and
    // total cost) can be read off directly. It sits below the roles and is
    // outside the range the Budget Summary sums, so nothing is double counted.
    if (isActualCost && personnelTotalPmCol !== null && summary.role_detail.length > 0) {
      const totalsRow = persSheet.addRow(['Total']);
      totalsRow.font = { bold: true };
      for (const col of [personnelTotalPmCol, personnelTotalCol]) {
        const letter = colLetter(col);
        totalsRow.getCell(col).value = {
          formula: `SUM(${letter}${personnelFirstDataRow}:${letter}${personnelLastRow})`,
        };
      }
    }

    // (Column C — the salary — is formatted per cell above, with its currency.)
    for (let col = personnelBaseMonthlyCol; col <= personnelTotalCol; col++) {
      persSheet.getColumn(col).numFmt = col === personnelTotalPmCol ? '0.0' : '#,##0.00';
    }
  }

  // ── Sheet 4: Equipment ────────────────────────────────────────────────────

  if (equipmentExists) {
    const eqSheet = wb.addWorksheet('Equipment');
    eqSheet.properties.defaultColWidth = 20;
    const eh = eqSheet.addRow([
      'Item', 'Theoretical (€)', 'Max (€)', 'Eligible Depreciation (€)', 'Capped?',
      'Purchase Cost (€)', 'Total Need an External Funding (€)',
      // Which WP the item's depreciation is charged to (a WP number the
      // Budget Summary's per-WP Equipment cells sum by); Lump Sum only.
      ...(isActualCost ? [] : ['Work Package']),
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
      if (!isActualCost) row.getCell(8).value = item.work_package_id;
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

  // Lump Sum: header cells + per-row cells for the "# WPs charged" count and the
  // one-flag-per-WP columns that the Budget Summary's per-WP formulas read.
  const wpAllocationHeaders = isActualCost ? [] : ['# WPs charged', ...wpBudgets.map((wp) => `${wpLabel(wp)} (1/0)`)];
  const writeWpAllocation = (row: ExcelJS.Row, countColIdx: number, wpIds: number[]) => {
    if (isActualCost || wpBudgets.length === 0) return;
    const first = colLetter(countColIdx + 1);
    const last = colLetter(countColIdx + wpBudgets.length);
    row.getCell(countColIdx).value = { formula: `SUM(${first}${row.number}:${last}${row.number})` };
    wpBudgets.forEach((wp, i) => {
      row.getCell(countColIdx + 1 + i).value = wpIds.includes(wp.work_package_id) ? 1 : 0;
    });
  };

  if (travelExists) {
    const travelSheet = wb.addWorksheet('Travel');
    travelSheet.properties.defaultColWidth = 18;
    const th = travelSheet.addRow([
      'Trip', ...(isActualCost ? [] : ['Work Package(s)']), 'Instances', 'Per Instance (€)', 'Total (€)',
      'Destination Country', 'Flight Cost (€)', 'Accommodation Cost (€)',
      'Subsistence Cost (€)', 'Domestic Transport Cost (€)',
      ...wpAllocationHeaders,
    ]);
    th.font = { bold: true };

    for (const trip of summary.trip_detail) {
      const tripRow = travelSheet.addRow([
        trip.name,
        ...(isActualCost ? [] : [wpTag(trip.work_package_ids)]),
        trip.number_of_instances,
        n(trip.per_instance_total_eur),
        n(trip.total_trip_cost_eur),
        countryName(trip.destination_country_code),
        trip.flight_cost_per_instance !== null ? n(trip.flight_cost_per_instance) : '',
        trip.accommodation_cost_per_instance !== null ? n(trip.accommodation_cost_per_instance) : '',
        trip.subsistence_cost_per_instance !== null ? n(trip.subsistence_cost_per_instance) : '',
        trip.domestic_transport_per_instance !== null ? n(trip.domestic_transport_per_instance) : '',
      ]);
      writeWpAllocation(tripRow, TRAVEL_WP_COUNT_COL, trip.work_package_ids);
    }
    for (const col of [4, 5, 7, 8, 9, 10]) {
      travelSheet.getColumn(col - wpColShift).numFmt = '#,##0.00';
    }
  }

  // ── Sheet 6: Other Direct Costs ───────────────────────────────────────────

  if (otherCostExists) {
    const ocSheet = wb.addWorksheet('Other Direct Costs');
    ocSheet.properties.defaultColWidth = 20;
    const oh = ocSheet.addRow(['Item', ...(isActualCost ? [] : ['Work Package(s)']), 'Amount (€)', 'CFS?', 'Notes', ...wpAllocationHeaders]);
    oh.font = { bold: true };

    for (const item of summary.other_cost_detail) {
      const itemRow = ocSheet.addRow([
        item.name,
        ...(isActualCost ? [] : [wpTag(item.work_package_ids)]),
        n(item.amount_eur),
        item.is_cfs_item ? 'Yes' : 'No',
        item.notes ?? '',
      ]);
      writeWpAllocation(itemRow, OTHER_COST_WP_COUNT_COL, item.work_package_ids);
    }
    ocSheet.getColumn(3 - wpColShift).numFmt = '#,##0.00';
  }

  // ── Sheet 7: Subcontracting ────────────────────────────────────────────────

  if (subcontractingExists) {
    const subSheet = wb.addWorksheet('Subcontracting');
    subSheet.properties.defaultColWidth = 20;
    const sh = subSheet.addRow(['Item', ...(isActualCost ? [] : ['Work Package(s)']), 'Amount (€)', 'Notes', ...wpAllocationHeaders]);
    sh.font = { bold: true };

    for (const item of summary.subcontracting_detail) {
      const itemRow = subSheet.addRow([
        item.name,
        ...(isActualCost ? [] : [wpTag(item.work_package_ids)]),
        n(item.amount_eur),
        item.notes ?? '',
      ]);
      writeWpAllocation(itemRow, SUBCONTRACTING_WP_COUNT_COL, item.work_package_ids);
    }
    subSheet.getColumn(3 - wpColShift).numFmt = '#,##0.00';
  }

  const filename = `${(config?.project_title ?? 'M2-EU-Budgeter').replace(/\s+/g, '_')}_Budget.xlsx`;
  await downloadWorkbook(wb, filename);
}
