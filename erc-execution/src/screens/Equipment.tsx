/**
 * M-09: Equipment Tracking. `delivery_confirmed = false` excludes the
 * procurement from actuals until confirmed (BR-EQ-04).
 */

import { useState } from 'react';
import { useExecutionStore } from '../store/executionStore';
import { useExecutionMutation } from '../hooks/useExecutionMutation';
import { addEquipmentProcurement, deleteEquipmentProcurement } from '../ipc/commands';
import type { EquipmentProcurementInputDto } from '../types';
import { fmtEur } from '../utils/currency';

const emptyForm: EquipmentProcurementInputDto = {
  equipment_item_id: '',
  actual_purchase_cost_eur: '',
  purchase_date: '',
  delivery_confirmed: false,
};

export function Equipment() {
  const summary = useExecutionStore((s) => s.summary);
  const { run, error, isSubmitting } = useExecutionMutation();
  const [form, setForm] = useState(emptyForm);

  if (!summary) return null;
  const { planned_equipment, equipment_procurements } = summary;

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    const ok = await run(() => addEquipmentProcurement(form));
    if (ok) setForm(emptyForm);
  };

  return (
    <div className="screen">
      <h1>Equipment Tracking</h1>
      {error && <p className="error-banner">{error}</p>}

      <table>
        <thead>
          <tr>
            <th>Item</th>
            <th>Purchase Date</th>
            <th>Actual Cost (EUR)</th>
            <th>Delivered</th>
            <th>Eligible Depreciation</th>
            <th />
          </tr>
        </thead>
        <tbody>
          {equipment_procurements.map((ep) => (
            <tr key={ep.id}>
              <td>{ep.equipment_item_name}</td>
              <td>{ep.purchase_date}</td>
              <td>{fmtEur(ep.actual_purchase_cost_eur)}</td>
              <td>{ep.delivery_confirmed ? 'Yes' : 'Pending'}</td>
              <td>{ep.actual_eligible_depreciation_eur != null ? fmtEur(ep.actual_eligible_depreciation_eur) : '—'}</td>
              <td>
                {ep.overspend_warning && <span className="warning-banner">&gt;10% over</span>}
                <button
                  onClick={() => run(() => deleteEquipmentProcurement(ep.id))}
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
          value={form.equipment_item_id}
          onChange={(e) => setForm({ ...form, equipment_item_id: e.target.value })}
          required
        >
          <option value="">Equipment item…</option>
          {planned_equipment.map((item) => (
            <option key={item.id} value={item.id}>
              {item.name} (planned {fmtEur(item.planned_cost_eur)})
            </option>
          ))}
        </select>
        <input
          type="date"
          value={form.purchase_date}
          onChange={(e) => setForm({ ...form, purchase_date: e.target.value })}
          required
        />
        <input
          placeholder="Actual purchase cost (EUR)"
          value={form.actual_purchase_cost_eur}
          onChange={(e) => setForm({ ...form, actual_purchase_cost_eur: e.target.value })}
          required
        />
        <label>
          <input
            type="checkbox"
            checked={form.delivery_confirmed}
            onChange={(e) => setForm({ ...form, delivery_confirmed: e.target.checked })}
          />
          Delivery confirmed
        </label>
        <button type="submit" disabled={isSubmitting}>
          Add Procurement
        </button>
      </form>
    </div>
  );
}
