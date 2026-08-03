/**
 * M-14: Reporting Period Management. Periods are pre-populated on project
 * open (BR-RP-05); this screen lets the PI set a real deadline, toggle the
 * two report-submission flags, and mark a period `Submitted` once both are
 * set (BR-RP-03, enforced server-side). BR-RP-01/02 coverage gaps are shown
 * as an advisory banner, never blocking.
 */

import { useState } from 'react';
import { useExecutionStore } from '../store/executionStore';
import { useExecutionMutation } from '../hooks/useExecutionMutation';
import { addReportingPeriod, deleteReportingPeriod, updateReportingPeriod } from '../ipc/commands';
import type { ReportingPeriodDetailDto, ReportingPeriodInputDto } from '../types';

const emptyForm: ReportingPeriodInputDto = {
  start_month: 1,
  end_month: 1,
  submission_deadline: null,
  technical_report_submitted: false,
  financial_report_submitted: false,
  status: 'Open',
};

function toInputDto(p: ReportingPeriodDetailDto): ReportingPeriodInputDto {
  return {
    start_month: p.start_month,
    end_month: p.end_month,
    submission_deadline: p.submission_deadline,
    technical_report_submitted: p.technical_report_submitted,
    financial_report_submitted: p.financial_report_submitted,
    status: p.status,
  };
}

export function ReportingPeriods() {
  const summary = useExecutionStore((s) => s.summary);
  const { run, error, isSubmitting } = useExecutionMutation();
  const [form, setForm] = useState(emptyForm);

  if (!summary) return null;
  const { reporting_periods, reporting_period_coverage } = summary;

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    const ok = await run(() => addReportingPeriod(form));
    if (ok) setForm(emptyForm);
  };

  const toggleFlag = (p: ReportingPeriodDetailDto, field: 'technical_report_submitted' | 'financial_report_submitted') =>
    run(() => updateReportingPeriod(p.id, { ...toInputDto(p), [field]: !p[field] }));

  const setDeadline = (p: ReportingPeriodDetailDto, deadline: string) =>
    run(() => updateReportingPeriod(p.id, { ...toInputDto(p), submission_deadline: deadline || null }));

  const submitPeriod = (p: ReportingPeriodDetailDto) =>
    run(() => updateReportingPeriod(p.id, { ...toInputDto(p), status: 'Submitted' }));

  return (
    <div className="screen">
      <h1>Reporting Periods</h1>
      {error && <p className="error-banner">{error}</p>}
      {reporting_period_coverage.gaps_detected && (
        <p className="warning-banner">
          These periods don't fully cover the project duration without gaps (BR-RP-01).
        </p>
      )}
      {!reporting_period_coverage.final_period_covers_project_end && (
        <p className="warning-banner">
          The final period doesn't end at the project's last month (BR-RP-02).
        </p>
      )}

      <table>
        <thead>
          <tr>
            <th>Period</th>
            <th>Months</th>
            <th>Deadline</th>
            <th>Deliverables Due</th>
            <th>Technical Report</th>
            <th>Financial Report</th>
            <th>Status</th>
            <th />
          </tr>
        </thead>
        <tbody>
          {reporting_periods.map((p) => (
            <tr key={p.id}>
              <td>P{p.period_number}</td>
              <td>
                M{p.start_month}–M{p.end_month}
              </td>
              <td>
                <input
                  type="date"
                  value={p.submission_deadline ?? ''}
                  onChange={(e) => setDeadline(p, e.target.value)}
                  disabled={isSubmitting}
                />
              </td>
              <td>
                {p.deliverables_submitted} / {p.deliverables_due}
              </td>
              <td>
                <input
                  type="checkbox"
                  checked={p.technical_report_submitted}
                  onChange={() => toggleFlag(p, 'technical_report_submitted')}
                  disabled={isSubmitting}
                />
              </td>
              <td>
                <input
                  type="checkbox"
                  checked={p.financial_report_submitted}
                  onChange={() => toggleFlag(p, 'financial_report_submitted')}
                  disabled={isSubmitting}
                />
              </td>
              <td>{p.status}</td>
              <td>
                {p.status !== 'Submitted' && (
                  <button onClick={() => submitPeriod(p)} disabled={isSubmitting}>
                    Mark Submitted
                  </button>
                )}
                <button onClick={() => run(() => deleteReportingPeriod(p.id))} disabled={isSubmitting}>
                  Delete
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>

      <form onSubmit={submit} className="inline-form">
        <input
          type="number"
          min={1}
          placeholder="Start month"
          value={form.start_month}
          onChange={(e) => setForm({ ...form, start_month: Number(e.target.value) })}
          required
        />
        <input
          type="number"
          min={1}
          placeholder="End month"
          value={form.end_month}
          onChange={(e) => setForm({ ...form, end_month: Number(e.target.value) })}
          required
        />
        <button type="submit" disabled={isSubmitting}>
          Add Period
        </button>
      </form>
    </div>
  );
}
