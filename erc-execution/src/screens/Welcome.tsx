/**
 * Welcome screen — the only entry point into the app (M-02: Budget Open &
 * Project Import). The Execution App never creates a new project; it always
 * opens a `.ercbudget` file already produced by the Budget App.
 */

import { open } from '@tauri-apps/plugin-dialog';
import { useExecutionStore } from '../store/executionStore';
import { openExecutionProject } from '../ipc/commands';
import type { AppError } from '../types';

export function Welcome() {
  const setSummary = useExecutionStore((s) => s.setSummary);
  const setLoading = useExecutionStore((s) => s.setLoading);
  const setError = useExecutionStore((s) => s.setError);
  const error = useExecutionStore((s) => s.error);

  const handleOpen = async () => {
    const path = await open({
      multiple: false,
      filters: [{ name: 'ERC Budget Project', extensions: ['ercbudget'] }],
    });
    if (!path || Array.isArray(path)) return;

    setLoading(true);
    setError(null);
    try {
      const summary = await openExecutionProject(path);
      setSummary(summary, path);
    } catch (e) {
      setError(e as AppError);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="welcome-screen">
      <h1>ERC Execution</h1>
      <p>Track project execution against a budget created in the Budget Application.</p>
      <button onClick={handleOpen}>Open .ercbudget File…</button>
      {error && <p className="error-banner">{JSON.stringify(error)}</p>}
    </div>
  );
}
