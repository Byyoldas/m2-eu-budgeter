/**
 * Vitest global test setup.
 * Mocks the Tauri IPC bridge so store/hooks can be tested in a jsdom
 * environment without a real Tauri backend.
 */

import { vi, beforeAll, afterAll } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn().mockResolvedValue(null),
}));

const originalError = console.error;
beforeAll(() => {
  console.error = (...args: unknown[]) => {
    if (typeof args[0] === 'string' && args[0].includes('Warning:')) return;
    originalError(...args);
  };
});
afterAll(() => {
  console.error = originalError;
});
