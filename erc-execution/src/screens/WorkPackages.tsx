/**
 * M-04: Work Package Management. WPs themselves are read-only (from the
 * Budget App); this screen shows derived status + planned-vs-actual and lets
 * the user assign a leader / add notes.
 */

import { useState } from 'react';
import { useExecutionStore } from '../store/executionStore';
import { useExecutionMutation } from '../hooks/useExecutionMutation';
import { setWorkPackageExecution } from '../ipc/commands';
import type { WorkPackageExecutionInputDto } from '../types';
import { fmtEur } from '../utils/currency';

const STATUS_LABELS: Record<string, string> = {
  NotStarted: 'Not Started',
  OnTrack: 'On Track',
  AtRisk: 'At Risk',
  Delayed: 'Delayed',
  Completed: 'Completed',
};

export function WorkPackages() {
  const summary = useExecutionStore((s) => s.summary);
  const { run, error, isSubmitting } = useExecutionMutation();
  const [editingId, setEditingId] = useState<number | null>(null);
  const [form, setForm] = useState<WorkPackageExecutionInputDto>({
    leader_role_id: null,
    notes: null,
  });

  if (!summary) return null;
  const { personnel_roles, work_packages } = summary;

  const startEdit = (wpId: number) => {
    const wp = work_packages.find((w) => w.work_package_id === wpId);
    setForm({ leader_role_id: wp?.leader_role_id ?? null, notes: wp?.notes ?? null });
    setEditingId(wpId);
  };

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (editingId === null) return;
    const ok = await run(() => setWorkPackageExecution(editingId, form));
    if (ok) setEditingId(null);
  };

  return (
    <div className="screen">
      <h1>Work Package Management</h1>
      {error && <p className="error-banner">{error}</p>}

      <div className="wp-grid">
        {work_packages.map((wp) => (
          <div key={wp.work_package_id} className={`wp-card wp-status-${wp.status}`}>
            <h3>
              WP{wp.work_package_id}
              {wp.work_package_name ? ` — ${wp.work_package_name}` : ''}
            </h3>
            <p className="wp-status-badge">{STATUS_LABELS[wp.status] ?? wp.status}</p>
            <dl>
              <dt>Leader</dt>
              <dd>{wp.leader_role_label ?? '—'}</dd>
              <dt>Planned (EUR)</dt>
              <dd>{fmtEur(wp.planned_eur)}</dd>
              <dt>Actual (EUR, personnel only)</dt>
              <dd>{fmtEur(wp.actual_eur)}</dd>
              {wp.notes && (
                <>
                  <dt>Notes</dt>
                  <dd>{wp.notes}</dd>
                </>
              )}
            </dl>
            {wp.overspend_warning && <p className="warning-banner">Actual exceeds planned by &gt;5%</p>}
            {editingId === wp.work_package_id ? (
              <form onSubmit={submit} className="inline-form">
                <select
                  value={form.leader_role_id ?? ''}
                  onChange={(e) =>
                    setForm({ ...form, leader_role_id: e.target.value || null })
                  }
                >
                  <option value="">No leader</option>
                  {personnel_roles.map((r) => (
                    <option key={r.id} value={r.id}>
                      {r.role_label}
                    </option>
                  ))}
                </select>
                <input
                  placeholder="Notes"
                  value={form.notes ?? ''}
                  onChange={(e) => setForm({ ...form, notes: e.target.value || null })}
                />
                <button type="submit" disabled={isSubmitting}>
                  Save
                </button>
                <button type="button" onClick={() => setEditingId(null)}>
                  Cancel
                </button>
              </form>
            ) : (
              <button onClick={() => startEdit(wp.work_package_id)}>Edit</button>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
