/**
 * Step 7 — Other Direct Costs (C3) + Subcontracting (B).
 * Regular C3 items, subcontracting lump sum, and CFS item management.
 */

import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { otherCostSchema, subcontractingSchema, type OtherCostFormData, type SubcontractingFormData } from '../validators/schemas';
import { useProjectStore } from '../store/projectStore';
import {
  addOtherCost, updateOtherCost, deleteOtherCost,
  removeCfsItem, setSubcontracting,
} from '../ipc/commands';
import { useBudgetSummary } from '../hooks/useBudgetSummary';
import { EmptyStateCard } from '../components/EmptyStateCard';
import { CFSModal } from '../components/CFSModal';
import { WarningBanner } from '../components/WarningBanner';
import type { OtherCostInput, OtherCostItemDetailDto } from '../types';

interface OtherCostsProps {
  onNext: () => void;
  onBack: () => void;
}

type Mode = 'list' | 'add' | 'edit';

function fmt(v: string | undefined): string {
  if (!v) return '€ 0.00';
  const n = parseFloat(v);
  return isNaN(n) ? '€ 0.00' : `€ ${n.toLocaleString('en-GB', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

export function OtherCosts({ onNext, onBack }: OtherCostsProps) {
  const summary = useProjectStore((s) => s.summary);
  const projectConfig = useProjectStore((s) => s.projectConfig);
  const wpCount = projectConfig?.work_package_count ?? 1;
  const wpNames = projectConfig?.work_package_names ?? [];

  const [mode, setMode] = useState<Mode>('list');
  const [editingId, setEditingId] = useState<string | null>(null);
  const [showCfsModal, setShowCfsModal] = useState(false);

  const { mutate, isLoading, fieldErrors, formError } = useBudgetSummary();

  // C3 items (excluding CFS)
  const regularItems = summary?.other_cost_detail?.filter((i) => !i.is_cfs_item) ?? [];
  const hasCfsItem = summary?.cfs_status === 'REQUIRED_AND_PRESENT';

  const {
    register, handleSubmit, reset, watch, setValue,
    formState: { errors },
  } = useForm<OtherCostFormData>({
    resolver: zodResolver(otherCostSchema),
    defaultValues: { work_package_ids: [] },
  });

  const watchedWpIds = watch('work_package_ids');

  const {
    register: regSub, handleSubmit: handleSub, formState: { errors: subErrors },
  } = useForm<SubcontractingFormData>({
    resolver: zodResolver(subcontractingSchema),
    defaultValues: { amount_eur: summary?.category_b_total ?? '0' },
  });

  const fieldError = (field: string) =>
    fieldErrors.find((e) => e.field === field)?.message ??
    (errors as Record<string, { message?: string }>)[field]?.message;

  const openAdd = () => { reset({ work_package_ids: [] }); setEditingId(null); setMode('add'); };
  const openEdit = (item: OtherCostItemDetailDto) => {
    reset({
      name: item.name,
      amount_eur: item.amount_eur,
      notes: item.notes ?? '',
      work_package_ids: item.work_package_ids,
    });
    setEditingId(item.id);
    setMode('edit');
  };
  const handleDelete = async (id: string) => {
    if (!window.confirm('Delete this cost item?')) return;
    await mutate(() => deleteOtherCost(id));
  };
  const handleRemoveCfs = async () => {
    if (!window.confirm('Remove the CFS item?')) return;
    await mutate(() => removeCfsItem());
  };

  const onSubmit = async (data: OtherCostFormData) => {
    const input: OtherCostInput = {
      name: data.name,
      amount_eur: data.amount_eur,
      notes: data.notes ?? null,
      work_package_ids: data.work_package_ids,
    };
    const command = editingId
      ? () => updateOtherCost(editingId, input)
      : () => addOtherCost(input);
    const result = await mutate(command);
    if (result) setMode('list');
  };

  const onSubSubmit = async (data: SubcontractingFormData) => {
    await mutate(() => setSubcontracting(data.amount_eur, Number(data.work_package_id)));
  };

  if (mode !== 'list') {
    return (
      <div className="screen">
        <div className="screen-header">
          <h2 className="screen-title">{editingId ? 'Edit Cost Item' : 'Add Other Direct Cost'}</h2>
        </div>
        {formError && <WarningBanner message={formError} severity="error" />}

        <form className="screen-form" onSubmit={handleSubmit(onSubmit)} noValidate>
          <div className="form-section">
            <div className="form-field">
              <label htmlFor="oc-name" className="form-label required">Item Name</label>
              <input id="oc-name" type="text" placeholder="e.g. Open Access Publication Fee, Lab Consumables"
                className={`form-input${fieldError('name') ? ' form-input--error' : ''}`}
                {...register('name')} />
              {fieldError('name') && <span className="form-error">{fieldError('name')}</span>}
            </div>
            <div className="form-field">
              <label htmlFor="oc-amount" className="form-label required">Amount (€)</label>
              <input id="oc-amount" type="number" step="any" min={0}
                className={`form-input${fieldError('amount_eur') ? ' form-input--error' : ''}`}
                {...register('amount_eur')} />
              {fieldError('amount_eur') && <span className="form-error">{fieldError('amount_eur')}</span>}
            </div>
            <div className="form-field">
              <label htmlFor="oc-notes" className="form-label">Notes</label>
              <textarea id="oc-notes" rows={2} className="form-input" {...register('notes')} />
            </div>
            <div className="form-field">
              <label className="form-label required">Work Package(s)</label>
              <div className="checkbox-grid">
                {Array.from({ length: wpCount }, (_, i) => i + 1).map((wpId) => (
                  <label key={wpId} className="checkbox-label">
                    <input
                      type="checkbox"
                      value={wpId}
                      onChange={(e) => {
                        const current = watchedWpIds ?? [];
                        setValue('work_package_ids', e.target.checked ? [...current, wpId] : current.filter((w) => w !== wpId));
                      }}
                      checked={(watchedWpIds ?? []).includes(wpId)}
                    />
                    {(wpNames[wpId - 1] as string | null) ?? `WP${wpId}`}
                  </label>
                ))}
              </div>
              {fieldError('work_package_ids') && <span className="form-error">{fieldError('work_package_ids')}</span>}
              <span className="form-hint">Cost is split evenly across all selected Work Packages.</span>
            </div>
          </div>
          <div className="screen-footer">
            <button type="button" className="btn btn--ghost" onClick={() => setMode('list')}>Cancel</button>
            <button type="submit" className="btn btn--primary" disabled={isLoading}>
              {isLoading ? 'Saving…' : (editingId ? 'Update Item' : 'Add Item')}
            </button>
          </div>
        </form>
      </div>
    );
  }

  return (
    <div className="screen">
      <div className="screen-header">
        <h2 className="screen-title">Other Costs & Subcontracting</h2>
        <p className="screen-description">
          Category C3: Other direct costs. Category B: Subcontracting.
        </p>
      </div>

      {summary?.cfs_prompt_required && (
        <WarningBanner
          message="Budget exceeds €430,000 — a Certificate on Financial Statements (CFS) is required."
          severity="warning"
          action={{ label: 'Add CFS Cost', onClick: () => setShowCfsModal(true) }}
        />
      )}

      {/* Subcontracting */}
      <div className="form-section card">
        <h3 className="form-section-title">Subcontracting (Category B)</h3>
        <p className="form-section-hint">
          Total subcontracting for the project. Enter 0 if none. Excluded from indirect cost base.
        </p>
        <form onSubmit={handleSub(onSubSubmit)} className="inline-form">
          <div className="form-field">
            <label htmlFor="sub-amount" className="form-label">Total Subcontracting (€)</label>
            <input id="sub-amount" type="number" step="any" min={0}
              className={`form-input${subErrors.amount_eur ? ' form-input--error' : ''}`}
              {...regSub('amount_eur')} />
            {subErrors.amount_eur && <span className="form-error">{subErrors.amount_eur.message}</span>}
          </div>
          <div className="form-field">
            <label htmlFor="sub-wp" className="form-label">Work Package</label>
            <select id="sub-wp"
              className={`form-input${subErrors.work_package_id ? ' form-input--error' : ''}`}
              {...regSub('work_package_id')}>
              <option value="">— Select Work Package —</option>
              {Array.from({ length: wpCount }, (_, i) => i + 1).map((wpId) => (
                <option key={wpId} value={wpId}>{(wpNames[wpId - 1] as string | null) ?? `WP${wpId}`}</option>
              ))}
            </select>
            {subErrors.work_package_id && <span className="form-error">{subErrors.work_package_id.message}</span>}
          </div>
          <button type="submit" className="btn btn--ghost" disabled={isLoading}>Update</button>
        </form>
        <div className="totals-row">
          <span>Current total:</span>
          <strong>{fmt(summary?.category_b_total)}</strong>
        </div>
      </div>

      {/* Other Direct Costs */}
      <div className="list-section">
        <div className="list-section-header">
          <h3>Other Direct Costs (Category C3)</h3>
          <button className="btn btn--primary" onClick={openAdd}>+ Add Item</button>
        </div>

        {hasCfsItem && (
          <div className="item-card item-card--cfs">
            <div className="item-card-header">
              <div className="item-card-info">
                <span className="badge badge--info">CFS</span>
                <span className="item-card-title">Certificate on Financial Statements</span>
              </div>
              <button className="btn btn--sm btn--danger" onClick={handleRemoveCfs}>Remove</button>
            </div>
          </div>
        )}

        {regularItems.length === 0 && !hasCfsItem ? (
          <EmptyStateCard icon="📋" title="No other direct costs yet"
            description="Add publication fees, lab consumables, software licenses, and other direct costs."
            action={{ label: '+ Add First Item', onClick: openAdd }} />
        ) : (
          regularItems.map((item: OtherCostItemDetailDto) => (
            <div key={item.id} className="item-card">
              <div className="item-card-header">
                <div className="item-card-info">
                  <span className="item-card-tag">
                    {item.work_package_ids.map((id) => (wpNames[id - 1] as string | null) ?? `WP${id}`).join(', ') || '—'}
                  </span>
                  <h4 className="item-card-title">{item.name}</h4>
                  <span className="item-card-sub">{fmt(item.amount_eur)}</span>
                  {item.notes && <span className="item-card-hint">{item.notes}</span>}
                </div>
                <div className="item-card-actions">
                  <button className="btn btn--sm btn--ghost" onClick={() => openEdit(item)}>Edit</button>
                  <button className="btn btn--sm btn--danger" onClick={() => handleDelete(item.id)}>Delete</button>
                </div>
              </div>
            </div>
          ))
        )}
      </div>

      <div className="screen-footer">
        <button className="btn btn--ghost" onClick={onBack}>← Back</button>
        <button className="btn btn--primary btn--lg" onClick={onNext}>Next: Review & Export →</button>
      </div>

      <CFSModal open={showCfsModal} onClose={() => setShowCfsModal(false)} />
    </div>
  );
}
