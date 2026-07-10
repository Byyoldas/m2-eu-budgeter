/**
 * Right-panel budget summary — category totals and grand total.
 * Updates in real time after every mutation.
 */

import { useSummary } from '../store/projectStore';

function fmt(v: string | undefined): string {
  if (!v) return '€ 0.00';
  const n = parseFloat(v);
  return isNaN(n) ? '€ 0.00' : `€ ${n.toLocaleString('en-GB', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

export function CategoryTotalsPanel() {
  const summary = useSummary();

  if (!summary) {
    return (
      <div className="totals-panel totals-panel--empty">
        <p>Create or open a project to see the budget summary.</p>
      </div>
    );
  }

  const rows = [
    { label: 'A  Personnel', value: summary.category_a_total, accent: 'cat-a' },
    { label: 'B  Subcontracting', value: summary.category_b_total, accent: 'cat-b' },
    { label: 'C1 Travel', value: summary.category_c1_total, accent: 'cat-c1' },
    { label: 'C2 Equipment', value: summary.category_c2_total, accent: 'cat-c2' },
    { label: 'C3 Other Direct Costs', value: summary.category_c3_total, accent: 'cat-c3' },
    { label: 'E  Indirect Costs (25%)', value: summary.category_e_total, accent: 'cat-e' },
  ] as const;

  return (
    <div className="totals-panel">
      <h3 className="totals-heading">Budget Summary</h3>

      <div className="totals-rows">
        {rows.map(({ label, value, accent }) => (
          <div key={label} className={`totals-row totals-row--${accent}`}>
            <span className="totals-row-label">{label}</span>
            <span className="totals-row-value">{fmt(value)}</span>
          </div>
        ))}
      </div>

      <div className="totals-separator" />

      <div className="totals-grand">
        <span className="totals-grand-label">Total Eligible Costs</span>
        <span className="totals-grand-value">{fmt(summary.total_eligible_costs)}</span>
      </div>

      <div className="totals-eu">
        <span className="totals-eu-label">EU Contribution Requested</span>
        <span className="totals-eu-value">{fmt(summary.requested_eu_contribution)}</span>
      </div>

      {summary.cfs_prompt_required && (
        <div className="totals-cfs-warning">
          ⚠ Budget exceeds €430,000. A Certificate on Financial Statements is required.
        </div>
      )}
    </div>
  );
}
