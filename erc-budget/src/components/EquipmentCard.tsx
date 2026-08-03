/**
 * Card component displaying one equipment item with depreciation breakdown.
 */

import type { EquipmentItemDetailDto } from '../types';

interface EquipmentCardProps {
  item: EquipmentItemDetailDto;
  onEdit: (item: EquipmentItemDetailDto) => void;
  onDelete: (id: string) => void;
}

function fmt(v: string): string {
  const n = parseFloat(v);
  return isNaN(n) ? '€ 0.00' : `€ ${n.toLocaleString('en-GB', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

export function EquipmentCard({ item, onEdit, onDelete }: EquipmentCardProps) {
  return (
    <div className="item-card">
      <div className="item-card-header">
        <div className="item-card-info">
          <h4 className="item-card-title">{item.name}</h4>
          <div className="item-card-meta">
            <span className="item-card-sub">
              Eligible depreciation: <strong>{fmt(item.eligible_depreciation_eur)}</strong>
            </span>
            {item.is_capped && (
              <span className="badge badge--warning" title="Depreciation capped at cost × usage%">
                Capped
              </span>
            )}
          </div>
          <div className="item-card-sub-row">
            <span>Theoretical: {fmt(item.theoretical_eligible_eur)}</span>
            <span>Max (usage cap): {fmt(item.maximum_eligible_eur)}</span>
          </div>
        </div>
        <div className="item-card-actions">
          <button className="btn btn--sm btn--ghost" onClick={() => onEdit(item)}>Edit</button>
          <button className="btn btn--sm btn--danger" onClick={() => onDelete(item.id)}>Delete</button>
        </div>
      </div>
    </div>
  );
}
