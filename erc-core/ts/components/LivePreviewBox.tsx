/**
 * Generic live preview box — shown alongside forms to display real-time calculation results.
 */

interface PreviewRow {
  label: string;
  value: string;
  highlight?: boolean;
}

interface LivePreviewBoxProps {
  title: string;
  rows: PreviewRow[];
  isLoading?: boolean;
  error?: string | null;
  emptyMessage?: string;
}

export function LivePreviewBox({ title, rows, isLoading, error, emptyMessage }: LivePreviewBoxProps) {
  return (
    <div className="live-preview">
      <div className="live-preview-header">
        <span className="live-preview-title">{title}</span>
        {isLoading && <span className="live-preview-spinner" aria-label="Calculating…" />}
      </div>

      {error ? (
        <div className="live-preview-error">{error}</div>
      ) : rows.length === 0 ? (
        <div className="live-preview-empty">{emptyMessage ?? 'Fill in the form to see a preview.'}</div>
      ) : (
        <div className="live-preview-rows">
          {rows.map(({ label, value, highlight }) => (
            <div key={label} className={`live-preview-row${highlight ? ' live-preview-row--highlight' : ''}`}>
              <span className="live-preview-row-label">{label}</span>
              <span className="live-preview-row-value">{value}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
