/**
 * M-01: Project Dashboard — Sprint E1 scaffold. Shows the project header and
 * planned budget totals only; actuals/progress/warnings panels are added in
 * Sprints E2–E3 once financial_engine/progress_engine/notification_engine
 * exist (see docs/executer/execution-architecture.md §7).
 */

import { useExecutionStore } from '../store/executionStore';

export function Dashboard() {
  const summary = useExecutionStore((s) => s.summary);

  if (!summary) return null;

  const { project_info, planned } = summary;

  return (
    <div className="dashboard-screen">
      <header className="project-header">
        <h1>{project_info.project_title}</h1>
        <p>
          {project_info.pi_name} · {project_info.call_reference} ·{' '}
          {project_info.duration_years} year(s) · {project_info.work_package_count} WPs
        </p>
      </header>

      <section className="budget-summary-panel">
        <h2>Planned Budget</h2>
        <dl>
          <dt>Category A (Personnel)</dt>
          <dd>{planned.category_a_total}</dd>
          <dt>Category B (Subcontracting)</dt>
          <dd>{planned.category_b_total}</dd>
          <dt>Category C1 (Travel)</dt>
          <dd>{planned.category_c1_total}</dd>
          <dt>Category C2 (Equipment)</dt>
          <dd>{planned.category_c2_total}</dd>
          <dt>Category C3 (Other)</dt>
          <dd>{planned.category_c3_total}</dd>
          <dt>Category E (Indirect)</dt>
          <dd>{planned.category_e_total}</dd>
          <dt>Requested EU Contribution</dt>
          <dd>{planned.requested_eu_contribution}</dd>
        </dl>
      </section>
    </div>
  );
}
