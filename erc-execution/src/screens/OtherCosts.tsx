/**
 * M-10: Other Costs Tracking (Category C3). Entries not linked to a planned
 * item are "unbudgeted" and require justification text (BR-OC-03).
 */

import { useState } from 'react';
import { useExecutionStore } from '../store/executionStore';
import { useExecutionMutation } from '../hooks/useExecutionMutation';
import { addActualCostEntry, deleteActualCostEntry } from '../ipc/commands';
import type { ActualCostEntryInputDto } from '../types';
import { fmtEur } from '../utils/currency';

const emptyForm: ActualCostEntryInputDto = {
  linked_entity_id: null,
  amount_eur: '',
  description: '',
  incurred_date: '',
  status: 'Approved',
  justification: null,
};

export function OtherCosts() {
  const summary = useExecutionStore((s) => s.summary);
  const { run, error, isSubmitting } = useExecutionMutation();
  const [form, setForm] = useState(emptyForm);

  if (!summary) return null;
  const { planned_other_costs, actual_cost_entries } = summary;
  const isUnbudgeted = form.linked_entity_id === null;

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    const ok = await run(() => addActualCostEntry(form));
    if (ok) setForm(emptyForm);
  };

  return (
    <div className="screen">
      <h1>Other Costs Tracking</h1>
      {error && <p className="error-banner">{error}</p>}

      <table>
        <thead>
          <tr>
            <th>Linked Item</th>
            <th>Description</th>
            <th>Amount (EUR)</th>
            <th>Date</th>
            <th>Status</th>
            <th />
          </tr>
        </thead>
        <tbody>
          {actual_cost_entries.map((entry) => (
            <tr key={entry.id}>
              <td>{entry.linked_entity_name ?? 'Unbudgeted'}</td>
              <td>{entry.description}</td>
              <td>{fmtEur(entry.amount_eur)}</td>
              <td>{entry.incurred_date}</td>
              <td>{entry.status}</td>
              <td>
                {entry.overspend_warning && <span className="warning-banner">&gt;10% over</span>}
                <button
                  onClick={() => run(() => deleteActualCostEntry(entry.id))}
                  disabled={isSubmitting}
                >
                  Delete
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>

      <form onSubmit={submit} className="inline-form">
        <select
          value={form.linked_entity_id ?? ''}
          onChange={(e) =>
            setForm({ ...form, linked_entity_id: e.target.value || null })
          }
        >
          <option value="">Unbudgeted item</option>
          {planned_other_costs.map((item) => (
            <option key={item.id} value={item.id}>
              {item.name} (planned {fmtEur(item.amount_eur)})
            </option>
          ))}
        </select>
        <input
          placeholder="Description"
          value={form.description}
          onChange={(e) => setForm({ ...form, description: e.target.value })}
          required
        />
        <input
          placeholder="Amount (EUR)"
          value={form.amount_eur}
          onChange={(e) => setForm({ ...form, amount_eur: e.target.value })}
          required
        />
        <input
          type="date"
          value={form.incurred_date}
          onChange={(e) => setForm({ ...form, incurred_date: e.target.value })}
          required
        />
        {isUnbudgeted && (
          <input
            placeholder="Justification (required for unbudgeted items)"
            value={form.justification ?? ''}
            onChange={(e) => setForm({ ...form, justification: e.target.value || null })}
            required
          />
        )}
        <button type="submit" disabled={isSubmitting}>
          Add Entry
        </button>
      </form>
    </div>
  );
}
