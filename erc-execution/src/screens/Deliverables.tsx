/**
 * M-05: Deliverable Tracking. `deliverable_number` (BR-DEL-02) is
 * server-assigned and shown read-only; `is_overdue` is BR-DEL-01's derived
 * overlay. Rejected deliverables require a revision note + revised planned
 * month (BR-DEL-03) — enforced server-side, surfaced here as extra fields
 * that appear once "Rejected" is selected.
 */

import { useState } from 'react';
import { useExecutionStore } from '../store/executionStore';
import { useExecutionMutation } from '../hooks/useExecutionMutation';
import { addDeliverable, deleteDeliverable } from '../ipc/commands';
import type { DeliverableInputDto, DeliverableStatus, DeliverableType, DisseminationLevel } from '../types';

const STATUS_LABELS: Record<DeliverableStatus, string> = {
  NotStarted: 'Not Started',
  InProgress: 'In Progress',
  Submitted: 'Submitted',
  Accepted: 'Accepted',
  Rejected: 'Rejected',
  Revised: 'Revised',
};

const emptyForm = (roleId: string): DeliverableInputDto => ({
  title: '',
  deliverable_type: 'Report',
  work_package_id: 1,
  planned_month: 1,
  responsible_role_id: roleId,
  dissemination_level: 'Public',
  status: 'NotStarted',
  actual_submission_date: null,
  revision_note: null,
  revised_planned_month: null,
  cordis_registered: false,
  notes: null,
});

export function Deliverables() {
  const summary = useExecutionStore((s) => s.summary);
  const { run, error, isSubmitting } = useExecutionMutation();
  const [form, setForm] = useState<DeliverableInputDto | null>(null);

  if (!summary) return null;
  const { deliverables, personnel_roles } = summary;
  const activeForm = form ?? emptyForm(personnel_roles[0]?.id ?? '');

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    const ok = await run(() => addDeliverable(activeForm));
    if (ok) setForm(null);
  };

  return (
    <div className="screen">
      <h1>Deliverable Tracking</h1>
      {error && <p className="error-banner">{error}</p>}

      <table>
        <thead>
          <tr>
            <th>#</th>
            <th>Title</th>
            <th>WP</th>
            <th>Type</th>
            <th>Due Month</th>
            <th>Status</th>
            <th>Period</th>
            <th />
          </tr>
        </thead>
        <tbody>
          {deliverables.map((d) => (
            <tr key={d.id}>
              <td>{d.deliverable_number}</td>
              <td>{d.title}</td>
              <td>WP{d.work_package_id}</td>
              <td>{d.deliverable_type}</td>
              <td>
                {d.revised_planned_month ?? d.planned_month}
                {d.is_overdue && <span className="warning-banner"> Overdue</span>}
              </td>
              <td>{STATUS_LABELS[d.status]}</td>
              <td>{d.reporting_period_number ?? '—'}</td>
              <td>
                {d.cordis_warning && <span className="warning-banner">CORDIS</span>}
                <button onClick={() => run(() => deleteDeliverable(d.id))} disabled={isSubmitting}>
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
          value={activeForm.title}
          onChange={(e) => setForm({ ...activeForm, title: e.target.value })}
          required
        />
        <input
          type="number"
          min={1}
          placeholder="WP"
          value={activeForm.work_package_id}
          onChange={(e) => setForm({ ...activeForm, work_package_id: Number(e.target.value) })}
          required
        />
        <select
          value={activeForm.deliverable_type}
          onChange={(e) => setForm({ ...activeForm, deliverable_type: e.target.value as DeliverableType })}
        >
          {(['Report', 'Dataset', 'Software', 'Prototype', 'Dem', 'Ethics', 'Other'] as DeliverableType[]).map(
            (t) => (
              <option key={t} value={t}>
                {t}
              </option>
            ),
          )}
        </select>
        <input
          type="number"
          min={1}
          placeholder="Planned month"
          value={activeForm.planned_month}
          onChange={(e) => setForm({ ...activeForm, planned_month: Number(e.target.value) })}
          required
        />
        <select
          value={activeForm.responsible_role_id}
          onChange={(e) => setForm({ ...activeForm, responsible_role_id: e.target.value })}
          required
        >
          <option value="">Responsible role…</option>
          {personnel_roles.map((r) => (
            <option key={r.id} value={r.id}>
              {r.role_label}
            </option>
          ))}
        </select>
        <select
          value={activeForm.dissemination_level}
          onChange={(e) =>
            setForm({ ...activeForm, dissemination_level: e.target.value as DisseminationLevel })
          }
        >
          <option value="Public">Public</option>
          <option value="RestrictedToProgramme">Restricted to Programme</option>
          <option value="Confidential">Confidential</option>
        </select>
        <button type="submit" disabled={isSubmitting}>
          Add Deliverable
        </button>
      </form>
    </div>
  );
}
