/**
 * Left-panel vertical stepper showing wizard progress.
 * Clicking a completed step navigates to it.
 */

import { SCREENS, SCREEN_LABELS, type Screen } from '../types';
import { useProjectStore } from '../store/projectStore';

const STEP_SCREENS: Screen[] = SCREENS.filter((s) => s !== 'welcome');

interface ProgressStepperProps {
  onNavigate: (screen: Screen) => void;
}

export function ProgressStepper({ onNavigate }: ProgressStepperProps) {
  const currentScreen = useProjectStore((s) => s.screen);
  const summary = useProjectStore((s) => s.summary);
  const hasProject = summary !== null;

  const currentIdx = STEP_SCREENS.indexOf(currentScreen as Screen);

  return (
    <nav className="stepper" aria-label="Budget wizard steps">
      <div className="stepper-logo">
        <span className="stepper-logo-icon">🇪🇺</span>
        <span className="stepper-logo-text">M2-EU Budgeter</span>
      </div>

      <ul className="stepper-list">
        {STEP_SCREENS.map((screen, idx) => {
          const isCompleted = idx < currentIdx && hasProject;
          const isCurrent = screen === currentScreen;
          const isAccessible = hasProject || idx === 0;

          return (
            <li key={screen} className="stepper-item">
              <button
                className={`stepper-btn${isCurrent ? ' stepper-btn--active' : ''}${isCompleted ? ' stepper-btn--done' : ''}${!isAccessible ? ' stepper-btn--locked' : ''}`}
                onClick={() => isAccessible && onNavigate(screen)}
                disabled={!isAccessible}
                aria-current={isCurrent ? 'step' : undefined}
              >
                <span className="stepper-num">
                  {isCompleted ? '✓' : idx + 1}
                </span>
                <span className="stepper-label">{SCREEN_LABELS[screen]}</span>
              </button>
            </li>
          );
        })}
      </ul>

      {hasProject && (
        <div className="stepper-version">
          <span>Format 1.0</span>
        </div>
      )}
    </nav>
  );
}
