/**
 * IPC command wrappers — thin typed wrappers around Tauri's invoke().
 * Every function maps 1:1 to a Rust #[tauri::command] in
 * erc-execution/src-tauri/src/commands/*.rs.
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  AmendmentInputDto,
  ExecutionProjectSummaryDto,
  MilestoneInputDto,
  PersonInputDto,
  PersonMonthRecordInputDto,
  WorkPackageExecutionInputDto,
} from '../types';

// ─── Project ──────────────────────────────────────────────────────────────────

export const openExecutionProject = (path: string): Promise<ExecutionProjectSummaryDto> =>
  invoke('open_execution_project', { path });

export const saveExecutionProject = (): Promise<void> =>
  invoke('save_execution_project');

// ─── M-03: Personnel & Person-Month Tracking ──────────────────────────────────

export const addPerson = (input: PersonInputDto): Promise<ExecutionProjectSummaryDto> =>
  invoke('add_person', { input });

export const updatePerson = (id: string, input: PersonInputDto): Promise<ExecutionProjectSummaryDto> =>
  invoke('update_person', { id, input });

export const deletePerson = (id: string): Promise<ExecutionProjectSummaryDto> =>
  invoke('delete_person', { id });

export const addPersonMonthRecord = (
  input: PersonMonthRecordInputDto,
): Promise<ExecutionProjectSummaryDto> => invoke('add_person_month_record', { input });

export const updatePersonMonthRecord = (
  id: string,
  input: PersonMonthRecordInputDto,
): Promise<ExecutionProjectSummaryDto> => invoke('update_person_month_record', { id, input });

export const deletePersonMonthRecord = (id: string): Promise<ExecutionProjectSummaryDto> =>
  invoke('delete_person_month_record', { id });

// ─── M-04: Work Package Management ────────────────────────────────────────────

export const setWorkPackageExecution = (
  workPackageId: number,
  input: WorkPackageExecutionInputDto,
): Promise<ExecutionProjectSummaryDto> =>
  invoke('set_work_package_execution', { work_package_id: workPackageId, input });

// ─── M-06: Milestone Tracking ──────────────────────────────────────────────────

export const addMilestone = (input: MilestoneInputDto): Promise<ExecutionProjectSummaryDto> =>
  invoke('add_milestone', { input });

export const updateMilestone = (
  id: string,
  input: MilestoneInputDto,
): Promise<ExecutionProjectSummaryDto> => invoke('update_milestone', { id, input });

export const completeMilestone = (id: string): Promise<ExecutionProjectSummaryDto> =>
  invoke('complete_milestone', { id });

export const deleteMilestone = (id: string): Promise<ExecutionProjectSummaryDto> =>
  invoke('delete_milestone', { id });

// ─── Amendment Management (from-scratch design) ───────────────────────────────

export const recordAmendment = (input: AmendmentInputDto): Promise<ExecutionProjectSummaryDto> =>
  invoke('record_amendment', { input });

export const updateAmendment = (
  id: string,
  input: AmendmentInputDto,
): Promise<ExecutionProjectSummaryDto> => invoke('update_amendment', { id, input });

export const deleteAmendment = (id: string): Promise<ExecutionProjectSummaryDto> =>
  invoke('delete_amendment', { id });
