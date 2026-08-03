/**
 * Donut chart: category share of total eligible budget.
 */

import { PieChart, Pie, Cell, Tooltip, ResponsiveContainer } from 'recharts';
import { useSummary } from '../store/projectStore';

const SLICES = [
  { key: 'category_a_total', label: 'A Personnel', color: '#3b82f6' },
  { key: 'category_b_total', label: 'B Subcontracting', color: '#ef4444' },
  { key: 'category_c1_total', label: 'C1 Travel', color: '#10b981' },
  { key: 'category_c2_total', label: 'C2 Equipment', color: '#f59e0b' },
  { key: 'category_c3_total', label: 'C3 Other', color: '#8b5cf6' },
  { key: 'category_e_total', label: 'E Indirect', color: '#6b7280' },
] as const;

type SummaryKey = typeof SLICES[number]['key'];

function eur(v: string): number {
  return parseFloat(v) || 0;
}

export function BudgetRingChart() {
  const summary = useSummary();
  if (!summary) return null;

  const data = SLICES
    .map(({ key, label, color }) => ({
      name: label,
      value: eur(summary[key as SummaryKey] as string),
      color,
    }))
    .filter((d) => d.value > 0);

  if (data.length === 0) return null;

  const total = eur(summary.total_eligible_costs);

  return (
    <div className="chart-container">
      <h4 className="chart-title">Category Split</h4>
      <ResponsiveContainer width="100%" height={200}>
        <PieChart>
          <Pie
            data={data}
            dataKey="value"
            nameKey="name"
            cx="50%"
            cy="50%"
            innerRadius={55}
            outerRadius={85}
            paddingAngle={2}
          >
            {data.map((d) => (
              <Cell key={d.name} fill={d.color} />
            ))}
          </Pie>
          <Tooltip
            formatter={(v: number) => [
              `€${v.toLocaleString('en-GB', { minimumFractionDigits: 0, maximumFractionDigits: 0 })}`,
              `${((v / total) * 100).toFixed(1)}%`,
            ]}
            contentStyle={{ background: '#1e293b', border: 'none', fontSize: 12 }}
          />
        </PieChart>
      </ResponsiveContainer>
    </div>
  );
}
