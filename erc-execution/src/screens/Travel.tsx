/**
 * M-08: Travel Tracking. `planned_trips` come from the read-only Budget App
 * data; each execution links to one trip instance and a traveller `Person`.
 */

import { useState } from 'react';
import { useExecutionStore } from '../store/executionStore';
import { useExecutionMutation } from '../hooks/useExecutionMutation';
import { addTripExecution, deleteTripExecution } from '../ipc/commands';
import type { EntryStatus, TripExecutionInputDto } from '../types';

const emptyForm: TripExecutionInputDto = {
  trip_id: '',
  instance_number: 1,
  traveller_person_id: '',
  actual_travel_date: '',
  actual_cost_eur: '',
  status: 'Approved',
};

export function Travel() {
  const summary = useExecutionStore((s) => s.summary);
  const { run, error, isSubmitting } = useExecutionMutation();
  const [form, setForm] = useState(emptyForm);

  if (!summary) return null;
  const { planned_trips, persons, trip_executions } = summary;

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    const ok = await run(() => addTripExecution(form));
    if (ok) setForm(emptyForm);
  };

  return (
    <div className="screen">
      <h1>Travel Tracking</h1>
      {error && <p className="error-banner">{error}</p>}

      <table>
        <thead>
          <tr>
            <th>Trip</th>
            <th>Instance</th>
            <th>Traveller</th>
            <th>Date</th>
            <th>Actual (EUR)</th>
            <th>Planned/Instance</th>
            <th>Status</th>
            <th />
          </tr>
        </thead>
        <tbody>
          {trip_executions.map((te) => (
            <tr key={te.id}>
              <td>{te.trip_name}</td>
              <td>{te.instance_number}</td>
              <td>{te.traveller_name}</td>
              <td>{te.actual_travel_date}</td>
              <td>{te.actual_cost_eur}</td>
              <td>{te.planned_cost_per_instance_eur}</td>
              <td>{te.status}</td>
              <td>
                {te.overspend_warning && <span className="warning-banner">&gt;20% over</span>}
                <button onClick={() => run(() => deleteTripExecution(te.id))} disabled={isSubmitting}>
                  Delete
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>

      <form onSubmit={submit} className="inline-form">
        <select
          value={form.trip_id}
          onChange={(e) => setForm({ ...form, trip_id: e.target.value })}
          required
        >
          <option value="">Trip…</option>
          {planned_trips.map((t) => (
            <option key={t.id} value={t.id}>
              {t.name} ({t.number_of_instances} instances)
            </option>
          ))}
        </select>
        <input
          type="number"
          min={1}
          placeholder="Instance #"
          value={form.instance_number}
          onChange={(e) => setForm({ ...form, instance_number: Number(e.target.value) })}
          required
        />
        <select
          value={form.traveller_person_id}
          onChange={(e) => setForm({ ...form, traveller_person_id: e.target.value })}
          required
        >
          <option value="">Traveller…</option>
          {persons.map((p) => (
            <option key={p.id} value={p.id}>
              {p.full_name}
            </option>
          ))}
        </select>
        <input
          type="date"
          value={form.actual_travel_date}
          onChange={(e) => setForm({ ...form, actual_travel_date: e.target.value })}
          required
        />
        <input
          placeholder="Actual cost (EUR)"
          value={form.actual_cost_eur}
          onChange={(e) => setForm({ ...form, actual_cost_eur: e.target.value })}
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
        <button type="submit" disabled={isSubmitting}>
          Add Trip Execution
        </button>
      </form>
    </div>
  );
}
