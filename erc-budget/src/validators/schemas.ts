/**
 * Zod schemas for front-end form validation.
 *
 * The 7 schemas shared with the future Execution Application
 * (Milestone 1, Step 9 — see docs/executer/shared-core-roadmap.md §6) live
 * in erc-core/ts/schemas.ts and are re-exported below unchanged. Only the
 * Budget-App-specific CFS schema is still hand-written in this file.
 */

import { z } from 'zod';
import { decimalStr } from '../../../erc-core/ts/schemas';

export {
  decimalStr,
  nonNegDecimalStr,
  projectSetupSchema,
  budgetSettingsSchema,
  personnelRoleSchema,
  equipmentItemSchema,
  itemizedTripSchema,
  flatTripSchema,
  tripSchema,
  otherCostSchema,
  subcontractingItemSchema,
} from '../../../erc-core/ts/schemas';

export type {
  ProjectSetupFormData,
  BudgetSettingsFormData,
  PersonnelRoleFormData,
  EquipmentItemFormData,
  TripFormData,
  OtherCostFormData,
  SubcontractingItemFormData,
} from '../../../erc-core/ts/schemas';

// ─── CFS Item Schema ──────────────────────────────────────────────────────────

export const cfsItemSchema = z.object({
  amount_eur: decimalStr('CFS amount'),
});

export type CfsItemFormData = z.infer<typeof cfsItemSchema>;
