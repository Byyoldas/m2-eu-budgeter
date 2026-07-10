/**
 * Global Zustand store.
 *
 * Single source of truth for:
 * - Current screen / wizard progress
 * - BudgetSummaryDto (updated after every mutation)
 * - Project configuration (mirrored for form pre-population)
 * - Loading / error state
 * - Rate data (loaded once on app start)
 */

import { create } from 'zustand';
import type {
  BudgetSummaryDto,
  Screen,
  ProjectConfigInput,
  AppError,
  RateVersionSummary,
  CountrySummary,
} from '../types';

interface ProjectStore {
  // Navigation
  screen: Screen;
  setScreen: (screen: Screen) => void;

  // Budget summary (mirrors CALC-19 output — updated after every mutation)
  summary: BudgetSummaryDto | null;
  setSummary: (summary: BudgetSummaryDto) => void;

  // Project configuration (for form pre-population)
  projectConfig: ProjectConfigInput | null;
  setProjectConfig: (config: ProjectConfigInput) => void;

  // Project file path (for save vs. save-as decisions)
  projectPath: string | null;
  setProjectPath: (path: string | null) => void;

  // Rate data (loaded once at app start)
  rateVersions: RateVersionSummary[];
  setRateVersions: (versions: RateVersionSummary[]) => void;
  countries: CountrySummary[];
  setCountries: (countries: CountrySummary[]) => void;

  // Global loading / error state
  isLoading: boolean;
  setLoading: (loading: boolean) => void;
  globalError: AppError | null;
  setGlobalError: (error: AppError | null) => void;

  // Dirty flag (unsaved changes)
  isDirty: boolean;
  setDirty: (dirty: boolean) => void;

  // Reset to initial state (e.g. when creating a new project)
  reset: () => void;
}

const initialState = {
  screen: 'welcome' as Screen,
  summary: null,
  projectConfig: null,
  projectPath: null,
  rateVersions: [],
  countries: [],
  isLoading: false,
  globalError: null,
  isDirty: false,
};

export const useProjectStore = create<ProjectStore>((set) => ({
  ...initialState,

  setScreen: (screen) => set({ screen }),
  setSummary: (summary) => set({ summary, isDirty: true }),
  setProjectConfig: (projectConfig) => set({ projectConfig }),
  setProjectPath: (projectPath) => set({ projectPath }),
  setRateVersions: (rateVersions) => set({ rateVersions }),
  setCountries: (countries) => set({ countries }),
  setLoading: (isLoading) => set({ isLoading }),
  setGlobalError: (globalError) => set({ globalError }),
  setDirty: (isDirty) => set({ isDirty }),
  reset: () => set(initialState),
}));

// ─── Selectors (derived) ──────────────────────────────────────────────────────

/** Returns true once a project is open (summary exists). */
export const useHasProject = () => useProjectStore((s) => s.summary !== null);

/** Returns the current screen name. */
export const useScreen = () => useProjectStore((s) => s.screen);

/** Returns the budget summary or null. */
export const useSummary = () => useProjectStore((s) => s.summary);

/** Returns the project configuration or null. */
export const useProjectConfig = () => useProjectStore((s) => s.projectConfig);

/** Returns the list of personnel roles from the current summary. */
export const usePersonnelRoles = () =>
  useProjectStore((s) => s.summary?.role_detail ?? []);

/** Returns the list of equipment items from the current summary. */
export const useEquipmentItems = () =>
  useProjectStore((s) => s.summary?.equipment_detail ?? []);

/** Returns the list of trips from the current summary. */
export const useTrips = () =>
  useProjectStore((s) => s.summary?.trip_detail ?? []);

/** Returns the CFS status from the summary. */
export const useCfsStatus = () =>
  useProjectStore((s) => s.summary?.cfs_status ?? 'NOT_REQUIRED');
