/**
 * Auto-save hook.
 *
 * Debounces saving 2 seconds after any mutation (detected via summary change).
 * Auto-save is silent — errors are swallowed (real save-on-close handles critical errors).
 * The Rust backend also auto-saves to a .autosave file after every mutation,
 * so this hook is a belt-and-suspenders secondary mechanism for the named file.
 */

import { useEffect, useRef } from 'react';
import { useExecutionStore } from '../store/executionStore';
import { saveExecutionProject } from '../ipc/commands';

const DEBOUNCE_MS = 2000;

export function useAutoSave(): void {
  const summary = useExecutionStore((s) => s.summary);
  const projectPath = useExecutionStore((s) => s.projectPath);
  const setDirty = useExecutionStore((s) => s.setDirty);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    // Only auto-save if a file is open.
    if (!summary || !projectPath) return;

    if (timerRef.current) {
      clearTimeout(timerRef.current);
    }

    timerRef.current = setTimeout(async () => {
      try {
        await saveExecutionProject();
        setDirty(false);
      } catch {
        // Silent fail — auto-save is best-effort
      }
    }, DEBOUNCE_MS);

    return () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
    };
  }, [summary, projectPath, setDirty]);
}
