/**
 * Step 8 — Subcontracting (Category B).
 * Entered item-by-item, the same way Other Direct Cost (C3) items are —
 * name, amount, notes, and Work Package(s). Entirely optional: a project
 * with no subcontracting simply has an empty list and moves on.
 */

import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { subcontractingItemSchema, type SubcontractingItemFormData } from '../validators/schemas';
import { useProjectStore, useSubcontractingItems } from '../store/projectStore';
import { addSubcontractingItem, updateSubcontractingItem, deleteSubcontractingItem } from '../ipc/commands';
import { useBudgetSummary } from '../hooks/useBudgetSummary';
import { EmptyStateCard } from '../components/EmptyStateCard';
import { WarningBanner } from '../components/WarningBanner';
import type { SubcontractingInput, SubcontractingItemDetailDto } from '../types';

interface SubcontractingProps {
  onNext: () => void;
  onBack: () => void;
}

type Mode = 'list' | 'add' | 'edit';

const emptyDefaults: SubcontractingItemFormData = {
  name: '', amount_eur: '', notes: '', work_package_ids: [],
};

function fmt(v: string | undefined): string {
  if (!v) return '€ 0.00';
  const n = parseFloat(v);
  return isNaN(n) ? '€ 0.00' : `€ ${n.toLocaleString('en-GB', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

export function Subcontracting({ onNext, onBack }: SubcontractingProps) {
  const items = useSubcontractingItems();
  const projectConfig = useProjectStore((s) => s.projectConfig);
  const wpCount = projectConfig?.work_package_count ?? 1;
  const wpNames = projectConfig?.work_package_names ?? [];

  const [mode, setMode] = useState<Mode>('list');
  const [editingId, setEditingId] = useState<string | null>(null);

  const { mutate, isLoading, fieldErrors, formError } = useBudgetSummary();

  const {
    register, handleSubmit, reset, watch, setValue,
    formState: { errors },
  } = useForm<SubcontractingItemFormData>({
    resolver: zodResolver(subcontractingItemSchema),
    defaultValues: emptyDefaults,
  });

  const watchedWpIds = watch('work_package_ids');

  const fieldError = (field: string) =>
    fieldErrors.find((e) => e.field === field)?.message ??
    (errors as Record<string, { message?: string }>)[field]?.message;

  // Passing an explicit empty object (not a bare reset()) matters here — a
  // bare reset() falls back to whatever values the *last* reset(values) call
  // set as the form's new baseline, so after editing (or duplicating) an
  // item, the next "Add Item" would reopen with that item's data still
  // filled in instead of a clean form.
  const openAdd = () => { reset(emptyDefaults); setEditingId(null); setMode('add'); };
  const openEdit = (item: SubcontractingItemDetailDto) => {
    reset({
      name: item.name,
      amount_eur: item.amount_eur,
      notes: item.notes ?? '',
      work_package_ids: item.work_package_ids,
    });
    setEditingId(item.id);
    setMode('edit');
  };
  const openDuplicate = (item: SubcontractingItemDetailDto) => {
    reset({
      name: `${item.name} (copy)`,
      amount_eur: item.amount_eur,
      notes: item.notes ?? '',
      work_package_ids: item.work_package_ids,
    });
    setEditingId(null);
    setMode('add');
  };
  const handleDelete = async (id: string) => {
    if (!window.confirm('Delete this subcontracting item?')) return;
    await mutate(() => deleteSubcontractingItem(id));
  };

  const onSubmit = async (data: SubcontractingItemFormData) => {
    const input: SubcontractingInput = {
      name: data.name,
      amount_eur: data.amount_eur,
      notes: data.notes ?? null,
      work_package_ids: data.work_package_ids,
    };
    const command = editingId
      ? () => updateSubcontractingItem(editingId, input)
      : () => addSubcontractingItem(input);
    const result = await mutate(command);
    if (result) setMode('list');
  };

  if (mode !== 'list') {
    return (
      <div className="screen">
        <div className="screen-header">
          <h2 className="screen-title">{editingId ? 'Edit Subcontracting Item' : 'Add Subcontracting Item'}</h2>
        </div>
        {formError && <WarningBanner message={formError} severity="error" />}

        <form className="screen-form" onSubmit={handleSubmit(onSubmit)} noValidate>
          <div className="form-section">
            <div className="form-field">
              <label htmlFor="sub-name" className="form-label required">Item Name</label>
              <input id="sub-name" type="text" placeholder="e.g. Fieldwork subcontract, Data annotation service"
                className={`form-input${fieldError('name') ? ' form-input--error' : ''}`}
                {...register('name')} />
              {fieldError('name') && <span className="form-error">{fieldError('name')}</span>}
            </div>
            <div className="form-field">
              <label htmlFor="sub-amount" className="form-label required">Amount (€)</label>
              <input id="sub-amount" type="number" step="any" min={0}
                className={`form-input${fieldError('amount_eur') ? ' form-input--error' : ''}`}
                {...register('amount_eur')} />
              {fieldError('amount_eur') && <span className="form-error">{fieldError('amount_eur')}</span>}
            </div>
            <div className="form-field">
              <label htmlFor="sub-notes" className="form-label">Notes</label>
              <textarea id="sub-notes" rows={2} className="form-input" {...register('notes')} />
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
        <h2 className="screen-title">Subcontracting (Category B)</h2>
        <p className="screen-description">
          Optional — leave empty if this project has no subcontracting. Add each subcontract
          as its own item, same as Other Direct Costs.
        </p>
        <button className="btn btn--primary" onClick={openAdd}>+ Add Item</button>
      </div>
      <div className="item-list">
        {items.length === 0 ? (
          <EmptyStateCard icon="🤝" title="No subcontracting yet"
            description="Add subcontracted fieldwork, services, or other externally-delivered work. Skip this if the project has none."
            action={{ label: '+ Add First Item', onClick: openAdd }} />
        ) : (
          items.map((item: SubcontractingItemDetailDto) => (
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
                  <button className="btn btn--sm btn--ghost" onClick={() => openDuplicate(item)}>Duplicate</button>
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
    </div>
  );
}
