/**
 * Step 3 — Work Packages.
 * Names the WPs defined in Project Setup (count is fixed there) and assigns each
 * one a start/end year, visualised live as a Gantt-style timeline.
 * Stored in projectConfig; doesn't trigger recalculation (informational only in v1).
 */

import { useForm } from 'react-hook-form';
import { useProjectStore } from '../store/projectStore';
import { WorkPackageGanttChart } from '../components/WorkPackageGanttChart';

interface WorkPackagesProps {
  onNext: () => void;
  onBack: () => void;
}

interface FormData {
  work_package_names: { name: string }[];
  work_package_start_years: number[];
  work_package_end_years: number[];
}

export function WorkPackages({ onNext, onBack }: WorkPackagesProps) {
  const projectConfig = useProjectStore((s) => s.projectConfig);
  const setProjectConfig = useProjectStore((s) => s.setProjectConfig);

  const count = projectConfig?.work_package_count ?? 1;
  const duration = projectConfig?.duration_years ?? 5;
  const existingNames = projectConfig?.work_package_names ?? [];
  const existingStarts = projectConfig?.work_package_start_years ?? [];
  const existingEnds = projectConfig?.work_package_end_years ?? [];

  const { register, handleSubmit, watch, setValue } = useForm<FormData>({
    defaultValues: {
      work_package_names: Array.from({ length: count }, (_, i) => ({
        name: (existingNames[i] as string | null) ?? '',
      })),
      work_package_start_years: Array.from({ length: count }, (_, i) => existingStarts[i] ?? 1),
      work_package_end_years: Array.from({ length: count }, (_, i) => existingEnds[i] ?? duration),
    },
  });

  const watchedNames = watch('work_package_names');
  const watchedStarts = watch('work_package_start_years');
  const watchedEnds = watch('work_package_end_years');

  const handleStartChange = (i: number, value: number) => {
    setValue(`work_package_start_years.${i}`, value);
    if ((watchedEnds[i] ?? duration) < value) {
      setValue(`work_package_end_years.${i}`, value);
    }
  };

  const onSubmit = (data: FormData) => {
    if (!projectConfig) return;
    setProjectConfig({
      ...projectConfig,
      work_package_names: data.work_package_names.map((wp) => wp.name || null),
      work_package_start_years: data.work_package_start_years,
      work_package_end_years: data.work_package_end_years,
    });
    onNext();
  };

  const yearOptions = Array.from({ length: duration }, (_, i) => i + 1);

  return (
    <div className="screen">
      <div className="screen-header">
        <h2 className="screen-title">Work Packages</h2>
        <p className="screen-description">
          Assign names and active years to your {count} Work Package{count > 1 ? 's' : ''}.
          Names are optional — they're used as labels when assigning budget lines to WPs.
        </p>
      </div>

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
                <label htmlFor={`wp-${i}-start`} className="form-label">Start Year</label>
                <select
                  id={`wp-${i}-start`}
                  className="form-input"
                  {...register(`work_package_start_years.${i}`, {
                    valueAsNumber: true,
                    onChange: (e) => handleStartChange(i, Number(e.target.value)),
                  })}
                >
                  {yearOptions.map((y) => (
                    <option key={y} value={y}>Year {y}</option>
                  ))}
                </select>
              </div>
              <div className="form-field">
                <label htmlFor={`wp-${i}-end`} className="form-label">End Year</label>
                <select
                  id={`wp-${i}-end`}
                  className="form-input"
                  {...register(`work_package_end_years.${i}`, { valueAsNumber: true })}
                >
                  {yearOptions
                    .filter((y) => y >= (watchedStarts[i] ?? 1))
                    .map((y) => (
                      <option key={y} value={y}>Year {y}</option>
                    ))}
                </select>
              </div>
            </div>
          ))}
        </div>

        <WorkPackageGanttChart
          names={watchedNames.map((wp) => wp.name)}
          startYears={watchedStarts}
          endYears={watchedEnds}
          durationYears={duration}
        />

        <div className="screen-footer">
          <button type="button" className="btn btn--ghost" onClick={onBack}>← Back</button>
          <button type="submit" className="btn btn--primary btn--lg">
            Next: Personnel →
          </button>
        </div>
      </form>
    </div>
  );
}
