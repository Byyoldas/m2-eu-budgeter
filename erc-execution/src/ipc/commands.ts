/**
 * IPC command wrappers — thin typed wrappers around Tauri's invoke().
 * Every function maps 1:1 to a Rust #[tauri::command] in
 * erc-execution/src-tauri/src/commands/*.rs.
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  ActualCostEntryInputDto,
  AmendmentInputDto,
  DeliverableInputDto,
  EquipmentProcurementInputDto,
  ExecutionProjectSummaryDto,
  MilestoneInputDto,
  PersonInputDto,
  PersonMonthRecordInputDto,
  ReportingPeriodInputDto,
  SubcontractingLineInputDto,
  TripExecutionInputDto,
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

// ─── M-08: Travel Tracking ──────────────────────────────────────────────────────

export const addTripExecution = (
  input: TripExecutionInputDto,
): Promise<ExecutionProjectSummaryDto> => invoke('add_trip_execution', { input });

export const updateTripExecution = (
  id: string,
  input: TripExecutionInputDto,
): Promise<ExecutionProjectSummaryDto> => invoke('update_trip_execution', { id, input });

export const deleteTripExecution = (id: string): Promise<ExecutionProjectSummaryDto> =>
  invoke('delete_trip_execution', { id });

// ─── M-09: Equipment Tracking ───────────────────────────────────────────────────

export const addEquipmentProcurement = (
  input: EquipmentProcurementInputDto,
): Promise<ExecutionProjectSummaryDto> => invoke('add_equipment_procurement', { input });

export const updateEquipmentProcurement = (
  id: string,
  input: EquipmentProcurementInputDto,
): Promise<ExecutionProjectSummaryDto> => invoke('update_equipment_procurement', { id, input });

export const deleteEquipmentProcurement = (id: string): Promise<ExecutionProjectSummaryDto> =>
  invoke('delete_equipment_procurement', { id });

// ─── M-10: Other Costs Tracking ─────────────────────────────────────────────────

export const addActualCostEntry = (
  input: ActualCostEntryInputDto,
): Promise<ExecutionProjectSummaryDto> => invoke('add_actual_cost_entry', { input });

export const updateActualCostEntry = (
  id: string,
  input: ActualCostEntryInputDto,
): Promise<ExecutionProjectSummaryDto> => invoke('update_actual_cost_entry', { id, input });

export const deleteActualCostEntry = (id: string): Promise<ExecutionProjectSummaryDto> =>
  invoke('delete_actual_cost_entry', { id });

// ─── M-11: Subcontracting Tracking ──────────────────────────────────────────────

export const addSubcontractingLine = (
  input: SubcontractingLineInputDto,
): Promise<ExecutionProjectSummaryDto> => invoke('add_subcontracting_line', { input });

export const updateSubcontractingLine = (
  id: string,
  input: SubcontractingLineInputDto,
): Promise<ExecutionProjectSummaryDto> => invoke('update_subcontracting_line', { id, input });

export const deleteSubcontractingLine = (id: string): Promise<ExecutionProjectSummaryDto> =>
  invoke('delete_subcontracting_line', { id });

// ─── M-05: Deliverable Tracking ──────────────────────────────────────────────────

export const addDeliverable = (
  input: DeliverableInputDto,
): Promise<ExecutionProjectSummaryDto> => invoke('add_deliverable', { input });

export const updateDeliverable = (
  id: string,
  input: DeliverableInputDto,
): Promise<ExecutionProjectSummaryDto> => invoke('update_deliverable', { id, input });

export const deleteDeliverable = (id: string): Promise<ExecutionProjectSummaryDto> =>
  invoke('delete_deliverable', { id });

// ─── M-14: Reporting Period Management ───────────────────────────────────────────

export const addReportingPeriod = (
  input: ReportingPeriodInputDto,
): Promise<ExecutionProjectSummaryDto> => invoke('add_reporting_period', { input });

export const updateReportingPeriod = (
  id: string,
  input: ReportingPeriodInputDto,
): Promise<ExecutionProjectSummaryDto> => invoke('update_reporting_period', { id, input });

export const deleteReportingPeriod = (id: string): Promise<ExecutionProjectSummaryDto> =>
  invoke('delete_reporting_period', { id });
