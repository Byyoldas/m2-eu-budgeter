/**
 * Left navigation panel. Sprint E2 added Personnel/Work Packages/Milestones
 * (M-03/M-04/M-06) and Amendments; Sprint E3 added Travel/Equipment/Other
 * Costs/Subcontracting (M-08–M-11); Sprint E4 adds Deliverables (M-05) and
 * Reporting Periods (M-14). The rest wait for Sprint E5 (Risk/Issues).
 */

import { useExecutionStore } from '../store/executionStore';
import type { ExecutionScreen } from '../types';

const MODULES: { label: string; screen: ExecutionScreen | null }[] = [
  { label: 'Dashboard', screen: 'dashboard' },
  { label: 'Work Packages', screen: 'work-packages' },
  { label: 'Deliverables', screen: 'deliverables' },
  { label: 'Personnel', screen: 'personnel' },
  { label: 'Milestones', screen: 'milestones' },
  { label: 'Amendments', screen: 'amendments' },
  { label: 'Travel', screen: 'travel' },
  { label: 'Equipment', screen: 'equipment' },
  { label: 'Other Costs', screen: 'other-costs' },
  { label: 'Subcontracting', screen: 'subcontracting' },
  { label: 'Reporting Periods', screen: 'reporting-periods' },
  { label: 'Risk Register', screen: null },
  { label: 'Issue Log', screen: null },
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
