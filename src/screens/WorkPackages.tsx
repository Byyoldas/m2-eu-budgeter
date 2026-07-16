/**
 * Step 3 — Work Packages.
 * Names the WPs defined in Project Setup (count is fixed there) and assigns
 * each one a start/end month, visualised live as a Gantt-style timeline.
 * Comes before Personnel so real WP timelines exist before roles are entered
 * (Personnel's per-WP cost split is derived from how a role's Start/End Month
 * overlaps each WP's timeline). Submitting calls `updateProjectConfig` to
 * persist the timelines to the backend project state — this used to be a
 * client-store-only update, which meant the backend's WP timelines silently
 * stayed at their untouched full-project-span defaults and every Personnel
 * split came out even no matter what was entered here.
 */

import { useForm } from 'react-hook-form';
import { useProjectStore } from '../store/projectStore';
import { updateProjectConfig } from '../ipc/commands';
import { useBudgetSummary } from '../hooks/useBudgetSummary';
import { WarningBanner } from '../components/WarningBanner';
import { WorkPackageGanttChart } from '../components/WorkPackageGanttChart';

interface WorkPackagesProps {
  onNext: () => void;
  onBack: () => void;
}

interface FormData {
  work_package_names: { name: string }[];
  work_package_start_months: number[];
  work_package_end_months: number[];
}

export function WorkPackages({ onNext, onBack }: WorkPackagesProps) {
  const projectConfig = useProjectStore((s) => s.projectConfig);
  const setProjectConfig = useProjectStore((s) => s.setProjectConfig);
  const { mutate, isLoading, formError } = useBudgetSummary();

  const count = projectConfig?.work_package_count ?? 1;
  const duration = projectConfig?.duration_years ?? 5;
  const durationMonths = duration * 12;
  const existingNames = projectConfig?.work_package_names ?? [];
  const existingStarts = projectConfig?.work_package_start_months ?? [];
  const existingEnds = projectConfig?.work_package_end_months ?? [];

  const { register, handleSubmit, watch, setValue } = useForm<FormData>({
    defaultValues: {
      work_package_names: Array.from({ length: count }, (_, i) => ({
        name: (existingNames[i] as string | null) ?? '',
      })),
      work_package_start_months: Array.from({ length: count }, (_, i) => existingStarts[i] ?? 1),
      work_package_end_months: Array.from({ length: count }, (_, i) => existingEnds[i] ?? durationMonths),
    },
  });

  const watchedNames = watch('work_package_names');
  const watchedStarts = watch('work_package_start_months');
  const watchedEnds = watch('work_package_end_months');

  const handleStartChange = (i: number, value: number) => {
    setValue(`work_package_start_months.${i}`, value);
    if ((watchedEnds[i] ?? durationMonths) < value) {
      setValue(`work_package_end_months.${i}`, value);
    }
  };

  const onSubmit = async (data: FormData) => {
    if (!projectConfig) return;
    const fullConfig = {
      ...projectConfig,
      work_package_names: data.work_package_names.map((wp) => wp.name || null),
      work_package_start_months: data.work_package_start_months,
      work_package_end_months: data.work_package_end_months,
    };

    const result = await mutate(() => updateProjectConfig(fullConfig));
    if (result) {
      setProjectConfig(fullConfig);
      onNext();
    }
  };

  return (
    <div className="screen">
      <div className="screen-header">
        <h2 className="screen-title">Work Packages</h2>
        <p className="screen-description">
          Assign names and an active month range to your {count} Work Package{count > 1 ? 's' : ''}.
          Names are optional — they're used as labels when assigning budget lines to WPs.
        </p>
      </div>

      {formError && <WarningBanner message={formError} severity="error" />}

      <form className="screen-form" onSubmit={handleSubmit(onSubmit)} noValidate>
        <div className="form-section">
          {Array.from({ length: count }, (_, i) => (
            <div className="form-row" key={i}>
              <div className="form-field">
                <label htmlFor={`wp-${i}`} className="form-label">
                  WP{i + 1} Name
                </label>
                <input
                  id={`wp-${i}`}
                  type="text"
                  placeholder={`e.g. WP${i + 1}: Data Collection`}
                  className="form-input"
                  {...register(`work_package_names.${i}.name`)}
                />
              </div>
              <div className="form-field">
                <label htmlFor={`wp-${i}-start`} className="form-label">Start Month</label>
                <input
                  id={`wp-${i}-start`}
                  type="number"
                  min={1}
                  max={durationMonths}
                  className="form-input"
                  {...register(`work_package_start_months.${i}`, {
                    valueAsNumber: true,
                    onChange: (e) => handleStartChange(i, Number(e.target.value)),
                  })}
                />
              </div>
              <div className="form-field">
                <label htmlFor={`wp-${i}-end`} className="form-label">End Month</label>
                <input
                  id={`wp-${i}-end`}
                  type="number"
                  min={watchedStarts[i] ?? 1}
                  max={durationMonths}
                  className="form-input"
                  {...register(`work_package_end_months.${i}`, { valueAsNumber: true })}
                />
              </div>
            </div>
          ))}
        </div>

        <WorkPackageGanttChart
          names={watchedNames.map((wp) => wp.name)}
          startMonths={watchedStarts}
          endMonths={watchedEnds}
          durationMonths={durationMonths}
        />

        <div className="screen-footer">
          <button type="button" className="btn btn--ghost" onClick={onBack}>← Back</button>
          <button type="submit" className="btn btn--primary btn--lg" disabled={isLoading}>
            {isLoading ? 'Saving…' : 'Next: Personnel →'}
          </button>
        </div>
      </form>
    </div>
  );
}
