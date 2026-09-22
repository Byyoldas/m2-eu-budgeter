/**
 * Verifies the "Update Available" modal actually surfaces the release notes
 * a new version ships with — the full path from the Tauri updater plugin's
 * `check()` result, through `updaterStore`, into what `UpdateChecker`
 * renders. The notes text here is exactly what's published on the live
 * v1.11.0 GitHub release's `latest.json`, so this proves real users get a
 * readable, line-separated changelog in the prompt rather than the notes
 * being silently dropped somewhere in that chain.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { UpdateChecker } from './UpdateChecker';
import { useUpdaterStore } from '../store/updaterStore';

const v1_11_0_NOTES =
  "Person-Months (PM) rounding now drives the budget calculation, not just the Excel display: PM is rounded to 1 decimal (standard round-half-up, e.g. 2.37 -> 2.4) before being priced, matching the EU Funding & Tenders Portal convention.\n" +
  "Excel export's per-Work-Package cost cells and row Totals are live formulas again, not static values — editing salary/FTE/dates in the sheet and recalculating reproduces the same numbers the app computed.";

const mockCheck = vi.fn();

vi.mock('@tauri-apps/plugin-updater', () => ({
  check: () => mockCheck(),
}));
vi.mock('@tauri-apps/api/app', () => ({
  getVersion: vi.fn().mockResolvedValue('1.10.0'),
}));
vi.mock('@tauri-apps/plugin-process', () => ({
  relaunch: vi.fn(),
}));

describe('UpdateChecker', () => {
  beforeEach(() => {
    mockCheck.mockReset();
    // Reset the shared zustand store between tests so one test's "update
    // found" state doesn't leak into the next.
    useUpdaterStore.setState({ update: null, result: 'idle', currentVersion: null });
  });

  it('renders the release notes from a found update, one line per paragraph', async () => {
    mockCheck.mockResolvedValue({
      version: '1.11.0',
      currentVersion: '1.10.0',
      body: v1_11_0_NOTES,
      downloadAndInstall: vi.fn(),
    });

    render(<UpdateChecker />);

    expect(await screen.findByText('Update Available — v1.11.0')).toBeTruthy();
    expect(screen.getByText('A new version of M2-EU Budgeter is available (you have v1.10.0).')).toBeTruthy();

    // Each \n-separated line of `update.body` must render as its own
    // paragraph inside the scrollable .update-notes box, not run together.
    const notesBox = document.querySelector('.update-notes');
    expect(notesBox).not.toBeNull();
    const paragraphs = notesBox!.querySelectorAll('p');
    expect(paragraphs).toHaveLength(2);
    expect(paragraphs[0].textContent).toContain('Person-Months (PM) rounding now drives the budget calculation');
    expect(paragraphs[1].textContent).toContain("Excel export's per-Work-Package cost cells and row Totals are live formulas again");

    // Sanity: the full published text is present somewhere in the DOM.
    for (const line of v1_11_0_NOTES.split('\n')) {
      expect(screen.getByText(line)).toBeTruthy();
    }
  });

  it('renders nothing while no update has been found yet (still checking / up to date)', async () => {
    mockCheck.mockResolvedValue(null);

    const { container } = render(<UpdateChecker />);

    await waitFor(() => expect(mockCheck).toHaveBeenCalled());
    expect(container.firstChild).toBeNull();
  });
});
