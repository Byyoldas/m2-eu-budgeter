/**
 * Vitest configuration for the ERC Budget frontend.
 *
 * Tests are co-located with source files (*.test.ts) or placed in src/__tests__/.
 * The Tauri IPC is mocked globally via the setup file.
 */

import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/__tests__/setup.ts'],
    include: ['src/**/*.test.{ts,tsx}', 'src/__tests__/**/*.test.{ts,tsx}'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      include: [
        'src/validators/**',
        'src/store/**',
        'src/hooks/**',
        'src/ipc/**',
        'src/export/**',
      ],
      exclude: [
        'src/__tests__/**',
        'src/main.tsx',
      ],
      thresholds: {
        lines: 80,
        branches: 75,
        functions: 80,
        statements: 80,
      },
    },
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
});
