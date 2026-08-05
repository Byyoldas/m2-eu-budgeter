/**
 * M-01: Project Dashboard. Sprint E1 built the project header + planned
 * budget; Sprint E3 adds the planned-vs-actual panel (M-07) now that
 * financial_engine exists (see docs/executer/execution-architecture.md §7).
 * Progress/warnings panels are still deferred — no notification_engine yet.
 */

import { useExecutionStore } from '../store/executionStore';
import { fmtEur } from '../utils/currency';

const CFS_LABELS: Record<string, string> = {
  NOT_REQUIRED: 'Not Required',
  REQUIRED_AND_PRESENT: 'Required — Present',
  REQUIRED_BUT_DISMISSED: 'Required — Dismissed',
  REQUIRED_AND_UNADDRESSED: 'Required — Unaddressed',
};

export function Dashboard() {
  const summary = useExecutionStore((s) => s.summary);

  if (!summary) return null;

  const { project_info, planned, actuals, current_project_month } = summary;
  const timeElapsedPct =
    project_info.duration_years > 0
      ? Math.round((current_project_month / (project_info.duration_years * 12)) * 100)
      : 0;

  return (
    <div className="dashboard-screen">
      <header className="project-header">
        <h1>{project_info.project_title}</h1>
        <p>
          {project_info.pi_name} · {project_info.call_reference} ·{' '}
          {project_info.duration_years} year(s) · {project_info.work_package_count} WPs · Month{' '}
          {current_project_month} ({timeElapsedPct}% elapsed)
        </p>
      </header>

      <section className="budget-summary-panel">
        <h2>Planned vs. Actual</h2>
        <table>
          <thead>
            <tr>
              <th>Category</th>
              <th>Planned</th>
              <th>Actual</th>
              <th />
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>A (Personnel)</td>
              <td>{fmtEur(planned.category_a_total)}</td>
              <td>{fmtEur(actuals.a_actual)}</td>
              <td>{actuals.category_a_overrun && <span className="warning-banner">&gt;15%</span>}</td>
            </tr>
            <tr>
              <td>B (Subcontracting)</td>
              <td>{fmtEur(planned.category_b_total)}</td>
              <td>{fmtEur(actuals.b_actual)}</td>
              <td>{actuals.category_b_overrun && <span className="warning-banner">&gt;15%</span>}</td>
            </tr>
            <tr>
              <td>C1 (Travel)</td>
              <td>{fmtEur(planned.category_c1_total)}</td>
              <td>{fmtEur(actuals.c1_actual)}</td>
              <td>{actuals.category_c1_overrun && <span className="warning-banner">&gt;15%</span>}</td>
            </tr>
            <tr>
              <td>C2 (Equipment)</td>
              <td>{fmtEur(planned.category_c2_total)}</td>
              <td>{fmtEur(actuals.c2_actual)}</td>
              <td>{actuals.category_c2_overrun && <span className="warning-banner">&gt;15%</span>}</td>
            </tr>
            <tr>
              <td>C3 (Other)</td>
              <td>{fmtEur(planned.category_c3_total)}</td>
              <td>{fmtEur(actuals.c3_actual)}</td>
              <td>{actuals.category_c3_overrun && <span className="warning-banner">&gt;15%</span>}</td>
            </tr>
            <tr>
              <td>E (Indirect)</td>
              <td>{fmtEur(planned.category_e_total)}</td>
              <td>{fmtEur(actuals.e_actual)}</td>
              <td />
            </tr>
            <tr>
              <td>
                <strong>Requested EU Contribution</strong>
              </td>
              <td>
                <strong>{fmtEur(planned.requested_eu_contribution)}</strong>
              </td>
              <td>
                <strong>{fmtEur(actuals.requested_eu_contribution_actual)}</strong>
              </td>
              <td />
            </tr>
          </tbody>
        </table>
        <p>CFS status (actual): {CFS_LABELS[actuals.cfs_status_actual] ?? actuals.cfs_status_actual}</p>
      </section>
    </div>
  );
}
