/**
 * Step 2 — Budget Settings.
 * Collects: TRY/EUR rate, default inflation, indirect cost rate, rate version.
 * On submit: calls createProject (or updateProjectConfig if project already exists).
 */

import { useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { budgetSettingsSchema, type BudgetSettingsFormData } from '../validators/schemas';
import { useProjectStore } from '../store/projectStore';
import { createProject, updateProjectConfig, getRateVersions } from '../ipc/commands';
import { useBudgetSummary } from '../hooks/useBudgetSummary';

interface BudgetSettingsProps {
  onNext: () => void;
  onBack: () => void;
}

export function BudgetSettings({ onNext, onBack }: BudgetSettingsProps) {
  const projectConfig = useProjectStore((s) => s.projectConfig);
  const setProjectConfig = useProjectStore((s) => s.setProjectConfig);
  const rateVersions = useProjectStore((s) => s.rateVersions);
  const setRateVersions = useProjectStore((s) => s.setRateVersions);
  const summary = useProjectStore((s) => s.summary);
  const { mutate, isLoading, fieldErrors } = useBudgetSummary();

  useEffect(() => {
    if (rateVersions.length === 0) {
      getRateVersions().then(setRateVersions).catch(() => {});
    }
  }, []);

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<BudgetSettingsFormData>({
    resolver: zodResolver(budgetSettingsSchema),
    defaultValues: {
      try_eur_rate: projectConfig?.try_eur_rate ?? '50.62',
      default_inflation_rate_pct: projectConfig?.default_inflation_rate_pct ?? '10',
      indirect_cost_rate_pct: projectConfig?.indirect_cost_rate_pct ?? '25',
      rate_version_id: projectConfig?.rate_version_id ?? (rateVersions[0]?.version_id ?? 'from_2025_05_13'),
    },
  });

  const fieldError = (field: string) =>
    fieldErrors.find((e) => e.field === field)?.message ??
    (errors as Record<string, { message?: string }>)[field]?.message;

  const onSubmit = async (data: BudgetSettingsFormData) => {
    if (!projectConfig) return;

    const fullConfig = {
      ...projectConfig,
      try_eur_rate: data.try_eur_rate,
      default_inflation_rate_pct: data.default_inflation_rate_pct,
      indirect_cost_rate_pct: data.indirect_cost_rate_pct,
      rate_version_id: data.rate_version_id,
    };

    setProjectConfig(fullConfig);

    const command = summary ? updateProjectConfig : createProject;
    const result = await mutate(() => command(fullConfig));
    if (result) onNext();
  };

  return (
    <div className="screen">
      <div className="screen-header">
        <h2 className="screen-title">Budget Settings</h2>
        <p className="screen-description">
          Financial parameters that apply to all calculations in this project.
        </p>
      </div>

      <form className="screen-form" onSubmit={handleSubmit(onSubmit)} noValidate>
        <div className="form-section">
          <h3 className="form-section-title">Currency & Rates</h3>

          <div className="form-field">
            <label htmlFor="try_eur_rate" className="form-label required">
              TRY / EUR Exchange Rate
            </label>
            <input
              id="try_eur_rate"
              type="number"
              step="any"
              placeholder="e.g. 50.62"
              className={`form-input${fieldError('try_eur_rate') ? ' form-input--error' : ''}`}
              {...register('try_eur_rate')}
            />
            {fieldError('try_eur_rate') && (
              <span className="form-error">{fieldError('try_eur_rate')}</span>
            )}
            <span className="form-hint">
              Number of Turkish Lira per 1 Euro. Used to convert all personnel salaries.
            </span>
          </div>

          <div className="form-field">
            <label htmlFor="default_inflation_rate_pct" className="form-label required">
              Default Annual Salary Inflation (%)
            </label>
            <input
              id="default_inflation_rate_pct"
              type="number"
              step="any"
              min={0}
              max={100}
              placeholder="e.g. 10"
              className={`form-input${fieldError('default_inflation_rate_pct') ? ' form-input--error' : ''}`}
              {...register('default_inflation_rate_pct')}
            />
            {fieldError('default_inflation_rate_pct') && (
              <span className="form-error">{fieldError('default_inflation_rate_pct')}</span>
            )}
            <span className="form-hint">
              Applied to each role unless overridden per-role. Typical range: 5%–20%.
            </span>
          </div>

          <div className="form-field">
            <label htmlFor="indirect_cost_rate_pct" className="form-label required">
              Indirect Cost Rate (%)
            </label>
            <input
              id="indirect_cost_rate_pct"
              type="number"
              step="any"
              min={0}
              max={50}
              className={`form-input${fieldError('indirect_cost_rate_pct') ? ' form-input--error' : ''}`}
              {...register('indirect_cost_rate_pct')}
            />
            {fieldError('indirect_cost_rate_pct') && (
              <span className="form-error">{fieldError('indirect_cost_rate_pct')}</span>
            )}
            <span className="form-hint">
              ERC standard is 25%. Applied to A + C1 + C2 + C3 (excluding B).
            </span>
          </div>
        </div>

        <div className="form-section">
          <h3 className="form-section-title">EU Travel Rate Version</h3>

          <div className="form-field">
            <label htmlFor="rate_version_id" className="form-label required">Rate Table Version</label>
            <select
              id="rate_version_id"
              className={`form-input${fieldError('rate_version_id') ? ' form-input--error' : ''}`}
              {...register('rate_version_id')}
            >
              {rateVersions.map((v) => (
                <option key={v.version_id} value={v.version_id}>
                  {v.version_label} (from {v.applicable_from})
                </option>
              ))}
            </select>
            {fieldError('rate_version_id') && (
              <span className="form-error">{fieldError('rate_version_id')}</span>
            )}
            <span className="form-hint">
              Select the version that was active on the call opening date.
            </span>
          </div>
        </div>

        <div className="screen-footer">
          <button type="button" className="btn btn--ghost" onClick={onBack}>← Back</button>
          <button type="submit" className="btn btn--primary btn--lg" disabled={isLoading}>
            {isLoading ? 'Creating project…' : (summary ? 'Save & Continue →' : 'Create Project →')}
          </button>
        </div>
      </form>
    </div>
  );
}
