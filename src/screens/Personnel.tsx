/**
 * Step 4 — Personnel (Category A).
 * Lists existing roles, opens add/edit form, handles live preview.
 */

import { useState, useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { personnelRoleSchema, type PersonnelRoleFormData } from '../validators/schemas';
import { useProjectStore, usePersonnelRoles } from '../store/projectStore';
import { addPersonnelRole, updatePersonnelRole, deletePersonnelRole, previewRoleCost } from '../ipc/commands';
import { useBudgetSummary, usePreview } from '../hooks/useBudgetSummary';
import { RoleCard } from '../components/RoleCard';
import { EmptyStateCard } from '../components/EmptyStateCard';
import { LivePreviewBox } from '../components/LivePreviewBox';
import { WarningBanner } from '../components/WarningBanner';
import type { PersonnelRoleDetailDto, RoleCostPreviewDto, PersonnelRoleInput } from '../types';

interface PersonnelProps {
  onNext: () => void;
  onBack: () => void;
}

type Mode = 'list' | 'add' | 'edit';

const ROLE_TYPES = [
  { value: 'Pi', label: 'Principal Investigator' },
  { value: 'Expert', label: 'Expert / Senior Researcher' },
  { value: 'PostDoc', label: 'Post-Doctoral Researcher' },
  { value: 'PhdStudent', label: 'PhD Student' },
  { value: 'MscStudent', label: 'MSc Student' },
  { value: 'Admin', label: 'Administrative Staff' },
];

function fmt(v: string | undefined): string {
  if (!v) return '—';
  const n = parseFloat(v);
  return isNaN(n) ? '—' : `€ ${n.toLocaleString('en-GB', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

export function Personnel({ onNext, onBack }: PersonnelProps) {
  const roles = usePersonnelRoles();
  const projectConfig = useProjectStore((s) => s.projectConfig);
  const duration = projectConfig?.duration_years ?? 5;
  const durationMonths = duration * 12;
  const wpNames = projectConfig?.work_package_names ?? [];

  const [mode, setMode] = useState<Mode>('list');
  const [editingRole, setEditingRole] = useState<PersonnelRoleDetailDto | null>(null);
  const [previewResult, setPreviewResult] = useState<RoleCostPreviewDto | null>(null);

  const { mutate, isLoading, fieldErrors, formError } = useBudgetSummary();
  const { preview, isLoading: previewLoading } = usePreview<RoleCostPreviewDto>();

  const {
    register,
    handleSubmit,
    watch,
    reset,
    formState: { errors },
  } = useForm<PersonnelRoleFormData>({
    resolver: zodResolver(personnelRoleSchema),
    defaultValues: {
      role_type: 'Expert',
      start_month: 1,
      end_month: durationMonths,
      inflation_rate_pct: projectConfig?.default_inflation_rate_pct ?? '10',
    },
  });

  const watchedValues = watch();

  // Live preview debounced
  useEffect(() => {
    const timer = setTimeout(async () => {
      const salary = parseFloat(watchedValues.current_monthly_salary_try);
      const fte = parseFloat(watchedValues.fte_fraction);
      const inflation = parseFloat(watchedValues.inflation_rate_pct);
      const startMonth = Number(watchedValues.start_month);
      const endMonth = Number(watchedValues.end_month);
      if (!salary || !fte || isNaN(salary) || isNaN(fte) || isNaN(inflation) || !startMonth || !endMonth) {
        setPreviewResult(null);
        return;
      }
      const result = await preview(() =>
        previewRoleCost({
          role_label: watchedValues.role_label ?? '',
          role_type: watchedValues.role_type ?? 'Expert',
          current_monthly_salary_try: watchedValues.current_monthly_salary_try,
          fte_fraction: watchedValues.fte_fraction,
          inflation_rate_pct: watchedValues.inflation_rate_pct,
          start_month: startMonth,
          end_month: endMonth,
        })
      );
      setPreviewResult(result);
    }, 400);
    return () => clearTimeout(timer);
  }, [JSON.stringify(watchedValues)]);

  const openAdd = () => {
    reset({
      role_type: 'Expert',
      start_month: 1,
      end_month: durationMonths,
      inflation_rate_pct: projectConfig?.default_inflation_rate_pct ?? '10',
    });
    setEditingRole(null);
    setPreviewResult(null);
    setMode('add');
  };

  const openEdit = (role: PersonnelRoleDetailDto) => {
    setEditingRole(role);
    reset({
      role_label: role.role_label,
      role_type: role.role_type,
      current_monthly_salary_try: role.current_monthly_salary_try,
      fte_fraction: role.fte_fraction,
      inflation_rate_pct: role.inflation_rate_pct,
      start_month: role.start_month,
      end_month: role.end_month,
    });
    setPreviewResult(null);
    setMode('edit');
  };

  const handleDelete = async (id: string) => {
    if (!window.confirm('Delete this personnel role?')) return;
    await mutate(() => deletePersonnelRole(id));
  };

  const fieldError = (field: string) =>
    fieldErrors.find((e) => e.field === field)?.message ??
    (errors as Record<string, { message?: string }>)[field]?.message;

  const onSubmit = async (data: PersonnelRoleFormData) => {
    const input: PersonnelRoleInput = {
      role_label: data.role_label,
      role_type: data.role_type,
      current_monthly_salary_try: data.current_monthly_salary_try,
      fte_fraction: data.fte_fraction,
      inflation_rate_pct: data.inflation_rate_pct,
      start_month: data.start_month,
      end_month: data.end_month,
    };

    const command = editingRole
      ? () => updatePersonnelRole(editingRole.id, input)
      : () => addPersonnelRole(input);

    const result = await mutate(command);
    if (result) setMode('list');
  };

  const previewRows = previewResult
    ? [
        { label: 'Base monthly (EUR)', value: fmt(previewResult.base_monthly_eur) },
        ...previewResult.cost_lines
          .filter((l) => l.is_active)
          .map((l) => ({
            label: `Year ${l.year} annual cost`,
            value: fmt(l.annual_cost_eur),
          })),
        { label: 'Total cost', value: fmt(previewResult.total_cost_eur), highlight: true },
        ...previewResult.wp_breakdown.map((w) => ({
          label: `→ ${(wpNames[w.work_package_id - 1] as string | null) ?? `WP${w.work_package_id}`}`,
          value: fmt(w.amount_eur),
        })),
      ]
    : [];

  if (mode !== 'list') {
    return (
      <div className="screen">
        <div className="screen-header">
          <h2 className="screen-title">{mode === 'add' ? 'Add Personnel Role' : 'Edit Personnel Role'}</h2>
        </div>

        {formError && <WarningBanner message={formError} severity="error" />}

        <div className="screen-split">
          <form className="screen-form" onSubmit={handleSubmit(onSubmit)} noValidate>
            <div className="form-section">
              <div className="form-field">
                <label htmlFor="role_label" className="form-label required">Role Label</label>
                <input
                  id="role_label"
                  type="text"
                  placeholder="e.g. PI, Post-Doc A, Research Assistant"
                  className={`form-input${fieldError('role_label') ? ' form-input--error' : ''}`}
                  {...register('role_label')}
                />
                {fieldError('role_label') && <span className="form-error">{fieldError('role_label')}</span>}
              </div>

              <div className="form-field">
                <label htmlFor="role_type" className="form-label required">Role Type</label>
                <select
                  id="role_type"
                  className="form-input"
                  {...register('role_type')}
                >
                  {ROLE_TYPES.map((r) => (
                    <option key={r.value} value={r.value}>{r.label}</option>
                  ))}
                </select>
                {fieldError('role_type') && <span className="form-error">{fieldError('role_type')}</span>}
              </div>

              <div className="form-row">
                <div className="form-field">
                  <label htmlFor="current_monthly_salary_try" className="form-label required">
                    Current Monthly Salary (TRY)
                  </label>
                  <input
                    id="current_monthly_salary_try"
                    type="number"
                    step="any"
                    placeholder="e.g. 227900"
                    className={`form-input${fieldError('current_monthly_salary_try') ? ' form-input--error' : ''}`}
                    {...register('current_monthly_salary_try')}
                  />
                  {fieldError('current_monthly_salary_try') && (
                    <span className="form-error">{fieldError('current_monthly_salary_try')}</span>
                  )}
                </div>

                <div className="form-field">
                  <label htmlFor="fte_fraction" className="form-label required">PM (0–1)</label>
                  <input
                    id="fte_fraction"
                    type="number"
                    step="0.01"
                    min={0.01}
                    max={1}
                    placeholder="e.g. 0.7"
                    className={`form-input${fieldError('fte_fraction') ? ' form-input--error' : ''}`}
                    {...register('fte_fraction')}
                  />
                  {fieldError('fte_fraction') && <span className="form-error">{fieldError('fte_fraction')}</span>}
                </div>
              </div>

              <div className="form-field">
                <label htmlFor="inflation_rate_pct" className="form-label required">
                  Annual Salary Inflation (%)
                </label>
                <input
                  id="inflation_rate_pct"
                  type="number"
                  step="any"
                  min={0}
                  max={100}
                  className={`form-input${fieldError('inflation_rate_pct') ? ' form-input--error' : ''}`}
                  {...register('inflation_rate_pct')}
                />
                {fieldError('inflation_rate_pct') && (
                  <span className="form-error">{fieldError('inflation_rate_pct')}</span>
                )}
                <span className="form-hint">Leave at project default or override per role.</span>
              </div>

              <div className="form-row">
                <div className="form-field">
                  <label htmlFor="start_month" className="form-label required">Start Month</label>
                  <input
                    id="start_month"
                    type="number"
                    min={1}
                    max={durationMonths}
                    className={`form-input${fieldError('start_month') ? ' form-input--error' : ''}`}
                    {...register('start_month', { valueAsNumber: true })}
                  />
                  {fieldError('start_month') && <span className="form-error">{fieldError('start_month')}</span>}
                </div>
                <div className="form-field">
                  <label htmlFor="end_month" className="form-label required">End Month</label>
                  <input
                    id="end_month"
                    type="number"
                    min={1}
                    max={durationMonths}
                    className={`form-input${fieldError('end_month') ? ' form-input--error' : ''}`}
                    {...register('end_month', { valueAsNumber: true })}
                  />
                  {fieldError('end_month') && <span className="form-error">{fieldError('end_month')}</span>}
                </div>
              </div>
              <span className="form-hint">
                Project runs months 1–{durationMonths}. The Work Package(s) this role's cost is
                charged to are determined automatically from this range.
              </span>
            </div>

            <div className="screen-footer">
              <button type="button" className="btn btn--ghost" onClick={() => setMode('list')}>
                Cancel
              </button>
              <button type="submit" className="btn btn--primary" disabled={isLoading}>
                {isLoading ? 'Saving…' : (editingRole ? 'Update Role' : 'Add Role')}
              </button>
            </div>
          </form>

          <aside className="screen-aside">
            <LivePreviewBox
              title="Cost Preview"
              rows={previewRows}
              isLoading={previewLoading}
            />
          </aside>
        </div>
      </div>
    );
  }

  return (
    <div className="screen">
      <div className="screen-header">
        <h2 className="screen-title">Personnel (Category A)</h2>
        <p className="screen-description">
          Add all staff whose salaries will be charged to the grant.
          The application calculates EUR costs and salary inflation automatically.
        </p>
        <button className="btn btn--primary" onClick={openAdd}>+ Add Role</button>
      </div>

      <div className="item-list">
        {roles.length === 0 ? (
          <EmptyStateCard
            icon="👤"
            title="No personnel roles yet"
            description="Add the PI and any researchers, post-docs, or admin staff funded by this grant."
            action={{ label: '+ Add First Role', onClick: openAdd }}
          />
        ) : (
          roles.map((role) => (
            <RoleCard key={role.id} role={role} onEdit={openEdit} onDelete={handleDelete} />
          ))
        )}
      </div>

      <div className="screen-footer">
        <button className="btn btn--ghost" onClick={onBack}>← Back</button>
        <button className="btn btn--primary btn--lg" onClick={onNext}>
          Next: Equipment →
        </button>
      </div>
    </div>
  );
}
