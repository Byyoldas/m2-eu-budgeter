/**
 * M-06: Milestone Tracking. `effective_status` (not `status`) is what's
 * displayed — it includes BR-MS-01's automatic "At Risk" overlay.
 */

import { useState } from 'react';
import { useExecutionStore } from '../store/executionStore';
import { useExecutionMutation } from '../hooks/useExecutionMutation';
import { addMilestone, completeMilestone, deleteMilestone } from '../ipc/commands';
import type { MilestoneInputDto } from '../types';

const STATUS_LABELS: Record<string, string> = {
  NotStarted: 'Not Started',
  OnTrack: 'On Track',
  AtRisk: 'At Risk',
  Delayed: 'Delayed',
  Completed: 'Completed',
  Cancelled: 'Cancelled',
};

const emptyMilestone: MilestoneInputDto = {
  title: '',
  work_package_id: 1,
  planned_month: 1,
  status: 'NotStarted',
  actual_completion_month: null,
  linked_deliverable_ids: [],
};

export function Milestones() {
  const summary = useExecutionStore((s) => s.summary);
  const { run, error, isSubmitting } = useExecutionMutation();
  const [form, setForm] = useState(emptyMilestone);

  if (!summary) return null;
  const { milestones } = summary;

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    const ok = await run(() => addMilestone(form));
    if (ok) setForm(emptyMilestone);
  };

  return (
    <div className="screen">
      <h1>Milestone Tracking</h1>
      {error && <p className="error-banner">{error}</p>}

      <table>
        <thead>
          <tr>
            <th>Title</th>
            <th>WP</th>
            <th>Planned Month</th>
            <th>Status</th>
            <th>Actual Completion</th>
            <th />
          </tr>
        </thead>
        <tbody>
          {milestones.map((m) => (
            <tr key={m.id}>
              <td>{m.title}</td>
              <td>WP{m.work_package_id}</td>
              <td>{m.planned_month}</td>
              <td>{STATUS_LABELS[m.effective_status] ?? m.effective_status}</td>
              <td>{m.actual_completion_month ?? '—'}</td>
              <td>
                {m.effective_status !== 'Completed' && (
                  <button onClick={() => run(() => completeMilestone(m.id))} disabled={isSubmitting}>
                    Complete
                  </button>
                )}
                <button onClick={() => run(() => deleteMilestone(m.id))} disabled={isSubmitting}>
                  Delete
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>

      <form onSubmit={submit} className="inline-form">
        <input
          placeholder="Title"
          value={form.title}
          onChange={(e) => setForm({ ...form, title: e.target.value })}
          required
        />
        <input
          type="number"
          min={1}
          placeholder="WP"
          value={form.work_package_id}
          onChange={(e) => setForm({ ...form, work_package_id: Number(e.target.value) })}
          required
        />
        <input
          type="number"
          min={1}
          placeholder="Planned month"
          value={form.planned_month}
          onChange={(e) => setForm({ ...form, planned_month: Number(e.target.value) })}
          required
        />
        <button type="submit" disabled={isSubmitting}>
          Add Milestone
        </button>
      </form>
    </div>
  );
}
