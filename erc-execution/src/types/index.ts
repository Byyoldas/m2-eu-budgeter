/**
 * Execution App types. Shared types (produced by ts-rs from erc-core structs)
 * are re-exported from the generated bindings; execution-specific DTOs
 * (defined in erc-execution/src-tauri, not yet ts-rs-generated) are
 * hand-written here, matching src-tauri/src/domain/dto.rs.
 */

export type { AppError } from '../../../erc-core/bindings/AppError';
export type { BudgetSummaryDto } from '../../../erc-core/bindings/BudgetSummaryDto';
export type { RoleType } from '../../../erc-core/bindings/RoleType';

import type { BudgetSummaryDto } from '../../../erc-core/bindings/BudgetSummaryDto';
import type { RoleType } from '../../../erc-core/bindings/RoleType';

export interface ProjectInfoDto {
  project_title: string;
  pi_name: string;
  call_reference: string;
  duration_years: number;
  work_package_count: number;
}

export interface PersonnelRoleSummaryDto {
  id: string;
  role_label: string;
  role_type: RoleType;
}

// ─── M-03: Personnel & Person-Month Tracking ──────────────────────────────────

export interface PersonInputDto {
  full_name: string;
  email: string | null;
  institution: string | null;
  orcid: string | null;
  linked_role_id: string;
  actual_start_date: string;
  actual_end_date: string | null;
}

export interface PersonDetailDto {
  id: string;
  full_name: string;
  email: string | null;
  institution: string | null;
  orcid: string | null;
  linked_role_id: string;
  linked_role_label: string;
  actual_start_date: string;
  actual_end_date: string | null;
}

export interface PersonMonthRecordInputDto {
  person_id: string;
  project_month: number;
  reported_months: string;
  approved_months: string | null;
}

export interface PersonMonthDetailDto {
  id: string;
  person_id: string;
  project_month: number;
  reported_months: string;
  approved_months: string | null;
  salary_cost_estimate_eur: string | null;
}

// ─── M-04: Work Package Management ────────────────────────────────────────────

export type WpStatus = 'NotStarted' | 'OnTrack' | 'AtRisk' | 'Delayed' | 'Completed';

export interface WorkPackageExecutionInputDto {
  leader_role_id: string | null;
  notes: string | null;
}

export interface WorkPackageExecutionDetailDto {
  work_package_id: number;
  work_package_name: string | null;
  leader_role_id: string | null;
  leader_role_label: string | null;
  notes: string | null;
  status: WpStatus;
  planned_eur: string;
  actual_eur: string;
  overspend_warning: boolean;
}

// ─── M-06: Milestone Tracking ──────────────────────────────────────────────────

export type MilestoneStatus = 'NotStarted' | 'OnTrack' | 'AtRisk' | 'Delayed' | 'Completed' | 'Cancelled';

export interface MilestoneInputDto {
  title: string;
  work_package_id: number;
  planned_month: number;
  status: MilestoneStatus;
  actual_completion_month: number | null;
}

export interface MilestoneDetailDto {
  id: string;
  title: string;
  work_package_id: number;
  planned_month: number;
  status: MilestoneStatus;
  effective_status: MilestoneStatus;
  actual_completion_month: number | null;
}

// ─── Amendment Management (from-scratch design) ───────────────────────────────

export type AmendmentType =
  | 'BudgetReallocation'
  | 'DurationExtension'
  | 'WorkPackageScopeChange'
  | 'PersonnelChange'
  | 'Other';

export type AmendmentStatus = 'Requested' | 'Approved' | 'Rejected';

export interface AmendmentInputDto {
  amendment_type: AmendmentType;
  title: string;
  description: string;
  requested_date: string;
  decision_date: string | null;
  status: AmendmentStatus;
  financial_impact_eur: string | null;
  affected_work_package_ids: number[];
  notes: string | null;
}

export interface AmendmentDetailDto {
  id: string;
  amendment_number: string;
  amendment_type: AmendmentType;
  title: string;
  description: string;
  requested_date: string;
  decision_date: string | null;
  status: AmendmentStatus;
  financial_impact_eur: string | null;
  affected_work_package_ids: number[];
  notes: string | null;
}

export interface ExecutionProjectSummaryDto {
  project_info: ProjectInfoDto;
  planned: BudgetSummaryDto;
  current_project_month: number;
  personnel_roles: PersonnelRoleSummaryDto[];
  persons: PersonDetailDto[];
  person_months: PersonMonthDetailDto[];
  work_packages: WorkPackageExecutionDetailDto[];
  milestones: MilestoneDetailDto[];
  amendments: AmendmentDetailDto[];
}

export type ExecutionScreen =
  | 'welcome'
  | 'dashboard'
  | 'personnel'
  | 'work-packages'
  | 'milestones'
  | 'amendments';
