/**
 * Root application component.
 *
 * Layout: fixed left sidebar (stepper + totals panel) + scrollable right content area.
 * Navigation is driven by Zustand screen state; no URL routing needed.
 */

import { useEffect } from 'react';
import { useProjectStore } from './store/projectStore';
import { getRateVersions } from './ipc/commands';
import { useAutoSave } from './hooks/useAutoSave';

import { ProgressStepper } from './components/ProgressStepper';
import { CategoryTotalsPanel } from './components/CategoryTotalsPanel';
import { BudgetWpBarChart } from './components/BudgetWpBarChart';
import { BudgetRingChart } from './components/BudgetRingChart';

import { Welcome } from './screens/Welcome';
import { ProjectSetup } from './screens/ProjectSetup';
import { BudgetSettings } from './screens/BudgetSettings';
import { WorkPackages } from './screens/WorkPackages';
import { Personnel } from './screens/Personnel';
import { Equipment } from './screens/Equipment';
import { Travel } from './screens/Travel';
import { OtherCosts } from './screens/OtherCosts';
import { ReviewExport } from './screens/ReviewExport';

import type { Screen } from './types';
import './App.css';

const STEP_ORDER: Screen[] = [
  'project-setup',
  'budget-settings',
  'personnel',
  'equipment',
  'travel',
  'other-costs',
  'work-packages',
  'review-export',
];

export function App() {
  const screen = useProjectStore((s) => s.screen);
  const setScreen = useProjectStore((s) => s.setScreen);
  const setRateVersions = useProjectStore((s) => s.setRateVersions);
  const summary = useProjectStore((s) => s.summary);

  // Activate auto-save
  useAutoSave();

  // Load rate versions on startup (they're lightweight)
  useEffect(() => {
    getRateVersions().then(setRateVersions).catch(() => {});
  }, []);

  const goTo = (s: Screen) => setScreen(s);
  const goNext = () => {
    const idx = STEP_ORDER.indexOf(screen as Screen);
    if (idx >= 0 && idx < STEP_ORDER.length - 1) setScreen(STEP_ORDER[idx + 1]);
  };
  const goBack = () => {
    const idx = STEP_ORDER.indexOf(screen as Screen);
    if (idx > 0) setScreen(STEP_ORDER[idx - 1]);
  };

  // ── Welcome screen (full-screen, no sidebar) ─────────────────────────────
  if (screen === 'welcome') {
    return (
      <div className="app-root app-root--welcome">
        <Welcome onNewProject={() => setScreen('project-setup')} />
      </div>
    );
  }

  // ── Main wizard layout ────────────────────────────────────────────────────
  return (
    <div className="app-root">
      {/* Left sidebar: stepper + live totals */}
      <aside className="app-sidebar">
        <ProgressStepper onNavigate={goTo} />
        <div className="sidebar-dashboard">
          <CategoryTotalsPanel />
          {summary && (
            <>
              <BudgetWpBarChart />
              <BudgetRingChart />
            </>
          )}
        </div>
      </aside>

      {/* Right content area */}
      <main className="app-main">
        {screen === 'project-setup' && (
          <ProjectSetup onNext={goNext} />
        )}
        {screen === 'budget-settings' && (
          <BudgetSettings onNext={goNext} onBack={goBack} />
        )}
        {screen === 'work-packages' && (
          <WorkPackages onNext={goNext} onBack={goBack} />
        )}
        {screen === 'personnel' && (
          <Personnel onNext={goNext} onBack={goBack} />
        )}
        {screen === 'equipment' && (
          <Equipment onNext={goNext} onBack={goBack} />
        )}
        {screen === 'travel' && (
          <Travel onNext={goNext} onBack={goBack} />
        )}
        {screen === 'other-costs' && (
          <OtherCosts onNext={goNext} onBack={goBack} />
        )}
        {screen === 'review-export' && (
          <ReviewExport onBack={goBack} />
        )}
      </main>
    </div>
  );
}
