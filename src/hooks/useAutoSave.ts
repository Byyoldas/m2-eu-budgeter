/**
 * Auto-save hook.
 *
 * Debounces saving 2 seconds after any mutation (detected via summary change).
 * Auto-save is silent — errors are swallowed (real save-on-close handles critical errors).
 * The Rust backend also auto-saves to a .autosave file after every mutation,
 * so this hook is a belt-and-suspenders secondary mechanism for the named file.
 */

import { useEffect, useRef } from 'react';
import { useProjectStore } from '../store/projectStore';
import { saveProject } from '../ipc/commands';

const DEBOUNCE_MS = 2000;

export function useAutoSave(): void {
  const summary = useProjectStore((s) => s.summary);
  const projectPath = useProjectStore((s) => s.projectPath);
  const setDirty = useProjectStore((s) => s.setDirty);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    // Only auto-save if we have a named file (not new unsaved projects)
    if (!summary || !projectPath) return;

    if (timerRef.current) {
      clearTimeout(timerRef.current);
    }

    timerRef.current = setTimeout(async () => {
      try {
        await saveProject();
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
