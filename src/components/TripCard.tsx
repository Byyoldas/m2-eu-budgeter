/**
 * Card component displaying one trip with cost breakdown.
 */

import { useState } from 'react';
import type { TripDetailDto } from '../types';

interface TripCardProps {
  trip: TripDetailDto;
  onEdit: (trip: TripDetailDto) => void;
  onDelete: (id: string) => void;
}

function fmt(v: string | null | undefined): string {
  if (!v) return '—';
  const n = parseFloat(v);
  return isNaN(n) ? '—' : `€ ${n.toLocaleString('en-GB', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

export function TripCard({ trip, onEdit, onDelete }: TripCardProps) {
  const [expanded, setExpanded] = useState(false);
  const isItemized = trip.flight_cost_per_instance !== null;

  return (
    <div className="item-card">
      <div className="item-card-header">
        <div className="item-card-info">
          <span className="item-card-tag">Year {trip.project_year} · ×{trip.number_of_instances}</span>
          <h4 className="item-card-title">{trip.name}</h4>
          <span className="item-card-sub">
            {fmt(trip.per_instance_total_eur)}/instance · Total: <strong>{fmt(trip.total_trip_cost_eur)}</strong>
          </span>
        </div>
        <div className="item-card-actions">
          {isItemized && (
            <button
              className="item-card-expand"
              onClick={() => setExpanded((e) => !e)}
              aria-expanded={expanded}
            >
              {expanded ? '▲' : '▼'}
            </button>
          )}
          <button className="btn btn--sm btn--ghost" onClick={() => onEdit(trip)}>Edit</button>
          <button className="btn btn--sm btn--danger" onClick={() => onDelete(trip.id)}>Delete</button>
        </div>
      </div>

      {expanded && isItemized && (
        <div className="item-card-body">
          <table className="breakdown-table">
            <tbody>
              <tr>
                <td>Flight cost</td>
                <td>{fmt(trip.flight_cost_per_instance)}</td>
              </tr>
              <tr>
                <td>Accommodation</td>
                <td>{fmt(trip.accommodation_cost_per_instance)}</td>
              </tr>
              <tr>
                <td>Subsistence</td>
                <td>{fmt(trip.subsistence_cost_per_instance)}</td>
              </tr>
              <tr>
                <td>Domestic transport</td>
                <td>{fmt(trip.domestic_transport_per_instance)}</td>
              </tr>
              <tr className="breakdown-table-row--total">
                <td><strong>Per instance</strong></td>
                <td><strong>{fmt(trip.per_instance_total_eur)}</strong></td>
              </tr>
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
