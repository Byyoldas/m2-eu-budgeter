/**
 * IPC command wrappers — thin typed wrappers around Tauri's invoke().
 * Every function maps 1:1 to a Rust #[tauri::command].
 * All errors are re-thrown as AppError (already the shape Tauri serialises).
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  ProjectConfigInput,
  PersonnelRoleInput,
  EquipmentItemInput,
  TripInput,
  OtherCostInput,
  BudgetSummaryDto,
  RoleCostPreviewDto,
  EquipmentPreviewDto,
  TripCostPreviewDto,
  RateVersionSummary,
  CountrySummary,
} from '../types';

// ─── Project ──────────────────────────────────────────────────────────────────

export const createProject = (config: ProjectConfigInput): Promise<BudgetSummaryDto> =>
  invoke('create_project', { config });

export const updateProjectConfig = (config: ProjectConfigInput): Promise<BudgetSummaryDto> =>
  invoke('update_project_config', { config });

export const loadProject = (path: string): Promise<BudgetSummaryDto> =>
  invoke('load_project', { path });

export const saveProject = (): Promise<void> =>
  invoke('save_project');

export const saveProjectAs = (path: string): Promise<void> =>
  invoke('save_project_as', { path });

export const getProject = (): Promise<BudgetSummaryDto> =>
  invoke('get_project');

export const getRateVersions = (): Promise<RateVersionSummary[]> =>
  invoke('get_rate_versions');

export const getCountries = (versionId: string): Promise<CountrySummary[]> =>
  invoke('get_countries', { version_id: versionId });

// ─── Personnel ────────────────────────────────────────────────────────────────

export const addPersonnelRole = (input: PersonnelRoleInput): Promise<BudgetSummaryDto> =>
  invoke('add_personnel_role', { input });

export const updatePersonnelRole = (id: string, input: PersonnelRoleInput): Promise<BudgetSummaryDto> =>
  invoke('update_personnel_role', { id, input });

export const deletePersonnelRole = (id: string): Promise<BudgetSummaryDto> =>
  invoke('delete_personnel_role', { id });

export const previewRoleCost = (input: PersonnelRoleInput): Promise<RoleCostPreviewDto> =>
  invoke('preview_role_cost', { input });

// ─── Equipment ────────────────────────────────────────────────────────────────

export const addEquipmentItem = (input: EquipmentItemInput): Promise<BudgetSummaryDto> =>
  invoke('add_equipment_item', { input });

export const updateEquipmentItem = (id: string, input: EquipmentItemInput): Promise<BudgetSummaryDto> =>
  invoke('update_equipment_item', { id, input });

export const deleteEquipmentItem = (id: string): Promise<BudgetSummaryDto> =>
  invoke('delete_equipment_item', { id });

export const previewEquipmentDepreciation = (input: EquipmentItemInput): Promise<EquipmentPreviewDto> =>
  invoke('preview_equipment_depreciation', { input });

// ─── Travel ───────────────────────────────────────────────────────────────────

export const addTrip = (input: TripInput): Promise<BudgetSummaryDto> =>
  invoke('add_trip', { input });

export const updateTrip = (id: string, input: TripInput): Promise<BudgetSummaryDto> =>
  invoke('update_trip', { id, input });

export const deleteTrip = (id: string): Promise<BudgetSummaryDto> =>
  invoke('delete_trip', { id });

export const previewTripCost = (input: TripInput): Promise<TripCostPreviewDto> =>
  invoke('preview_trip_cost', { input });

// ─── Other Costs & CFS ───────────────────────────────────────────────────────

export const addOtherCost = (input: OtherCostInput): Promise<BudgetSummaryDto> =>
  invoke('add_other_cost', { input });

export const updateOtherCost = (id: string, input: OtherCostInput): Promise<BudgetSummaryDto> =>
  invoke('update_other_cost', { id, input });

export const deleteOtherCost = (id: string): Promise<BudgetSummaryDto> =>
  invoke('delete_other_cost', { id });

export const addCfsItem = (amountEur: string, workPackageIds: number[]): Promise<BudgetSummaryDto> =>
  invoke('add_cfs_item', { amount_eur: amountEur, work_package_ids: workPackageIds });

export const removeCfsItem = (): Promise<BudgetSummaryDto> =>
  invoke('remove_cfs_item');

export const dismissCfsWarning = (): Promise<BudgetSummaryDto> =>
  invoke('dismiss_cfs_warning');

export const setSubcontracting = (amountEur: string, workPackageId: number): Promise<BudgetSummaryDto> =>
  invoke('set_subcontracting', { amount_eur: amountEur, work_package_id: workPackageId });
