/**
 * M-12: Risk Register. `risk_score`/`priority` (BR-RK-01/02) are derived
 * server-side, never entered directly. High-priority risks require a review
 * date within 30 days (BR-RK-03); `Closed` risks are terminal (BR-RK-04) —
 * both enforced server-side.
 */

import { useState } from 'react';
import { useExecutionStore } from '../store/executionStore';
import { useExecutionMutation } from '../hooks/useExecutionMutation';
import { addRiskEntry, deleteRiskEntry } from '../ipc/commands';
import type { Level, RiskEntryInputDto } from '../types';

const LEVELS: Level[] = ['Low', 'Medium', 'High'];

const PRIORITY_ICON: Record<Level, string> = {
  High: '🔴',
  Medium: '🟡',
  Low: '🟢',
};

const emptyForm: RiskEntryInputDto = {
  title: '',
  description: '',
  work_package_id: null,
  probability: 'Low',
  impact: 'Low',
  mitigation: null,
  status: 'Open',
  owner_role_id: null,
  identified_date: '',
  review_date: null,
  closed_date: null,
};

export function RiskRegister() {
  const summary = useExecutionStore((s) => s.summary);
  const { run, error, isSubmitting } = useExecutionMutation();
  const [form, setForm] = useState(emptyForm);
  const [view, setView] = useState<'list' | 'matrix'>('list');

  if (!summary) return null;
  const { risks } = summary;

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    const ok = await run(() => addRiskEntry(form));
    if (ok) setForm(emptyForm);
  };

  const matrixCount = (impact: Level, probability: Level) =>
    risks.filter((r) => r.impact === impact && r.probability === probability && r.status !== 'Closed').length;

  const sorted = [...risks].sort((a, b) => b.risk_score - a.risk_score);

  return (
    <div className="screen">
      <h1>Risk Register</h1>
      {error && <p className="error-banner">{error}</p>}

      <div className="inline-form">
        <button onClick={() => setView('list')} disabled={view === 'list'}>
          List
        </button>
        <button onClick={() => setView('matrix')} disabled={view === 'matrix'}>
          Matrix
        </button>
      </div>

      {view === 'matrix' ? (
        <table>
          <thead>
            <tr>
              <th>Probability \ Impact</th>
              {LEVELS.map((impact) => (
                <th key={impact}>{impact}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {[...LEVELS].reverse().map((probability) => (
              <tr key={probability}>
                <td>{probability}</td>
                {LEVELS.map((impact) => (
                  <td key={impact}>{matrixCount(impact, probability) || '—'}</td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      ) : (
        <table>
          <thead>
            <tr>
              <th>Score</th>
              <th>Title</th>
              <th>WP</th>
              <th>Status</th>
              <th>Review Date</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {sorted.map((r) => (
              <tr key={r.id}>
                <td>
                  {PRIORITY_ICON[r.priority]} {r.risk_score}
                </td>
                <td>{r.title}</td>
                <td>{r.work_package_id ? `WP${r.work_package_id}` : '—'}</td>
                <td>{r.status}</td>
                <td>{r.review_date ?? '—'}</td>
                <td>
                  <button onClick={() => run(() => deleteRiskEntry(r.id))} disabled={isSubmitting}>
                    Delete
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      <form onSubmit={submit} className="inline-form">
        <input
          placeholder="Title"
          value={form.title}
          onChange={(e) => setForm({ ...form, title: e.target.value })}
          required
        />
        <input
          placeholder="Description"
          value={form.description}
          onChange={(e) => setForm({ ...form, description: e.target.value })}
          required
        />
        <select
          value={form.probability}
          onChange={(e) => setForm({ ...form, probability: e.target.value as Level })}
        >
          {LEVELS.map((l) => (
            <option key={l} value={l}>
              Probability: {l}
            </option>
          ))}
        </select>
        <select
          value={form.impact}
          onChange={(e) => setForm({ ...form, impact: e.target.value as Level })}
        >
          {LEVELS.map((l) => (
            <option key={l} value={l}>
              Impact: {l}
            </option>
          ))}
        </select>
        <input
          type="date"
          value={form.identified_date}
          onChange={(e) => setForm({ ...form, identified_date: e.target.value })}
          required
        />
        <input
          type="date"
          placeholder="Review date"
          value={form.review_date ?? ''}
          onChange={(e) => setForm({ ...form, review_date: e.target.value || null })}
        />
        <button type="submit" disabled={isSubmitting}>
          Add Risk
        </button>
      </form>
    </div>
  );
}
