/**
 * Small shared store for the Tauri updater check.
 *
 * Both the always-mounted `UpdateChecker` (auto-checks silently on launch)
 * and the Welcome screen's manual "Check for Updates" button read/write this
 * same state, so a manual check reuses the exact same "Update Available"
 * modal instead of duplicating that UI — and the Welcome screen can show
 * explicit "up to date" / "error" feedback that the silent auto-check
 * deliberately doesn't.
 */

import { create } from 'zustand';
import { check, type Update } from '@tauri-apps/plugin-updater';
import { getVersion } from '@tauri-apps/api/app';

export type CheckResult = 'idle' | 'checking' | 'up-to-date' | 'available' | 'error';

interface UpdaterStore {
  update: Update | null;
  result: CheckResult;
  currentVersion: string | null;
  checkForUpdates: () => Promise<void>;
  clearUpdate: () => void;
}

export const useUpdaterStore = create<UpdaterStore>((set) => ({
  update: null,
  result: 'idle',
  currentVersion: null,

  checkForUpdates: async () => {
    set({ result: 'checking' });
    try {
      const [found, currentVersion] = await Promise.all([check(), getVersion()]);
      if (found) {
        set({ update: found, result: 'available', currentVersion });
      } else {
        set({ result: 'up-to-date', currentVersion });
      }
    } catch {
      set({ result: 'error' });
    }
  },

  clearUpdate: () => set({ update: null, result: 'idle' }),
}));
