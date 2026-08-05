/**
 * Execution App types. Shared types (produced by ts-rs from erc-core structs)
 * are re-exported from the generated bindings; execution-specific DTOs
 * (defined in erc-execution/src-tauri, not yet ts-rs-generated) are
 * hand-written here, matching src-tauri/src/domain/dto.rs.
 */

export type { AppError } from '../../../erc-core/bindings/AppError';
export type { BudgetSummaryDto } from '../../../erc-core/bindings/BudgetSummaryDto';
export type { RoleType } from '../../../erc-core/bindings/RoleType';
export type { CfsStatus } from '../../../erc-core/bindings/CfsStatus';

import type { BudgetSummaryDto } from '../../../erc-core/bindings/BudgetSummaryDto';
import type { RoleType } from '../../../erc-core/bindings/RoleType';
import type { CfsStatus } from '../../../erc-core/bindings/CfsStatus';

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

export interface PlannedTripSummaryDto {
  id: string;
  name: string;
  number_of_instances: number;
}

export interface PlannedEquipmentSummaryDto {
  id: string;
  name: string;
  planned_cost_eur: string;
}

export interface PlannedOtherCostSummaryDto {
  id: string;
  name: string;
  amount_eur: string;
}

export type EntryStatus = 'Pending' | 'Approved' | 'Rejected';

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
  calendar_year: number | null;
  calendar_month: number | null;
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
  linked_deliverable_ids: string[];
}

