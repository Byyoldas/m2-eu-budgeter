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

/**
 * Month ranges no Work Package covers (1-indexed, inclusive). The backend
 * requires the Work Packages to cover the whole project (overlaps are fine);
 * checking here lets the user see *which* months are missing before submitting.
 * A range that is empty, non-numeric or backwards covers nothing.
 */
function uncoveredRanges(starts: number[], ends: number[], totalMonths: number): [number, number][] {
  const covered = new Array<boolean>(totalMonths).fill(false);
  starts.forEach((start, i) => {
    const end = ends[i];
    if (!Number.isFinite(start) || !Number.isFinite(end) || start < 1 || end < start) return;
    for (let m = start; m <= Math.min(end, totalMonths); m++) covered[m - 1] = true;
  });
  const gaps: [number, number][] = [];
  for (let m = 1; m <= totalMonths; m++) {
    if (covered[m - 1]) continue;
    const from = m;
    while (m < totalMonths && !covered[m]) m++;
    gaps.push([from, m]);
  }
  return gaps;
}

interface FormData {
  work_package_names: { name: string }[];
  work_package_start_months: number[];
  work_package_end_months: number[];
}

export function WorkPackages({ onNext, onBack }: WorkPackagesProps) {
  const projectConfig = useProjectStore((s) => s.projectConfig);
  const setProjectConfig = useProjectStore((s) => s.setProjectConfig);
  const { mutate, isLoading, formError, fieldErrors } = useBudgetSummary();

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

  const gaps = uncoveredRanges(watchedStarts, watchedEnds, durationMonths);
  const backwards = watchedStarts
    .map((start, i) => ({ wp: i + 1, start, end: watchedEnds[i] }))
    .filter(({ start, end }) => Number.isFinite(start) && Number.isFinite(end) && end < start);
  // Field-level validation errors (e.g. "WP2 duration is invalid", "Month 25
  // is not covered by any Work Package") have no input of their own on this
  // screen, so without this they were swallowed and "Next" appeared to do
  // nothing. Show every distinct message the backend sent.
  const serverMessages = Array.from(new Set(fieldErrors.map((e) => e.message)));

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
      {serverMessages.length > 0 && <WarningBanner message={serverMessages.join(' ')} severity="error" />}

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

        {backwards.length > 0 && (
          <WarningBanner
            severity="warning"
            message={`${backwards.map(({ wp }) => `WP${wp}`).join(', ')}: the end month is before the start month.`}
          />
        )}
        {gaps.length > 0 && (
          <WarningBanner
            severity="warning"
            message={`${gaps.map(([a, b]) => (a === b ? `Month ${a}` : `Months ${a}–${b}`)).join(', ')} not covered by any Work Package. The Work Packages must together cover the whole ${durationMonths}-month project (overlaps are fine).`}
          />
        )}

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
