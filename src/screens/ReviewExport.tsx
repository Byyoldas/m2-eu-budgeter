/**
 * Step 8 — Review & Export.
 * Full budget summary, per-year breakdown, and export actions.
 */

import { useState } from 'react';
import { save } from '@tauri-apps/plugin-dialog';
import { saveProjectAs } from '../ipc/commands';
import { useProjectStore, useSummary } from '../store/projectStore';
import { BudgetWpBarChart } from '../components/BudgetWpBarChart';
import { BudgetRingChart } from '../components/BudgetRingChart';
import { exportToExcel } from '../export/excelExporter';
import { exportToCsv } from '../export/csvExporter';
import type { AppError } from '../types';

interface ReviewExportProps {
  onBack: () => void;
}

function fmt(v: string | undefined): string {
  if (!v) return '€ 0.00';
  const n = parseFloat(v);
  return isNaN(n) ? '€ 0.00' : `€ ${n.toLocaleString('en-GB', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

export function ReviewExport({ onBack }: ReviewExportProps) {
  const summary = useSummary();
  const projectConfig = useProjectStore((s) => s.projectConfig);
  const countries = useProjectStore((s) => s.countries);
  const projectPath = useProjectStore((s) => s.projectPath);
  const setProjectPath = useProjectStore((s) => s.setProjectPath);
  const setDirty = useProjectStore((s) => s.setDirty);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [exportStatus, setExportStatus] = useState<string | null>(null);

  if (!summary) return null;

  const handleSaveAs = async () => {
    try {
      const path = await save({
        filters: [{ name: 'M2-EU Budgeter File', extensions: ['ercbudget'] }],
        defaultPath: `${projectConfig?.project_title ?? 'm2-eu-budgeter'}.ercbudget`,
      });
      if (!path) return;
      await saveProjectAs(path);
      setProjectPath(path);
      setDirty(false);
      setSaveError(null);
    } catch (err) {
      setSaveError((err as AppError).kind === 'Persistence'
        ? (err as { kind: string; detail: string }).detail
        : 'Failed to save file.');
    }
  };

  const handleExcelExport = async () => {
    try {
      setExportStatus('Generating Excel…');
      await exportToExcel(summary, projectConfig, countries);
      setExportStatus('Excel export complete.');
    } catch {
      setExportStatus('Excel export failed.');
    }
  };

  const handleCsvExport = async () => {
    try {
      setExportStatus('Generating CSV…');
      await exportToCsv(summary, projectConfig);
      setExportStatus('CSV export complete.');
    } catch {
      setExportStatus('CSV export failed.');
    }
  };

  const categories: { label: string; total: string; key: keyof typeof summary.wp_budgets[number] | null }[] = [
    { label: 'A  Personnel', total: summary.category_a_total, key: 'personnel_eur' },
    { label: 'B  Subcontracting', total: summary.category_b_total, key: 'subcontracting_eur' },
    { label: 'C1 Travel', total: summary.category_c1_total, key: 'travel_eur' },
    { label: 'C2 Equipment', total: summary.category_c2_total, key: 'equipment_eur' },
    { label: 'C3 Other Direct', total: summary.category_c3_total, key: 'other_costs_eur' },
    { label: 'E  Indirect (25%)', total: summary.category_e_total, key: null },
  ];

  const wpBudgets = summary.wp_budgets;

  return (
    <div className="screen">
      <div className="screen-header">
        <h2 className="screen-title">Review & Export</h2>
        <div className="export-actions">
          <button className="btn btn--ghost" onClick={handleSaveAs}>
            💾 {projectPath ? 'Save As…' : 'Save Project…'}
          </button>
          <button className="btn btn--ghost" onClick={handleExcelExport}>📊 Export Excel</button>
          <button className="btn btn--ghost" onClick={handleCsvExport}>📄 Export CSV</button>
        </div>
      </div>

      {saveError && <div className="error-banner">{saveError}</div>}
      {exportStatus && <div className="info-banner">{exportStatus}</div>}

      {/* CFS Status */}
      {summary.cfs_threshold_exceeded && (
        <div className={`cfs-status-badge cfs-status-badge--${summary.cfs_status === 'REQUIRED_AND_PRESENT' ? 'ok' : 'warn'}`}>
          {summary.cfs_status === 'REQUIRED_AND_PRESENT'
            ? '✓ Certificate on Financial Statements included'
            : '⚠ Certificate on Financial Statements required but not yet added'}
        </div>
      )}

      {/* Summary table */}
      <div className="review-table-card">
        <h3 className="review-section-title">Budget Summary</h3>
        <table className="review-table">
          <thead>
            <tr>
              <th>Category</th>
              {wpBudgets.map((wp) => (
                <th key={wp.work_package_id}>{wp.work_package_name || `WP${wp.work_package_id}`}</th>
              ))}
              <th>Total</th>
            </tr>
          </thead>
          <tbody>
            {categories.map(({ label, total, key }) => (
              <tr key={label}>
                <td>{label}</td>
                {wpBudgets.map((wp) => (
                  <td key={wp.work_package_id}>{key ? fmt(wp[key] as string) : '—'}</td>
                ))}
                <td><strong>{fmt(total)}</strong></td>
              </tr>
            ))}
            <tr className="review-table-wp-total">
              <td><strong>Total</strong></td>
              {wpBudgets.map((wp) => (
                <td key={wp.work_package_id}><strong>{fmt(wp.total_eur)}</strong></td>
              ))}
              <td>
                <strong>
                  {fmt(wpBudgets.reduce((sum, wp) => sum + parseFloat(wp.total_eur), 0).toString())}
                </strong>
              </td>
            </tr>
          </tbody>
          <tfoot>
            <tr className="review-table-divider">
              <td colSpan={wpBudgets.length + 2} />
            </tr>
            <tr>
              <td><strong>Total Direct Costs</strong></td>
              {wpBudgets.map((wp) => <td key={wp.work_package_id} />)}
              <td><strong>{fmt(summary.total_direct_costs)}</strong></td>
            </tr>
            <tr>
              <td><strong>Total Eligible Costs</strong></td>
              {wpBudgets.map((wp) => <td key={wp.work_package_id} />)}
              <td><strong>{fmt(summary.total_eligible_costs)}</strong></td>
            </tr>
            <tr className="review-table-grand">
              <td><strong>Requested EU Contribution</strong></td>
              {wpBudgets.map((wp) => <td key={wp.work_package_id} />)}
              <td><strong>{fmt(summary.requested_eu_contribution)}</strong></td>
            </tr>
          </tfoot>
        </table>
      </div>

      {/* Charts */}
      <div className="review-charts">
        <BudgetWpBarChart />
        <BudgetRingChart />
      </div>

      {/* Personnel detail */}
      {summary.role_detail.length > 0 && (
        <div className="review-detail-card">
          <h3 className="review-section-title">Personnel Detail — Cost by Work Package</h3>
          <table className="review-table">
            <thead>
              <tr>
                <th>Role</th>
                <th>Type</th>
                <th>FTE</th>
                {wpBudgets.map((wp) => (
                  <th key={wp.work_package_id}>{wp.work_package_name || `WP${wp.work_package_id}`}</th>
                ))}
                <th>Total Cost</th>
              </tr>
            </thead>
            <tbody>
              {summary.role_detail.map((r) => (
                <tr key={r.id}>
                  <td>{r.role_label}</td>
                  <td>{r.role_type}</td>
                  <td>{parseFloat(r.fte_fraction).toFixed(2)}</td>
                  {wpBudgets.map((wp) => {
                    const amount = r.wp_breakdown.find((w) => w.work_package_id === wp.work_package_id);
                    return <td key={wp.work_package_id}>{amount ? fmt(amount.amount_eur) : '—'}</td>;
                  })}
                  <td><strong>{fmt(r.total_cost_eur)}</strong></td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Equipment detail */}
      {summary.equipment_detail.length > 0 && (
        <div className="review-detail-card">
          <h3 className="review-section-title">Equipment Detail</h3>
          <table className="review-table">
            <thead>
              <tr>
                <th>Item</th>
                <th>Eligible Depreciation</th>
                <th>Capped</th>
              </tr>
            </thead>
            <tbody>
              {summary.equipment_detail.map((e) => (
                <tr key={e.id}>
                  <td>{e.name}</td>
                  <td>{fmt(e.eligible_depreciation_eur)}</td>
                  <td>{e.is_capped ? 'Yes' : 'No'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      <div className="screen-footer">
        <button className="btn btn--ghost" onClick={onBack}>← Back</button>
      </div>
    </div>
  );
}
