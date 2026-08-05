/**
 * M-03: Personnel & Person-Month Tracking. Two sections: the roster (who's
 * linked to which planned role) and the month-by-month FTE ledger for them.
 */

import { useState } from 'react';
import { useExecutionStore } from '../store/executionStore';
import { useExecutionMutation } from '../hooks/useExecutionMutation';
import { addPerson, deletePerson, addPersonMonthRecord, deletePersonMonthRecord } from '../ipc/commands';
import type { PersonDetailDto, PersonInputDto, PersonMonthRecordInputDto } from '../types';
import { fmtEur } from '../utils/currency';
import { exportTimeDeclarations } from '../export/timeDeclarationExporter';

const emptyPerson: PersonInputDto = {
  full_name: '',
  email: null,
  institution: null,
  orcid: null,
  linked_role_id: '',
  actual_start_date: '',
  actual_end_date: null,
};

const emptyRecord: PersonMonthRecordInputDto = {
  person_id: '',
  project_month: 1,
  reported_months: '1',
  approved_months: null,
};

export function Personnel() {
  const summary = useExecutionStore((s) => s.summary);
  const { run, error, isSubmitting } = useExecutionMutation();
  const [personForm, setPersonForm] = useState(emptyPerson);
  const [recordForm, setRecordForm] = useState(emptyRecord);
  const [exportError, setExportError] = useState<string | null>(null);

  if (!summary) return null;
  const { personnel_roles, persons, person_months } = summary;

  const submitPerson = async (e: React.FormEvent) => {
    e.preventDefault();
    const ok = await run(() => addPerson(personForm));
    if (ok) setPersonForm(emptyPerson);
  };

  const submitRecord = async (e: React.FormEvent) => {
    e.preventDefault();
    const ok = await run(() => addPersonMonthRecord(recordForm));
    if (ok) setRecordForm(emptyRecord);
  };

  const exportTimeDeclaration = async (person: PersonDetailDto) => {
    setExportError(null);
    try {
      await exportTimeDeclarations(person, person_months);
    } catch (e) {
      setExportError(e instanceof Error ? e.message : 'Time declaration export failed.');
    }
  };

  return (
    <div className="screen">
      <h1>Personnel &amp; Person-Month Tracking</h1>
      {error && <p className="error-banner">{error}</p>}
      {exportError && <p className="error-banner">{exportError}</p>}

      <section>
        <h2>Roster</h2>
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Linked Role</th>
              <th>Start</th>
              <th>End</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {persons.map((p) => (
              <tr key={p.id}>
                <td>{p.full_name}</td>
                <td>{p.linked_role_label}</td>
                <td>{p.actual_start_date}</td>
                <td>{p.actual_end_date ?? '—'}</td>
                <td>
                  <button onClick={() => exportTimeDeclaration(p)} disabled={isSubmitting}>
                    Time Declaration
                  </button>
                  <button onClick={() => run(() => deletePerson(p.id))} disabled={isSubmitting}>
                    Delete
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>

        <form onSubmit={submitPerson} className="inline-form">
          <input
            placeholder="Full name"
            value={personForm.full_name}
            onChange={(e) => setPersonForm({ ...personForm, full_name: e.target.value })}
            required
          />
          <select
            value={personForm.linked_role_id}
            onChange={(e) => setPersonForm({ ...personForm, linked_role_id: e.target.value })}
            required
          >
            <option value="">Link to role…</option>
            {personnel_roles.map((r) => (
              <option key={r.id} value={r.id}>
                {r.role_label}
              </option>
            ))}
          </select>
          <input
            type="date"
            value={personForm.actual_start_date}
            onChange={(e) => setPersonForm({ ...personForm, actual_start_date: e.target.value })}
            required
          />
          <button type="submit" disabled={isSubmitting}>
            Add Person
          </button>
        </form>
      </section>

      <section>
        <h2>Person-Month Records</h2>
        <table>
          <thead>
            <tr>
              <th>Person</th>
              <th>Month</th>
              <th>Reported</th>
              <th>Approved</th>
              <th>Est. Cost (EUR)</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {person_months.map((r) => {
              const person = persons.find((p) => p.id === r.person_id);
              return (
                <tr key={r.id}>
                  <td>{person?.full_name ?? 'Unknown'}</td>
                  <td>{r.project_month}</td>
                  <td>{r.reported_months}</td>
                  <td>{r.approved_months ?? '—'}</td>
                  <td>{r.salary_cost_estimate_eur != null ? fmtEur(r.salary_cost_estimate_eur) : '—'}</td>
                  <td>
                    <button
                      onClick={() => run(() => deletePersonMonthRecord(r.id))}
                      disabled={isSubmitting}
                    >
                      Delete
                    </button>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>

        <form onSubmit={submitRecord} className="inline-form">
          <select
            value={recordForm.person_id}
            onChange={(e) => setRecordForm({ ...recordForm, person_id: e.target.value })}
            required
          >
            <option value="">Person…</option>
            {persons.map((p) => (
              <option key={p.id} value={p.id}>
                {p.full_name}
              </option>
            ))}
          </select>
          <input
            type="number"
            min={1}
            placeholder="Project month"
            value={recordForm.project_month}
            onChange={(e) =>
              setRecordForm({ ...recordForm, project_month: Number(e.target.value) })
            }
            required
          />
          <input
            placeholder="Reported (0–1)"
            value={recordForm.reported_months}
            onChange={(e) => setRecordForm({ ...recordForm, reported_months: e.target.value })}
            required
          />
          <input
            placeholder="Approved (optional)"
            value={recordForm.approved_months ?? ''}
            onChange={(e) =>
              setRecordForm({ ...recordForm, approved_months: e.target.value || null })
            }
          />
          <button type="submit" disabled={isSubmitting}>
            Add Record
          </button>
        </form>
      </section>
    </div>
  );
}
