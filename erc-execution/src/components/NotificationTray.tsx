/**
 * M-21: Notifications & Warnings tray. `warnings` are entirely derived
 * server-side (`engines::notification_engine::evaluate_warnings`, codes
 * W-01 through W-12) — this component only renders them and, on click,
 * navigates to the relevant screen.
 */

import { useState } from 'react';
import { useExecutionStore } from '../store/executionStore';
import type { ExecutionScreen, NavigationTarget, WarningSeverity } from '../types';

const SEVERITY_ICON: Record<WarningSeverity, string> = {
  Error: '🔴',
  Warning: '🟡',
  Info: '⚪',
};

const NAVIGATION_TARGET_TO_SCREEN: Record<NavigationTarget, ExecutionScreen> = {
  Dashboard: 'dashboard',
  WorkPackages: 'work-packages',
  Deliverables: 'deliverables',
  Milestones: 'milestones',
  Personnel: 'personnel',
  Travel: 'travel',
  Equipment: 'equipment',
  ReportingPeriods: 'reporting-periods',
  RiskRegister: 'risk-register',
  IssueLog: 'issue-log',
};

export function NotificationTray() {
  const summary = useExecutionStore((s) => s.summary);
  const setActiveScreen = useExecutionStore((s) => s.setActiveScreen);
  const [open, setOpen] = useState(false);

  if (!summary) return null;
  const { warnings } = summary;

  return (
    <div className="notification-tray">
      <button className="notification-tray-toggle" onClick={() => setOpen(!open)}>
        ▲ Warnings: {warnings.length}
      </button>
      {open && (
        <div className="notification-tray-panel">
          {warnings.length === 0 ? (
            <p className="notification-tray-empty">No warnings.</p>
          ) : (
            <ul>
              {warnings.map((w, i) => (
                <li
                  key={`${w.code}-${w.entity_id ?? i}`}
                  onClick={() => {
                    setActiveScreen(NAVIGATION_TARGET_TO_SCREEN[w.navigation_target]);
                    setOpen(false);
                  }}
                >
                  <span>{SEVERITY_ICON[w.severity]}</span> {w.message}
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
    </div>
  );
}
