/**
 * Background update check + "update available" prompt.
 *
 * On mount, silently asks the Tauri updater plugin whether a newer signed
 * release exists (per tauri.conf.json's `plugins.updater.endpoints`, which
 * points at the GitHub Releases `latest.json` asset). If one is found, shows
 * a dismissible modal offering to download and install it in place; the app
 * relaunches itself once the install finishes. Network/parse failures during
 * the background check are swallowed — this must never block or interrupt
 * normal use of the app.
 */

import { useEffect, useState } from 'react';
import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

type Phase = 'idle' | 'downloading' | 'installing' | 'error';

export function UpdateChecker() {
  const [update, setUpdate] = useState<Update | null>(null);
  const [dismissed, setDismissed] = useState(false);
  const [phase, setPhase] = useState<Phase>('idle');
  const [progress, setProgress] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    check()
      .then((result) => {
        if (result) setUpdate(result);
      })
      .catch(() => {
        // No network, no releases yet, endpoint unreachable, etc. — this is
        // a background check, so failures are silent rather than shown.
      });
  }, []);

  if (!update || dismissed) return null;

  const handleUpdate = async () => {
    setPhase('downloading');
    setError(null);
    let totalBytes = 0;
    let downloadedBytes = 0;
    try {
      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case 'Started':
            totalBytes = event.data.contentLength ?? 0;
            downloadedBytes = 0;
            setProgress(totalBytes > 0 ? 0 : null);
            break;
          case 'Progress':
            downloadedBytes += event.data.chunkLength;
            setProgress(totalBytes > 0 ? Math.min(99, Math.round((downloadedBytes / totalBytes) * 100)) : null);
            break;
          case 'Finished':
            setPhase('installing');
            setProgress(100);
            break;
        }
      });
      await relaunch();
    } catch {
      setPhase('error');
      setError('The update could not be downloaded or installed. Please check your internet connection and try again, or download it manually from the GitHub releases page.');
    }
  };

  return (
    <div className="modal-overlay" role="dialog" aria-modal="true">
      <div className="modal-box">
        <div className="modal-header">
          <span className="modal-icon">⬆</span>
          <h2 className="modal-title">Update Available — v{update.version}</h2>
        </div>

        <div className="modal-body">
          <p>
            A new version of M2-EU Budgeter is available (you have v{update.currentVersion}).
          </p>
          {update.body && (
            <div className="update-notes">
              {update.body.split('\n').map((line, i) => <p key={i}>{line}</p>)}
            </div>
          )}

          {phase === 'downloading' && (
            <p className="form-hint">
              Downloading{progress !== null ? `… ${progress}%` : '…'}
            </p>
          )}
          {phase === 'installing' && <p className="form-hint">Installing… the app will restart automatically.</p>}
          {phase === 'error' && error && <div className="error-banner">{error}</div>}
        </div>

        <div className="modal-footer">
          <button
            className="btn btn--ghost"
            onClick={() => setDismissed(true)}
            disabled={phase === 'downloading' || phase === 'installing'}
          >
            Later
          </button>
          <button
            className="btn btn--primary"
            onClick={handleUpdate}
            disabled={phase === 'downloading' || phase === 'installing'}
          >
            {phase === 'downloading' || phase === 'installing' ? 'Please wait…' : 'Update Now'}
          </button>
        </div>
      </div>
    </div>
  );
}
