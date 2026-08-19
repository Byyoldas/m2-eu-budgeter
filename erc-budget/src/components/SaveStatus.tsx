/**
 * Sidebar save control: a manual Save button plus a status line
 * ("All changes saved" / "Unsaved changes" / "Saving…" / the real error).
 *
 * The actual save logic (debounced auto-save + this button's immediate
 * save) lives in useAutoSave — this component only reflects its state.
 */

import { useProjectStore } from '../store/projectStore';

interface SaveStatusProps {
  onSaveNow: () => void;
}

export function SaveStatus({ onSaveNow }: SaveStatusProps) {
  const isDirty = useProjectStore((s) => s.isDirty);
  const saveStatus = useProjectStore((s) => s.saveStatus);
  const saveError = useProjectStore((s) => s.saveError);
  const projectPath = useProjectStore((s) => s.projectPath);

  const label =
    saveStatus === 'saving'
      ? 'Saving…'
      : saveStatus === 'error'
        ? (saveError ?? 'Failed to save.')
        : isDirty
          ? 'Unsaved changes'
          : 'All changes saved';

  return (
    <div className="save-status">
      <button
        className="btn btn--ghost btn--sm save-status-btn"
        onClick={onSaveNow}
        disabled={!isDirty || saveStatus === 'saving'}
      >
        💾 Save
      </button>
      <span className={`save-status-label${saveStatus === 'error' ? ' save-status-label--error' : ''}`}>
        {label}
      </span>
      {projectPath && (
        <span className="save-status-path" title={projectPath}>
          {projectPath}
        </span>
      )}
    </div>
  );
}
