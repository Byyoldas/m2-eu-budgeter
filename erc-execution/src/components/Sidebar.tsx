/**
 * Left navigation panel. Sprint E2 adds Personnel/Work Packages/Milestones
 * (M-03/M-04/M-06) and Amendments as real screens; the rest wait for their
 * sprint (E3: Financials/Travel/Equipment/Other Costs/Subcontracting, E4:
 * Documents/Reports, E5: Risk/Issues/Periods).
 */

import { useExecutionStore } from '../store/executionStore';
import type { ExecutionScreen } from '../types';

const MODULES: { label: string; screen: ExecutionScreen | null }[] = [
  { label: 'Dashboard', screen: 'dashboard' },
  { label: 'Work Packages', screen: 'work-packages' },
  { label: 'Personnel', screen: 'personnel' },
  { label: 'Milestones', screen: 'milestones' },
  { label: 'Amendments', screen: 'amendments' },
  { label: 'Travel', screen: null },
  { label: 'Equipment', screen: null },
  { label: 'Other Costs', screen: null },
  { label: 'Subcontracting', screen: null },
  { label: 'Risk Register', screen: null },
  { label: 'Issue Log', screen: null },
  { label: 'Reporting Periods', screen: null },
  { label: 'Reports & Export', screen: null },
];

export function Sidebar() {
  const activeScreen = useExecutionStore((s) => s.activeScreen);
  const setActiveScreen = useExecutionStore((s) => s.setActiveScreen);

  return (
    <nav className="sidebar">
      <div className="sidebar-title">ERC Execution</div>
      <ul>
        {MODULES.map((m) => (
          <li
            key={m.label}
            className={m.screen === null ? 'disabled' : m.screen === activeScreen ? 'active' : ''}
            onClick={() => m.screen && setActiveScreen(m.screen)}
          >
            {m.label}
          </li>
        ))}
      </ul>
    </nav>
  );
}
