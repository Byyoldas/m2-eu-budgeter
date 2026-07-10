/**
 * Vitest global test setup.
 * Mocks the Tauri IPC bridge so validators, store, and hooks
 * can be tested in a jsdom environment without a real Tauri backend.
 */

import { vi, beforeAll, afterAll } from 'vitest';

// Mock the Tauri invoke function — individual tests override this as needed.
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

// Silence console.error for expected validation messages in tests.
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
