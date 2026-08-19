/**
 * Auto-save hook.
 *
 * Debounces saving 2 seconds after any mutation (detected via summary change).
 * Also exposes `saveNow` for an immediate, manual save (e.g. the sidebar Save
 * button), which cancels any pending debounce so the two never race.
 *
 * Every project has a file path from the moment it's created (create_project
 * defaults it to the Desktop), so auto-save is active for the whole life of
 * a project, not just after an explicit first save.
 *
 * The Rust backend also auto-saves to a `.autosave` file after every mutation
 * as a crash-recovery safety net — this hook is what keeps the *named* file
 * itself up to date, and is what the sidebar's save status reflects.
 */

import { useEffect, useRef, useCallback } from 'react';
import { useProjectStore } from '../store/projectStore';
import { saveProject } from '../ipc/commands';
import { formatAppError } from '../utils/formatAppError';
import type { AppError } from '../types';

const DEBOUNCE_MS = 2000;

export function useAutoSave(): { saveNow: () => Promise<void> } {
  const summary = useProjectStore((s) => s.summary);
  const projectPath = useProjectStore((s) => s.projectPath);
  const setDirty = useProjectStore((s) => s.setDirty);
  const setSaveStatus = useProjectStore((s) => s.setSaveStatus);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const runSave = useCallback(async () => {
    setSaveStatus('saving');
    try {
      await saveProject();
      setDirty(false);
      setSaveStatus('idle');
    } catch (err) {
      setSaveStatus('error', formatAppError(err as AppError));
    }
  }, [setDirty, setSaveStatus]);

  useEffect(() => {
    // Only auto-save if we have a named file (not new unsaved projects)
    if (!summary || !projectPath) return;

    if (timerRef.current) {
      clearTimeout(timerRef.current);
    }

    timerRef.current = setTimeout(runSave, DEBOUNCE_MS);

    return () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
    };
  }, [summary, projectPath, runSave]);

  const saveNow = useCallback(async () => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }
    await runSave();
  }, [runSave]);

  return { saveNow };
}
