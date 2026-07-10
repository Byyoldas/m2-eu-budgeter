/**
 * Stacked bar chart: per-year budget breakdown by category.
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
  E: '#6b7280',
};

function eur(v: string | number): number {
  return typeof v === 'number' ? v : parseFloat(v) || 0;
}

export function BudgetYearBarChart() {
  const summary = useSummary();
  if (!summary) return null;

  // Build per-year data
  const yearMap: Record<number, Record<string, number>> = {};

  for (const y of summary.category_a_by_year) {
    yearMap[y.year] = { ...(yearMap[y.year] ?? {}), A: eur(y.amount_eur) };
  }
  for (const y of summary.category_c1_by_year) {
    yearMap[y.year] = { ...(yearMap[y.year] ?? {}), C1: eur(y.amount_eur) };
  }
  for (const y of summary.category_c3_by_year) {
    yearMap[y.year] = { ...(yearMap[y.year] ?? {}), C3: eur(y.amount_eur) };
  }
  for (const y of summary.category_e_by_year) {
    yearMap[y.year] = { ...(yearMap[y.year] ?? {}), E: eur(y.amount_eur) };
  }

  // Distribute C2 evenly across years
  const years = Object.keys(yearMap).map(Number).sort();
  if (years.length > 0) {
    const c2PerYear = eur(summary.category_c2_total) / years.length;
    for (const yr of years) {
      yearMap[yr].C2 = c2PerYear;
    }
  }

  const data = years.map((yr) => ({
    year: `Year ${yr}`,
    ...yearMap[yr],
  }));

  if (data.length === 0) return null;

  const formatEur = (v: number) =>
    `€${(v / 1000).toFixed(0)}k`;

  return (
    <div className="chart-container">
      <h4 className="chart-title">Budget by Year</h4>
      <ResponsiveContainer width="100%" height={200}>
        <BarChart data={data} margin={{ top: 4, right: 8, left: 0, bottom: 4 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="#1e293b" />
          <XAxis dataKey="year" tick={{ fontSize: 11, fill: '#94a3b8' }} />
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
          <Bar dataKey="E" stackId="s" fill={COLORS.E} name="Indirect" radius={[4, 4, 0, 0]} />
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}
