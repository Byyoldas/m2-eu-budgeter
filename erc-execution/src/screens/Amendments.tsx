/**
 * Amendment Management — from-scratch design (see
 * erc-execution/src-tauri/src/domain/enums.rs `AmendmentType` doc comment for
 * why: development-roadmap.md names this module but no business rules,
 * DTOs, or UX exist for it anywhere else in the planning docs).
 */

import { useState } from 'react';
import { useExecutionStore } from '../store/executionStore';
import { useExecutionMutation } from '../hooks/useExecutionMutation';
import { recordAmendment, deleteAmendment } from '../ipc/commands';
import type { AmendmentInputDto, AmendmentType } from '../types';

const TYPE_LABELS: Record<AmendmentType, string> = {
  BudgetReallocation: 'Budget Reallocation',
  DurationExtension: 'Duration Extension',
  WorkPackageScopeChange: 'Work Package Scope Change',
  PersonnelChange: 'Personnel Change',
  Other: 'Other',
};

const emptyAmendment: AmendmentInputDto = {
  amendment_type: 'Other',
  title: '',
  description: '',
  requested_date: '',
  decision_date: null,
  status: 'Requested',
  financial_impact_eur: null,
  affected_work_package_ids: [],
  notes: null,
};

export function Amendments() {
  const summary = useExecutionStore((s) => s.summary);
  const { run, error, isSubmitting } = useExecutionMutation();
  const [form, setForm] = useState(emptyAmendment);

  if (!summary) return null;
  const { amendments } = summary;

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    const ok = await run(() => recordAmendment(form));
    if (ok) setForm(emptyAmendment);
  };

  return (
    <div className="screen">
      <h1>Amendment Management</h1>
      {error && <p className="error-banner">{error}</p>}

      <table>
        <thead>
          <tr>
            <th>#</th>
            <th>Type</th>
            <th>Title</th>
            <th>Requested</th>
            <th>Status</th>
            <th />
          </tr>
        </thead>
        <tbody>
          {amendments.map((a) => (
            <tr key={a.id}>
              <td>{a.amendment_number}</td>
              <td>{TYPE_LABELS[a.amendment_type]}</td>
              <td>{a.title}</td>
              <td>{a.requested_date}</td>
              <td>{a.status}</td>
              <td>
                <button onClick={() => run(() => deleteAmendment(a.id))} disabled={isSubmitting}>
                  Delete
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>

      <form onSubmit={submit} className="inline-form">
        <select
          value={form.amendment_type}
          onChange={(e) =>
            setForm({ ...form, amendment_type: e.target.value as AmendmentType })
          }
        >
          {Object.entries(TYPE_LABELS).map(([value, label]) => (
            <option key={value} value={value}>
              {label}
            </option>
          ))}
        </select>
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
        <input
          type="date"
          value={form.requested_date}
          onChange={(e) => setForm({ ...form, requested_date: e.target.value })}
          required
        />
        <button type="submit" disabled={isSubmitting}>
          Record Amendment
        </button>
      </form>
    </div>
  );
}
