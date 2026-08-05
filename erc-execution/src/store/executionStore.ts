/**
 * projectStore — project identity, planned budget, and load/error state.
 * Per docs/executer/execution-architecture.md §5.2, this is split from a
 * future executionStore (live execution state) and uiStore (UI-only state)
 * once those exist (Sprint E2+); Sprint E1 only needs this one.
 */

import { create } from 'zustand';
import type { AppError, ExecutionProjectSummaryDto, ExecutionScreen } from '../types';

interface ExecutionState {
  summary: ExecutionProjectSummaryDto | null;
  projectPath: string | null;
  activeScreen: ExecutionScreen;
  isLoading: boolean;
  isDirty: boolean;
  error: AppError | null;
  setSummary: (summary: ExecutionProjectSummaryDto, path: string) => void;
  /** Applies a fresh summary after a mutation, without navigating away from
   * the current screen (unlike `setSummary`, which is for opening a file). */
  updateSummary: (summary: ExecutionProjectSummaryDto) => void;
  setLoading: (isLoading: boolean) => void;
  setError: (error: AppError | null) => void;
  setActiveScreen: (screen: ExecutionScreen) => void;
  setDirty: (isDirty: boolean) => void;
}

export const useExecutionStore = create<ExecutionState>((set) => ({
  summary: null,
  projectPath: null,
  activeScreen: 'welcome',
  isLoading: false,
  isDirty: false,
  error: null,
  setSummary: (summary, projectPath) =>
    set({ summary, projectPath, activeScreen: 'dashboard', error: null }),
  updateSummary: (summary) => set({ summary, error: null, isDirty: true }),
  setLoading: (isLoading) => set({ isLoading }),
  setError: (error) => set({ error }),
  setActiveScreen: (activeScreen) => set({ activeScreen }),
  setDirty: (isDirty) => set({ isDirty }),
}));
