/**
 * M-20: Excel / PDF Export. Every export is a static snapshot of the
 * current `summary` (BR-EX-01) — no separate export-time recalculation.
 */

import { useState } from 'react';
import { useExecutionStore } from '../store/executionStore';
import {
  exportFinancialReport,
  exportTechnicalReportAnnex,
  exportRiskRegister,
  exportPersonMonthDeclaration,
} from '../export/excelExporter';
import { exportProjectStatusReport } from '../export/pdfExporter';
import type { ExecutionProjectSummaryDto } from '../types';

const EXPORTS: {
  label: string;
  description: string;
  run: (summary: ExecutionProjectSummaryDto) => void | Promise<void>;
}[] = [
  {
    label: 'Financial Report (Excel)',
    description: 'Planned vs. actual per category and per Work Package.',
    run: exportFinancialReport,
  },
  {
    label: 'Technical Report Annex (Excel)',
    description: 'Deliverable and milestone status tables.',
    run: exportTechnicalReportAnnex,
  },
  {
    label: 'Project Status Report (PDF)',
    description: 'One-page dashboard export for the PI/coordinator.',
    run: exportProjectStatusReport,
  },
  {
    label: 'Risk Register (Excel)',
    description: 'Full risk register for review meetings.',
    run: exportRiskRegister,
  },
  {
    label: 'Person-Month Declaration (Excel)',
    description: 'Pre-filled template per reporting period per role.',
    run: exportPersonMonthDeclaration,
  },
];

export function ReportsExport() {
  const summary = useExecutionStore((s) => s.summary);
  const [error, setError] = useState<string | null>(null);

  if (!summary) return null;

  const run = async (fn: (typeof EXPORTS)[number]['run']) => {
    setError(null);
    try {
      await fn(summary);
    } catch {
      setError('Export failed. Please try again.');
    }
  };

  return (
    <div className="screen">
      <h1>Reports &amp; Export</h1>
      {error && <p className="error-banner">{error}</p>}
      <div className="export-grid">
        {EXPORTS.map((e) => (
          <div key={e.label} className="export-card">
            <h3>{e.label}</h3>
            <p>{e.description}</p>
            <button onClick={() => run(e.run)}>Export</button>
          </div>
        ))}
      </div>
    </div>
  );
}
