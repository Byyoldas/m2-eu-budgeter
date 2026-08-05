/**
 * Manual save control. The app already auto-saves 2s after every mutation
 * (`useAutoSave`), but that hook is silent by design — errors are swallowed
 * so a transient failure doesn't spam the UI. This button gives the user an
 * explicit way to save on demand and, unlike auto-save, surfaces a failure
 * instead of hiding it.
 */

import { useState } from 'react';
import { useExecutionStore } from '../store/executionStore';
import { saveExecutionProject } from '../ipc/commands';
import { appErrorMessage } from '../utils/errors';
import type { AppError } from '../types';

export function SaveButton() {
  const isDirty = useExecutionStore((s) => s.isDirty);
  const setDirty = useExecutionStore((s) => s.setDirty);
  const projectPath = useExecutionStore((s) => s.projectPath);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!projectPath) return null;

  const save = async () => {
    setIsSaving(true);
    setError(null);
    try {
      await saveExecutionProject();
      setDirty(false);
    } catch (e) {
      setError(appErrorMessage(e as AppError));
    } finally {
      setIsSaving(false);
    }
  };

  const status = isSaving ? 'Saving…' : error ? error : isDirty ? 'Unsaved changes' : 'All changes saved';

  return (
    <div className="save-control">
      <button onClick={save} disabled={isSaving || !isDirty}>
        Save
      </button>
      <p className={error ? 'save-status save-status-error' : 'save-status'}>{status}</p>
    </div>
  );
}
