/**
 * Step 5 — Equipment (Category C2).
 * Add/edit/delete equipment items. Live depreciation preview as user types.
 */

import { useState, useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { equipmentItemSchema, type EquipmentItemFormData } from '../validators/schemas';
import { useProjectStore, useEquipmentItems } from '../store/projectStore';
import {
  addEquipmentItem, updateEquipmentItem, deleteEquipmentItem,
  previewEquipmentDepreciation,
} from '../ipc/commands';
import { useBudgetSummary, usePreview } from '../hooks/useBudgetSummary';
import { EquipmentCard } from '../components/EquipmentCard';
import { EmptyStateCard } from '../components/EmptyStateCard';
import { LivePreviewBox } from '../components/LivePreviewBox';
import type { EquipmentItemDetailDto, EquipmentPreviewDto, EquipmentItemInput } from '../types';

interface EquipmentProps {
  onNext: () => void;
  onBack: () => void;
}

type Mode = 'list' | 'add' | 'edit';

function fmt(v: string): string {
  const n = parseFloat(v);
  return isNaN(n) ? '—' : `€ ${n.toLocaleString('en-GB', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

export function Equipment({ onNext, onBack }: EquipmentProps) {
  const items = useEquipmentItems();
  const projectConfig = useProjectStore((s) => s.projectConfig);
  const duration = projectConfig?.duration_years ?? 5;
  const wpCount = projectConfig?.work_package_count ?? 1;
  const wpNames = projectConfig?.work_package_names ?? [];

  const [mode, setMode] = useState<Mode>('list');
  const [editingItem, setEditingItem] = useState<EquipmentItemDetailDto | null>(null);
  const [previewResult, setPreviewResult] = useState<EquipmentPreviewDto | null>(null);

  const { mutate, isLoading, fieldErrors } = useBudgetSummary();
  const { preview, isLoading: previewLoading } = usePreview<EquipmentPreviewDto>();

  const {
    register, handleSubmit, watch, reset, setValue,
    formState: { errors },
  } = useForm<EquipmentItemFormData>({
    resolver: zodResolver(equipmentItemSchema),
    defaultValues: { work_package_ids: [] },
  });

  const watched = watch();

  useEffect(() => {
    const timer = setTimeout(async () => {
      const cost = parseFloat(watched.purchase_cost_eur);
      const lifetime = Number(watched.useful_lifetime_months);
      const usagePct = parseFloat(watched.grant_usage_pct);
      const usageMonths = Number(watched.grant_usage_months);
      if (!cost || !lifetime || !usagePct || !usageMonths) { setPreviewResult(null); return; }
      const result = await preview(() =>
        previewEquipmentDepreciation({
          name: watched.name ?? '',
          purchase_cost_eur: watched.purchase_cost_eur,
          useful_lifetime_months: lifetime,
          grant_usage_pct: watched.grant_usage_pct,
          grant_usage_months: usageMonths,
          year_of_purchase: watched.year_of_purchase ?? null,
          work_package_ids: watched.work_package_ids ?? [],
        })
      );
      setPreviewResult(result);
    }, 400);
    return () => clearTimeout(timer);
  }, [JSON.stringify(watched)]);

  const fieldError = (field: string) =>
    fieldErrors.find((e) => e.field === field)?.message ??
    (errors as Record<string, { message?: string }>)[field]?.message;

  const openAdd = () => { reset({ work_package_ids: [] }); setEditingItem(null); setPreviewResult(null); setMode('add'); };

  const openEdit = (item: EquipmentItemDetailDto) => {
    setEditingItem(item);
    reset({ name: item.name, work_package_ids: [] });
    setPreviewResult(null);
    setMode('edit');
  };

  const handleDelete = async (id: string) => {
    if (!window.confirm('Delete this equipment item?')) return;
    await mutate(() => deleteEquipmentItem(id));
  };

  const onSubmit = async (data: EquipmentItemFormData) => {
    const input: EquipmentItemInput = {
      name: data.name,
      purchase_cost_eur: data.purchase_cost_eur,
      useful_lifetime_months: Number(data.useful_lifetime_months),
      grant_usage_pct: data.grant_usage_pct,
      grant_usage_months: Number(data.grant_usage_months),
      year_of_purchase: data.year_of_purchase ?? null,
      work_package_ids: data.work_package_ids ?? [],
    };
    const command = editingItem
      ? () => updateEquipmentItem(editingItem.id, input)
      : () => addEquipmentItem(input);
    const result = await mutate(command);
    if (result) setMode('list');
  };

  const previewRows = previewResult
    ? [
        { label: 'Theoretical eligible', value: fmt(previewResult.theoretical_eligible_eur) },
        { label: 'Maximum (cost × usage%)', value: fmt(previewResult.maximum_eligible_eur) },
        {
          label: 'Eligible depreciation',
          value: fmt(previewResult.eligible_depreciation_eur),
          highlight: true,
        },
        ...(previewResult.is_capped
          ? [{ label: '⚠ Capped at cost × usage%', value: '' }]
          : []),
      ]
    : [];

  if (mode !== 'list') {
    return (
      <div className="screen">
        <div className="screen-header">
          <h2 className="screen-title">{mode === 'add' ? 'Add Equipment Item' : 'Edit Equipment Item'}</h2>
        </div>
        <div className="screen-split">
          <form className="screen-form" onSubmit={handleSubmit(onSubmit)} noValidate>
            <div className="form-section">
              <div className="form-field">
                <label htmlFor="eq-name" className="form-label required">Item Name</label>
                <input id="eq-name" type="text" placeholder="e.g. Laptop, Server" className={`form-input${fieldError('name') ? ' form-input--error' : ''}`} {...register('name')} />
                {fieldError('name') && <span className="form-error">{fieldError('name')}</span>}
              </div>
              <div className="form-row">
                <div className="form-field">
                  <label htmlFor="purchase_cost" className="form-label required">Purchase Cost (€)</label>
                  <input id="purchase_cost" type="number" step="any" className={`form-input${fieldError('purchase_cost_eur') ? ' form-input--error' : ''}`} {...register('purchase_cost_eur')} />
                  {fieldError('purchase_cost_eur') && <span className="form-error">{fieldError('purchase_cost_eur')}</span>}
                </div>
                <div className="form-field">
                  <label htmlFor="lifetime" className="form-label required">Useful Lifetime (months)</label>
                  <input id="lifetime" type="number" min={1} className={`form-input${fieldError('useful_lifetime_months') ? ' form-input--error' : ''}`} {...register('useful_lifetime_months', { valueAsNumber: true })} />
                  {fieldError('useful_lifetime_months') && <span className="form-error">{fieldError('useful_lifetime_months')}</span>}
                </div>
              </div>
              <div className="form-row">
                <div className="form-field">
                  <label htmlFor="usage_pct" className="form-label required">Grant Usage (%)</label>
                  <input id="usage_pct" type="number" step="any" min={0} max={100} placeholder="e.g. 80" className={`form-input${fieldError('grant_usage_pct') ? ' form-input--error' : ''}`} {...register('grant_usage_pct')} />
                  {fieldError('grant_usage_pct') && <span className="form-error">{fieldError('grant_usage_pct')}</span>}
                  <span className="form-hint">% of time the item is used for the grant.</span>
                </div>
                <div className="form-field">
                  <label htmlFor="usage_months" className="form-label required">Usage Months (grant period)</label>
                  <input id="usage_months" type="number" min={1} className={`form-input${fieldError('grant_usage_months') ? ' form-input--error' : ''}`} {...register('grant_usage_months', { valueAsNumber: true })} />
                  {fieldError('grant_usage_months') && <span className="form-error">{fieldError('grant_usage_months')}</span>}
                </div>
              </div>
              <div className="form-field">
                <label htmlFor="year_purchase" className="form-label">Year of Purchase (within project)</label>
                <select id="year_purchase" className="form-input" {...register('year_of_purchase', { setValueAs: (v) => v === '' ? null : Number(v) })}>
                  <option value="">— Select year —</option>
                  {Array.from({ length: duration }, (_, i) => i + 1).map((y) => (
                    <option key={y} value={y}>Year {y}</option>
                  ))}
                </select>
                <span className="form-hint">Used for per-year indirect cost distribution display only.</span>
              </div>
              {wpCount > 0 && (
                <div className="form-field">
                  <label className="form-label">Work Packages</label>
                  <div className="checkbox-grid">
                    {Array.from({ length: wpCount }, (_, i) => i + 1).map((wpId) => (
                      <label key={wpId} className="checkbox-label">
                        <input type="checkbox" value={wpId}
                          onChange={(e) => {
                            const current = watched.work_package_ids ?? [];
                            setValue('work_package_ids', e.target.checked ? [...current, wpId] : current.filter((w) => w !== wpId));
                          }}
                          checked={(watched.work_package_ids ?? []).includes(wpId)}
                        />
                        {(wpNames[wpId - 1] as string | null) ?? `WP${wpId}`}
                      </label>
                    ))}
                  </div>
                </div>
              )}
            </div>
            <div className="screen-footer">
              <button type="button" className="btn btn--ghost" onClick={() => setMode('list')}>Cancel</button>
              <button type="submit" className="btn btn--primary" disabled={isLoading}>
                {isLoading ? 'Saving…' : (editingItem ? 'Update Item' : 'Add Item')}
              </button>
            </div>
          </form>
          <aside className="screen-aside">
            <LivePreviewBox title="Depreciation Preview" rows={previewRows} isLoading={previewLoading} />
          </aside>
        </div>
      </div>
    );
  }

  return (
    <div className="screen">
      <div className="screen-header">
        <h2 className="screen-title">Equipment (Category C2)</h2>
        <p className="screen-description">
          Add equipment items. Depreciation is calculated automatically using the EU formula.
        </p>
        <button className="btn btn--primary" onClick={openAdd}>+ Add Item</button>
      </div>
      <div className="item-list">
        {items.length === 0 ? (
          <EmptyStateCard icon="💻" title="No equipment items yet"
            description="Add any equipment purchased during the grant period."
            action={{ label: '+ Add First Item', onClick: openAdd }} />
        ) : (
          items.map((item) => (
            <EquipmentCard key={item.id} item={item} onEdit={openEdit} onDelete={handleDelete} />
          ))
        )}
      </div>
      <div className="screen-footer">
        <button className="btn btn--ghost" onClick={onBack}>← Back</button>
        <button className="btn btn--primary btn--lg" onClick={onNext}>Next: Travel →</button>
      </div>
    </div>
  );
}
