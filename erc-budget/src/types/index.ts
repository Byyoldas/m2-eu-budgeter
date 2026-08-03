/**
 * TypeScript types for the ERC Budget frontend.
 *
 * The DTOs, entity enums, and error types below are generated directly from
 * the Rust structs in `erc-core` via ts-rs (Milestone 1, Step 8 — see
 * docs/executer/shared-core-roadmap.md §6). Do not hand-edit
 * `erc-core/bindings/*.ts` — regenerate with:
 *   cargo test -p erc-core --features ts-rs
 *
 * A handful of input DTOs are re-exported here under this app's established
 * names (`XInput` rather than the Rust struct's `XInputDto`/`XDto`) purely so
 * every existing screen/component import keeps resolving unchanged — the
 * underlying shape is identical either way.
 *
 * Only app-specific UI state (`Screen`/`SCREENS`/`SCREEN_LABELS`) is still
 * hand-written in this file.
 */

// ─── Enums ─────────────────────────────────────────────────────────────────────

export type { RoleType } from '../../../erc-core/bindings/RoleType';
export type { CfsStatus } from '../../../erc-core/bindings/CfsStatus';

// ─── Trip Types ─────────────────────────────────────────────────────────────────

export type { TripType } from '../../../erc-core/bindings/TripType';

// ─── Input DTOs (frontend → backend) ──────────────────────────────────────────

export type { ProjectConfigDto as ProjectConfigInput } from '../../../erc-core/bindings/ProjectConfigDto';
export type { PersonnelRoleInputDto as PersonnelRoleInput } from '../../../erc-core/bindings/PersonnelRoleInputDto';
export type { EquipmentItemInputDto as EquipmentItemInput } from '../../../erc-core/bindings/EquipmentItemInputDto';
export type { TripInputDto as TripInput } from '../../../erc-core/bindings/TripInputDto';
export type { OtherCostInputDto as OtherCostInput } from '../../../erc-core/bindings/OtherCostInputDto';

// ─── Output DTOs (backend → frontend) ────────────────────────────────────────

export type { WpCostAmountDto } from '../../../erc-core/bindings/WpCostAmountDto';
export type { RoleCostLineDto } from '../../../erc-core/bindings/RoleCostLineDto';
export type { PersonnelRoleDetailDto } from '../../../erc-core/bindings/PersonnelRoleDetailDto';
export type { RoleCostPreviewDto } from '../../../erc-core/bindings/RoleCostPreviewDto';
export type { EquipmentItemDetailDto } from '../../../erc-core/bindings/EquipmentItemDetailDto';
export type { EquipmentPreviewDto } from '../../../erc-core/bindings/EquipmentPreviewDto';
export type { OtherCostItemDetailDto } from '../../../erc-core/bindings/OtherCostItemDetailDto';
export type { TripDetailDto } from '../../../erc-core/bindings/TripDetailDto';
export type { TripCostPreviewDto } from '../../../erc-core/bindings/TripCostPreviewDto';
export type { WpBudgetDto } from '../../../erc-core/bindings/WpBudgetDto';
export type { BudgetSummaryDto } from '../../../erc-core/bindings/BudgetSummaryDto';

// ─── Rate Data Types ──────────────────────────────────────────────────────────

export type { RateVersionSummary } from '../../../erc-core/bindings/RateVersionSummary';
export type { CountrySummary } from '../../../erc-core/bindings/CountrySummary';

// ─── App Error ────────────────────────────────────────────────────────────────

export type { FieldError } from '../../../erc-core/bindings/FieldError';
export type { AppError } from '../../../erc-core/bindings/AppError';

// ─── App UI State ─────────────────────────────────────────────────────────────

export type Screen =
  | 'welcome'
  | 'project-setup'
  | 'budget-settings'
  | 'work-packages'
  | 'personnel'
  | 'equipment'
  | 'travel'
  | 'other-costs'
  | 'review-export';

export const SCREENS: Screen[] = [
  'project-setup',
  'budget-settings',
  'work-packages',
  'personnel',
  'equipment',
  'travel',
  'other-costs',
  'review-export',
];

export const SCREEN_LABELS: Record<Screen, string> = {
  'welcome': 'Welcome',
  'project-setup': 'Project Setup',
  'budget-settings': 'Budget Settings',
  'work-packages': 'Work Packages',
  'personnel': 'Personnel (A)',
  'equipment': 'Equipment (C2)',
  'travel': 'Travel (C1)',
  'other-costs': 'Other Costs (C3)',
  'review-export': 'Review & Export',
};
