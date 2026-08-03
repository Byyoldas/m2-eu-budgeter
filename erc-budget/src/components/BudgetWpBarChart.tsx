/**
 * Stacked bar chart: per-Work-Package budget breakdown by category.
 * Uses Recharts.
 */

import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';
import { useSummary } from '../store/projectStore';

const COLORS = {
  A: '#3b82f6',
  C1: '#10b981',
  C2: '#f59e0b',
  C3: '#8b5cf6',
  B: '#ef4444',
};

function eur(v: string | number): number {
  return typeof v === 'number' ? v : parseFloat(v) || 0;
}

export function BudgetWpBarChart() {
  const summary = useSummary();
  if (!summary || summary.wp_budgets.length === 0) return null;

  const data = summary.wp_budgets.map((wp) => ({
    wp: wp.work_package_name || `WP${wp.work_package_id}`,
    A: eur(wp.personnel_eur),
    C1: eur(wp.travel_eur),
    C2: eur(wp.equipment_eur),
    C3: eur(wp.other_costs_eur),
    B: eur(wp.subcontracting_eur),
  }));

  const formatEur = (v: number) =>
    `€${(v / 1000).toFixed(0)}k`;

  return (
    <div className="chart-container">
      <h4 className="chart-title">Budget by Work Package</h4>
      <ResponsiveContainer width="100%" height={200}>
        <BarChart data={data} margin={{ top: 4, right: 8, left: 0, bottom: 4 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="#1e293b" />
          <XAxis dataKey="wp" tick={{ fontSize: 11, fill: '#94a3b8' }} />
          <YAxis tickFormatter={formatEur} tick={{ fontSize: 11, fill: '#94a3b8' }} />
          <Tooltip
            formatter={(v: number) => `€${v.toLocaleString('en-GB', { maximumFractionDigits: 0 })}`}
            contentStyle={{ background: '#1e293b', border: 'none', fontSize: 12 }}
          />
          <Legend wrapperStyle={{ fontSize: 11 }} />
          <Bar dataKey="A" stackId="s" fill={COLORS.A} name="Personnel" />
          <Bar dataKey="C1" stackId="s" fill={COLORS.C1} name="Travel" />
          <Bar dataKey="C2" stackId="s" fill={COLORS.C2} name="Equipment" />
          <Bar dataKey="C3" stackId="s" fill={COLORS.C3} name="Other" />
          <Bar dataKey="B" stackId="s" fill={COLORS.B} name="Subcontracting" radius={[4, 4, 0, 0]} />
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}
