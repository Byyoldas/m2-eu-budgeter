/**
 * Warning banner — shown at the top of the left panel for critical alerts.
 */

interface WarningBannerProps {
  message: string;
  severity?: 'warning' | 'error' | 'info';
  onDismiss?: () => void;
  action?: { label: string; onClick: () => void };
}

export function WarningBanner({ message, severity = 'warning', onDismiss, action }: WarningBannerProps) {
  return (
    <div className={`warning-banner warning-banner--${severity}`} role="alert">
      <span className="warning-banner-icon">
        {severity === 'error' ? '✕' : severity === 'info' ? 'ℹ' : '⚠'}
      </span>
      <span className="warning-banner-message">{message}</span>
      <div className="warning-banner-actions">
        {action && (
          <button className="warning-banner-action-btn" onClick={action.onClick}>
            {action.label}
          </button>
        )}
        {onDismiss && (
          <button className="warning-banner-dismiss" onClick={onDismiss} aria-label="Dismiss">
            ×
          </button>
        )}
      </div>
    </div>
  );
}
