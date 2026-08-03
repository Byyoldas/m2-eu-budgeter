/**
 * Zod schemas for front-end form validation.
 *
 * The 6 schemas shared with the future Execution Application
 * (Milestone 1, Step 9 — see docs/executer/shared-core-roadmap.md §6) live
 * in erc-core/ts/schemas.ts and are re-exported below unchanged. Only the
 * two Budget-App-specific schemas (CFS, Subcontracting) are still
 * hand-written in this file.
 */

import { z } from 'zod';
import { decimalStr, nonNegDecimalStr } from '../../../erc-core/ts/schemas';

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
} from '../../../erc-core/ts/schemas';

export type {
  ProjectSetupFormData,
  BudgetSettingsFormData,
  PersonnelRoleFormData,
  EquipmentItemFormData,
  TripFormData,
  OtherCostFormData,
} from '../../../erc-core/ts/schemas';

// ─── CFS Item Schema ──────────────────────────────────────────────────────────

export const cfsItemSchema = z.object({
  amount_eur: decimalStr('CFS amount'),
});

export type CfsItemFormData = z.infer<typeof cfsItemSchema>;

// ─── Subcontracting Schema ────────────────────────────────────────────────────

export const subcontractingSchema = z.object({
  amount_eur: nonNegDecimalStr('Subcontracting amount'),
  work_package_id: z.coerce.number().int().positive('Select a Work Package.'),
});

export type SubcontractingFormData = z.infer<typeof subcontractingSchema>;
