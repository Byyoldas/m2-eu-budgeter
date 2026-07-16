/**
 * Gantt-style chart: each Work Package as a horizontal bar spanning its active years.
 * Built on Recharts' vertical BarChart using the standard "invisible offset + visible
 * duration" stacked-bar idiom, since Recharts has no native range/Gantt bar type.
 */

import { BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, Cell, LabelList } from 'recharts';

const WP_COLORS = [
  '#3b82f6', '#10b981', '#f59e0b', '#8b5cf6', '#ef4444',
  '#06b6d4', '#eab308', '#ec4899', '#84cc16', '#6b7280',
];

interface WorkPackageGanttChartProps {
  names: (string | null | undefined)[];
  startMonths: number[];
  endMonths: number[];
  durationMonths: number;
}

interface GanttRow {
  wp: string;
  offset: number;
  span: number;
  start: number;
  end: number;
}

function renderRangeLabel(props: unknown, rows: GanttRow[]) {
  const { x, y, width, height, index } = props as { x: number; y: number; width: number; height: number; index: number };
  const row = rows[index];
  if (!row || width < 24) return null;
  const label = row.start === row.end ? `M${row.start}` : `M${row.start}–M${row.end}`;
  return (
    <text
      x={x + width / 2}
      y={y + height / 2}
      textAnchor="middle"
      dominantBaseline="middle"
      fontSize={11}
      fontWeight={600}
      fill="#0f172a"
    >
      {label}
    </text>
  );
}

export function WorkPackageGanttChart({ names, startMonths, endMonths, durationMonths }: WorkPackageGanttChartProps) {
  if (durationMonths < 1 || names.length === 0) return null;

  const rows: GanttRow[] = names.map((name, i) => {
    const rawStart = startMonths[i] ?? 1;
    const rawEnd = endMonths[i] ?? durationMonths;
    const start = Math.min(Math.max(1, rawStart), durationMonths);
    const end = Math.min(Math.max(start, rawEnd), durationMonths);
    return {
      wp: name?.trim() ? name : `WP${i + 1}`,
      offset: start - 1,
      span: end - start + 1,
      start,
      end,
    };
  });

  // Tick every 12 months (one per project year) so the chart stays readable.
  const yearTicks = Array.from({ length: Math.ceil(durationMonths / 12) }, (_, i) => i * 12);

  return (
    <div className="chart-container">
      <h4 className="chart-title">Work Package Timeline</h4>
      <ResponsiveContainer width="100%" height={Math.max(120, rows.length * 44)}>
        <BarChart data={rows} layout="vertical" margin={{ top: 4, right: 16, left: 8, bottom: 4 }}>
          <XAxis
            type="number"
            domain={[0, durationMonths]}
            ticks={yearTicks}
            tickFormatter={(v: number) => `Year ${v / 12 + 1}`}
            tick={{ fontSize: 11, fill: '#94a3b8' }}
          />
          <YAxis type="category" dataKey="wp" width={110} tick={{ fontSize: 11, fill: '#94a3b8' }} />
          <Tooltip
            formatter={(_value: number, _name: string, entry: { payload?: GanttRow }) => {
              const row = entry.payload;
              if (!row) return ['', 'Active'];
              const label = row.start === row.end ? `Year ${row.start}` : `Year ${row.start}–${row.end}`;
              return [label, 'Active'];
            }}
            contentStyle={{ background: '#1e293b', border: 'none', fontSize: 12 }}
          />
          <Bar dataKey="offset" stackId="wp" fill="transparent" isAnimationActive={false} />
          <Bar dataKey="span" stackId="wp" radius={[4, 4, 4, 4]} isAnimationActive={false}>
            {rows.map((_, i) => (
              <Cell key={i} fill={WP_COLORS[i % WP_COLORS.length]} />
            ))}
            <LabelList dataKey="span" content={(props) => renderRangeLabel(props, rows)} />
          </Bar>
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}
