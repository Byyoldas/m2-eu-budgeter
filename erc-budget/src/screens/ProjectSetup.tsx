/**
 * Step 1 — Project Setup.
 * Collects: title, PI name, call reference, funding model (Actual Cost / Lumpsum),
 * duration, WP count (Lumpsum only), call opening date.
 * Does NOT yet create the project in the backend (that happens in BudgetSettings).
 */

import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { projectSetupSchema, type ProjectSetupFormData } from '../validators/schemas';
import { useProjectStore } from '../store/projectStore';

interface ProjectSetupProps {
  onNext: () => void;
}

export function ProjectSetup({ onNext }: ProjectSetupProps) {
  const projectConfig = useProjectStore((s) => s.projectConfig);
  const setProjectConfig = useProjectStore((s) => s.setProjectConfig);

  const {
    register,
    handleSubmit,
    watch,
    setValue,
    getValues,
    formState: { errors },
  } = useForm<ProjectSetupFormData>({
    resolver: zodResolver(projectSetupSchema),
    defaultValues: {
      project_title: projectConfig?.project_title ?? '',
      pi_name: projectConfig?.pi_name ?? '',
      call_reference: projectConfig?.call_reference ?? '',
      duration_years: projectConfig?.duration_years ?? 5,
      work_package_count: projectConfig?.work_package_count ?? 3,
      call_opening_date: projectConfig?.call_opening_date ?? '',
      funding_model: projectConfig?.funding_model ?? 'LumpSum',
    },
  });

  const fundingModel = watch('funding_model');
  const isActualCost = fundingModel === 'ActualCost';

  const onSubmit = (data: ProjectSetupFormData) => {
    // Actual Cost has no Work Packages: the project is one implicit WP that
    // spans the whole duration (the backend enforces the same collapse).
    const wpCount = data.funding_model === 'ActualCost' ? 1 : data.work_package_count;
    // Merge into existing config (budget settings fields are set in the next step)
    setProjectConfig({
      project_title: data.project_title,
      pi_name: data.pi_name,
      call_reference: data.call_reference,
      duration_years: data.duration_years,
      funding_model: data.funding_model,
      work_package_count: wpCount,
      work_package_names: Array.from({ length: wpCount }, (_, i) =>
        projectConfig?.work_package_names?.[i] ?? null
      ),
      // Default each WP to span the full project; preserved across edits unless
      // it would fall outside the (possibly changed) duration. Actual Cost
      // always spans the full project.
      work_package_start_months: Array.from({ length: wpCount }, (_, i) => {
        if (data.funding_model === 'ActualCost') return 1;
        const durationMonths = data.duration_years * 12;
        const existing = projectConfig?.work_package_start_months?.[i];
        return existing && existing <= durationMonths ? existing : 1;
      }),
      work_package_end_months: Array.from({ length: wpCount }, (_, i) => {
        const durationMonths = data.duration_years * 12;
        if (data.funding_model === 'ActualCost') return durationMonths;
        const existing = projectConfig?.work_package_end_months?.[i];
        return existing && existing <= durationMonths ? existing : durationMonths;
      }),
      // Preserve budget settings from previous step (if editing)
      default_inflation_rate_pct: projectConfig?.default_inflation_rate_pct ?? '10',
      try_eur_rate: projectConfig?.try_eur_rate ?? '50.62',
      indirect_cost_rate_pct: projectConfig?.indirect_cost_rate_pct ?? '25',
      rate_version_id: projectConfig?.rate_version_id ?? 'from_2025_05_13',
      call_opening_date: data.call_opening_date || null,
      // No input on this screen -- entered in ERC Execution instead. Just
      // preserve whatever's already there so re-submitting this form
      // doesn't silently wipe it out.
      project_start_date: projectConfig?.project_start_date ?? null,
    });
    onNext();
  };

  return (
    <div className="screen">
      <div className="screen-header">
        <h2 className="screen-title">Project Setup</h2>
        <p className="screen-description">
          Basic information about your ERC Consolidator Grant project.
        </p>
      </div>

      <form className="screen-form" onSubmit={handleSubmit(onSubmit)} noValidate>
        <div className="form-section">
          <h3 className="form-section-title">Project Identity</h3>

          <div className="form-field">
            <label htmlFor="project_title" className="form-label required">Project Title</label>
            <input
              id="project_title"
              type="text"
              placeholder="e.g. My ERC CoG Project"
              className={`form-input${errors.project_title ? ' form-input--error' : ''}`}
              {...register('project_title')}
            />
            {errors.project_title && (
              <span className="form-error">{errors.project_title.message}</span>
            )}
          </div>

          <div className="form-field">
            <label htmlFor="pi_name" className="form-label required">Principal Investigator</label>
            <input
              id="pi_name"
              type="text"
              placeholder="Full name of the PI"
              className={`form-input${errors.pi_name ? ' form-input--error' : ''}`}
              {...register('pi_name')}
            />
            {errors.pi_name && (
              <span className="form-error">{errors.pi_name.message}</span>
            )}
          </div>

          <div className="form-field">
            <label htmlFor="call_reference" className="form-label required">Call Reference</label>
            <input
              id="call_reference"
              type="text"
              placeholder="e.g. ERC-2025-CoG"
              className={`form-input${errors.call_reference ? ' form-input--error' : ''}`}
              {...register('call_reference')}
            />
            {errors.call_reference && (
              <span className="form-error">{errors.call_reference.message}</span>
            )}
          </div>

          <div className="form-field">
            <label htmlFor="call_opening_date" className="form-label">Call Opening Date</label>
            <input
              id="call_opening_date"
              type="date"
              className="form-input"
              {...register('call_opening_date')}
            />
            <span className="form-hint">
              Used to select the correct EU travel rate version automatically.
            </span>
          </div>
        </div>

        <div className="form-section">
          <h3 className="form-section-title">Funding Model</h3>

          <div className="form-field">
            <label className="form-label required" id="funding-model-label">How is this grant funded?</label>
            <div className="segmented-control" role="radiogroup" aria-labelledby="funding-model-label">
              <button
                type="button"
                role="radio"
                aria-checked={!isActualCost}
                className={`seg-btn${!isActualCost ? ' seg-btn--active' : ''}`}
                onClick={() => {
                  setValue('funding_model', 'LumpSum', { shouldDirty: true });
                  // An Actual Cost project is stored as a single implicit Work
                  // Package, so the count this field would show after switching
                  // back is that collapsed 1 — not a real choice. Offer the
                  // usual starting point instead of silently leaving one WP.
                  if (projectConfig?.funding_model === 'ActualCost' && Number(getValues('work_package_count')) <= 1) {
                    setValue('work_package_count', 3, { shouldDirty: true });
                  }
                }}
              >
                Lumpsum Funding
              </button>
              <button
                type="button"
                role="radio"
                aria-checked={isActualCost}
                className={`seg-btn${isActualCost ? ' seg-btn--active' : ''}`}
                onClick={() => setValue('funding_model', 'ActualCost', { shouldDirty: true })}
              >
                Actual Cost
              </button>
            </div>
            <span className="form-hint">
              {isActualCost
                ? 'Actual Cost: the budget is built from total costs per category. There is no Work Package breakdown — cost items need no Work Package, and the Work Packages step is skipped.'
                : 'Lumpsum Funding: the budget is organised by Work Package. Define each Work Package\'s timeline, then tag every cost to the Work Package(s) it belongs to.'}
            </span>
          </div>
        </div>

        <div className="form-section">
          <h3 className="form-section-title">Project Structure</h3>

          <div className="form-row">
            <div className="form-field">
              <label htmlFor="duration_years" className="form-label required">Duration (years)</label>
              <input
                id="duration_years"
                type="number"
                min={1}
                max={7}
                className={`form-input${errors.duration_years ? ' form-input--error' : ''}`}
                {...register('duration_years', { valueAsNumber: true })}
              />
              {errors.duration_years && (
                <span className="form-error">{errors.duration_years.message}</span>
              )}
              <span className="form-hint">ERC CoG: typically 5 years (max 7).</span>
            </div>

            {!isActualCost && (
            <div className="form-field">
              <label htmlFor="work_package_count" className="form-label required">Work Packages</label>
              <input
                id="work_package_count"
                type="number"
                min={1}
                max={20}
                className={`form-input${errors.work_package_count ? ' form-input--error' : ''}`}
                {...register('work_package_count', { valueAsNumber: true })}
              />
              {errors.work_package_count && (
                <span className="form-error">{errors.work_package_count.message}</span>
              )}
            </div>
            )}
          </div>
        </div>

        <div className="screen-footer">
          <button type="submit" className="btn btn--primary btn--lg">
            Next: Budget Settings →
          </button>
        </div>
      </form>
    </div>
  );
}
