/**
 * M-11: Subcontracting Tracking. `competitive_tender_warning`/
 * `host_institution_warning` are advisory-only (BR-SC-03/04) — the backend
 * still enforces the hard cap against the planned lump sum (BR-SC-01).
 */

import { useState } from 'react';
import { useExecutionStore } from '../store/executionStore';
import { useExecutionMutation } from '../hooks/useExecutionMutation';
import { addSubcontractingLine, deleteSubcontractingLine } from '../ipc/commands';
import type { EntryStatus, SubcontractingLineInputDto } from '../types';

const emptyForm: SubcontractingLineInputDto = {
  vendor: '',
  contract_reference: '',
  amount_eur: '',
  work_package_id: 1,
  status: 'Approved',
  vendor_is_host_institution: false,
  payment_date: null,
};

export function Subcontracting() {
  const summary = useExecutionStore((s) => s.summary);
  const { run, error, isSubmitting } = useExecutionMutation();
  const [form, setForm] = useState(emptyForm);

  if (!summary) return null;
  const { subcontracting_lines } = summary;

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    const ok = await run(() => addSubcontractingLine(form));
    if (ok) setForm(emptyForm);
  };

  return (
    <div className="screen">
      <h1>Subcontracting Tracking</h1>
      {error && <p className="error-banner">{error}</p>}

      <table>
        <thead>
          <tr>
            <th>Vendor</th>
            <th>Contract Ref</th>
            <th>Amount (EUR)</th>
            <th>WP</th>
            <th>Status</th>
            <th />
          </tr>
        </thead>
        <tbody>
          {subcontracting_lines.map((line) => (
            <tr key={line.id}>
              <td>{line.vendor}</td>
              <td>{line.contract_reference}</td>
              <td>{line.amount_eur}</td>
              <td>WP{line.work_package_id}</td>
              <td>{line.status}</td>
              <td>
                {line.competitive_tender_warning && (
                  <span className="warning-banner">Tender required</span>
                )}
                {line.host_institution_warning && (
                  <span className="warning-banner">Host institution</span>
                )}
                <button
                  onClick={() => run(() => deleteSubcontractingLine(line.id))}
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
        <input
          placeholder="Vendor"
          value={form.vendor}
          onChange={(e) => setForm({ ...form, vendor: e.target.value })}
          required
        />
        <input
          placeholder="Contract reference"
          value={form.contract_reference}
          onChange={(e) => setForm({ ...form, contract_reference: e.target.value })}
          required
        />
        <input
          placeholder="Amount (EUR)"
          value={form.amount_eur}
          onChange={(e) => setForm({ ...form, amount_eur: e.target.value })}
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
        <select
          value={form.status}
          onChange={(e) => setForm({ ...form, status: e.target.value as EntryStatus })}
        >
          <option value="Approved">Approved</option>
          <option value="Pending">Pending</option>
          <option value="Rejected">Rejected</option>
        </select>
        <label>
          <input
            type="checkbox"
            checked={form.vendor_is_host_institution}
            onChange={(e) =>
              setForm({ ...form, vendor_is_host_institution: e.target.checked })
            }
          />
          Vendor is host institution
        </label>
        <button type="submit" disabled={isSubmitting}>
          Add Line
        </button>
      </form>
    </div>
  );
}
