/**
 * Card component displaying one personnel role with expandable year breakdown.
 */

import { useState } from 'react';
import type { PersonnelRoleDetailDto } from '../types';

interface RoleCardProps {
  role: PersonnelRoleDetailDto;
  onEdit: (role: PersonnelRoleDetailDto) => void;
  onDelete: (id: string) => void;
}

const ROLE_TYPE_LABELS: Record<string, string> = {
  Pi: 'Principal Investigator',
  Expert: 'Expert / Senior Researcher',
  PostDoc: 'Post-Doctoral Researcher',
  PhdStudent: 'PhD Student',
  MscStudent: 'MSc Student',
  Admin: 'Administrative Staff',
};

function fmt(v: string): string {
  const n = parseFloat(v);
  return isNaN(n) ? '€ 0.00' : `€ ${n.toLocaleString('en-GB', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

export function RoleCard({ role, onEdit, onDelete }: RoleCardProps) {
  const [expanded, setExpanded] = useState(false);

  return (
    <div className="item-card">
      <div className="item-card-header">
        <div className="item-card-info">
          <span className="item-card-tag">{ROLE_TYPE_LABELS[role.role_type] ?? role.role_type}</span>
          <h4 className="item-card-title">{role.role_label}</h4>
          <span className="item-card-sub">
            PM {parseFloat(role.fte_fraction).toFixed(2)} · Total: {fmt(role.total_cost_eur)}
          </span>
        </div>
        <div className="item-card-actions">
          <button
            className="item-card-expand"
            onClick={() => setExpanded((e) => !e)}
            aria-expanded={expanded}
            aria-label={expanded ? 'Collapse year breakdown' : 'Expand year breakdown'}
          >
            {expanded ? '▲' : '▼'}
          </button>
          <button className="btn btn--sm btn--ghost" onClick={() => onEdit(role)}>Edit</button>
          <button className="btn btn--sm btn--danger" onClick={() => onDelete(role.id)}>Delete</button>
        </div>
      </div>

      {expanded && (
        <div className="item-card-body">
          <table className="breakdown-table">
            <thead>
              <tr>
                <th>Year</th>
                <th>Active</th>
                <th>Monthly (€)</th>
                <th>Annual Cost (€)</th>
              </tr>
            </thead>
            <tbody>
              {role.cost_lines.map((line) => (
                <tr key={line.year} className={line.is_active ? '' : 'breakdown-table-row--inactive'}>
                  <td>Y{line.year}</td>
                  <td>{line.is_active ? '✓' : '—'}</td>
                  <td>{line.is_active ? fmt(line.monthly_salary_eur) : '—'}</td>
                  <td>{line.is_active ? fmt(line.annual_cost_eur) : '—'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
