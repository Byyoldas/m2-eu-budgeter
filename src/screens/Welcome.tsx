/**
 * Welcome screen — shown on first launch.
 * Two actions: New Project, Open Project.
 */

import { open } from '@tauri-apps/plugin-dialog';
import { loadProject } from '../ipc/commands';
import { useProjectStore } from '../store/projectStore';
import type { AppError } from '../types';

interface WelcomeProps {
  onNewProject: () => void;
}

export function Welcome({ onNewProject }: WelcomeProps) {
  const setSummary = useProjectStore((s) => s.setSummary);
  const setProjectPath = useProjectStore((s) => s.setProjectPath);
  const setScreen = useProjectStore((s) => s.setScreen);
  const setGlobalError = useProjectStore((s) => s.setGlobalError);
  const setLoading = useProjectStore((s) => s.setLoading);

  const handleOpen = async () => {
    try {
      const path = await open({
        multiple: false,
        filters: [{ name: 'M2-EU Budgeter File', extensions: ['ercbudget'] }],
      });
      if (!path || typeof path !== 'string') return;

      setLoading(true);
      const summary = await loadProject(path);
      setSummary(summary);
      setProjectPath(path);
      setScreen('review-export');
    } catch (err) {
      setGlobalError(err as AppError);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="welcome-screen">
      <div className="welcome-card">
        <div className="welcome-logo">🇪🇺</div>
        <h1 className="welcome-title">M2-EU Budgeter</h1>
        <p className="welcome-subtitle">
          EU Grant Budget Preparation<br />
          Made simple, for any Horizon Europe action
        </p>

        <div className="welcome-actions">
          <button className="btn btn--primary btn--lg" onClick={onNewProject}>
            ✦ New Project
          </button>
          <button className="btn btn--ghost btn--lg" onClick={handleOpen}>
            📂 Open Project…
          </button>
        </div>

        <p className="welcome-hint">
          .ercbudget files are saved locally on your computer.<br />
          Your data never leaves this device.
        </p>
      </div>
    </div>
  );
}