export interface MilestoneDetailDto {
  id: string;
  title: string;
  work_package_id: number;
  planned_month: number;
  status: MilestoneStatus;
  effective_status: MilestoneStatus;
  actual_completion_month: number | null;
  linked_deliverable_ids: string[];
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

// ─── M-08: Travel Tracking ──────────────────────────────────────────────────────

export interface TripExecutionInputDto {
  trip_id: string;
  instance_number: number;
  traveller_person_id: string;
  actual_travel_date: string;
  actual_cost_eur: string;
  status: EntryStatus;
}

export interface TripExecutionDetailDto {
  id: string;
  trip_id: string;
  trip_name: string;
  instance_number: number;
  traveller_person_id: string;
  traveller_name: string;
  actual_travel_date: string;
  actual_cost_eur: string;
  status: EntryStatus;
  planned_cost_per_instance_eur: string;
  overspend_warning: boolean;
}

// ─── M-09: Equipment Tracking ───────────────────────────────────────────────────

export interface EquipmentProcurementInputDto {
  equipment_item_id: string;
  actual_purchase_cost_eur: string;
  purchase_date: string;
  delivery_confirmed: boolean;
}

export interface EquipmentProcurementDetailDto {
  id: string;
  equipment_item_id: string;
  equipment_item_name: string;
  actual_purchase_cost_eur: string;
  purchase_date: string;
  delivery_confirmed: boolean;
  actual_eligible_depreciation_eur: string | null;
  overspend_warning: boolean;
}

// ─── M-10: Other Costs Tracking ─────────────────────────────────────────────────

export interface ActualCostEntryInputDto {
  linked_entity_id: string | null;
  amount_eur: string;
  description: string;
  incurred_date: string;
  status: EntryStatus;
  justification: string | null;
}

export interface ActualCostEntryDetailDto {
  id: string;
  linked_entity_id: string | null;
  linked_entity_name: string | null;
  amount_eur: string;
  description: string;
  incurred_date: string;
  status: EntryStatus;
  justification: string | null;
  overspend_warning: boolean;
}

// ─── M-11: Subcontracting Tracking ──────────────────────────────────────────────

export interface SubcontractingLineInputDto {
  vendor: string;
  contract_reference: string;
  amount_eur: string;
  work_package_id: number;
  status: EntryStatus;
  vendor_is_host_institution: boolean;
  payment_date: string | null;
}

export interface SubcontractingLineDetailDto {
  id: string;
  vendor: string;
  contract_reference: string;
  amount_eur: string;
  work_package_id: number;
  status: EntryStatus;
  vendor_is_host_institution: boolean;
  payment_date: string | null;
  competitive_tender_warning: boolean;
  host_institution_warning: boolean;
}

// ─── M-07: Financial Reporting (Planned vs. Actual) ─────────────────────────────

export interface ActualFinancialsDto {
  a_actual: string;
  b_actual: string;
  c1_actual: string;
  c2_actual: string;
  c3_actual: string;
  e_actual: string;
  total_direct_actual: string;
  total_eligible_actual: string;
  requested_eu_contribution_actual: string;
  cfs_status_actual: CfsStatus;
  category_a_overrun: boolean;
  category_b_overrun: boolean;
  category_c1_overrun: boolean;
  category_c2_overrun: boolean;
  category_c3_overrun: boolean;
}

// ─── M-05: Deliverable Tracking ─────────────────────────────────────────────────

export type DeliverableType =
  | 'Report'
  | 'Dataset'
  | 'Software'
  | 'Prototype'
  | 'Dem'
  | 'Ethics'
  | 'Other';

export type DisseminationLevel = 'Public' | 'RestrictedToProgramme' | 'Confidential';

export type DeliverableStatus =
  | 'NotStarted'
  | 'InProgress'
  | 'Submitted'
  | 'Accepted'
  | 'Rejected'
  | 'Revised';

export interface DeliverableInputDto {
  title: string;
  deliverable_type: DeliverableType;
  work_package_id: number;
  planned_month: number;
  responsible_role_id: string;
  dissemination_level: DisseminationLevel;
  status: DeliverableStatus;
  actual_submission_date: string | null;
  revision_note: string | null;
  revised_planned_month: number | null;
  cordis_registered: boolean;
  notes: string | null;
}

export interface DeliverableDetailDto {
  id: string;
  deliverable_number: string;
  title: string;
  deliverable_type: DeliverableType;
  work_package_id: number;
  planned_month: number;
  responsible_role_id: string;
  responsible_role_label: string;
  dissemination_level: DisseminationLevel;
  status: DeliverableStatus;
  actual_submission_date: string | null;
  revision_note: string | null;
  revised_planned_month: number | null;
  cordis_registered: boolean;
  notes: string | null;
  is_overdue: boolean;
  cordis_warning: boolean;
  reporting_period_number: number | null;
}

// ─── M-14: Reporting Period Management ──────────────────────────────────────────

export type ReportingPeriodStatus = 'Open' | 'Submitted';

export interface ReportingPeriodInputDto {
  start_month: number;
  end_month: number;
  submission_deadline: string | null;
  technical_report_submitted: boolean;
  financial_report_submitted: boolean;
  status: ReportingPeriodStatus;
}

export interface ReportingPeriodDetailDto {
  id: string;
  period_number: number;
  start_month: number;
  end_month: number;
  submission_deadline: string | null;
  technical_report_submitted: boolean;
  financial_report_submitted: boolean;
  status: ReportingPeriodStatus;
  deliverables_due: number;
  deliverables_submitted: number;
}

export interface ReportingPeriodCoverageDto {
  gaps_detected: boolean;
  final_period_covers_project_end: boolean;
}

export interface ExecutionProjectSummaryDto {
  project_info: ProjectInfoDto;
  planned: BudgetSummaryDto;
  current_project_month: number;
  personnel_roles: PersonnelRoleSummaryDto[];
  planned_trips: PlannedTripSummaryDto[];
  planned_equipment: PlannedEquipmentSummaryDto[];
  planned_other_costs: PlannedOtherCostSummaryDto[];
  persons: PersonDetailDto[];
  person_months: PersonMonthDetailDto[];
  work_packages: WorkPackageExecutionDetailDto[];
  milestones: MilestoneDetailDto[];
  amendments: AmendmentDetailDto[];
  actuals: ActualFinancialsDto;
  trip_executions: TripExecutionDetailDto[];
  equipment_procurements: EquipmentProcurementDetailDto[];
  actual_cost_entries: ActualCostEntryDetailDto[];
  subcontracting_lines: SubcontractingLineDetailDto[];
  deliverables: DeliverableDetailDto[];
  reporting_periods: ReportingPeriodDetailDto[];
  reporting_period_coverage: ReportingPeriodCoverageDto;
  risks: RiskEntryDetailDto[];
  issues: IssueEntryDetailDto[];
  warnings: WarningDto[];
}

// ─── M-12: Risk Register ────────────────────────────────────────────────────────

export type Level = 'Low' | 'Medium' | 'High';

export type RiskStatus = 'Open' | 'Mitigated' | 'Closed';

export interface RiskEntryInputDto {
  title: string;
  description: string;
  work_package_id: number | null;
  probability: Level;
  impact: Level;
  mitigation: string | null;
  status: RiskStatus;
  owner_role_id: string | null;
  identified_date: string;
  review_date: string | null;
  closed_date: string | null;
}

export interface RiskEntryDetailDto {
  id: string;
  title: string;
  description: string;
  work_package_id: number | null;
  probability: Level;
  impact: Level;
  mitigation: string | null;
  status: RiskStatus;
  owner_role_id: string | null;
  owner_role_label: string | null;
  identified_date: string;
  review_date: string | null;
  closed_date: string | null;
  risk_score: number;
  priority: Level;
}

// ─── M-13: Issue Log ─────────────────────────────────────────────────────────────

export type IssueStatus = 'Open' | 'Closed';

export interface IssueEntryInputDto {
  description: string;
  work_package_id: number | null;
  raised_date: string;
  priority: Level;
  owner_role_id: string | null;
  status: IssueStatus;
  resolution: string | null;
  linked_risk_id: string | null;
}

export interface IssueEntryDetailDto {
  id: string;
  description: string;
  work_package_id: number | null;
  raised_date: string;
  priority: Level;
  owner_role_id: string | null;
  owner_role_label: string | null;
  status: IssueStatus;
  resolution: string | null;
  linked_risk_id: string | null;
  is_stale_warning: boolean;
}

// ─── M-21: Notifications & Warnings ─────────────────────────────────────────────

export type WarningSeverity = 'Error' | 'Warning' | 'Info';

export type NavigationTarget =
  | 'Dashboard'
  | 'WorkPackages'
  | 'Deliverables'
  | 'Milestones'
  | 'Personnel'
  | 'Travel'
  | 'Equipment'
  | 'ReportingPeriods'
  | 'RiskRegister'
  | 'IssueLog';

export interface WarningDto {
  code: string;
  severity: WarningSeverity;
  message: string;
  navigation_target: NavigationTarget;
  entity_id: string | null;
}

export type ExecutionScreen =
  | 'welcome'
  | 'dashboard'
  | 'personnel'
  | 'work-packages'
  | 'milestones'
  | 'deliverables'
  | 'amendments'
  | 'travel'
  | 'equipment'
  | 'other-costs'
  | 'subcontracting'
  | 'reporting-periods'
  | 'risk-register'
  | 'issue-log'
  | 'reports-export';
