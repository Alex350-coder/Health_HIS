import { z } from 'zod';

/**
 * Mirrors `src-tauri/src/validation/billing_validation.rs` exactly (Rule 17.4).
 * Server-side validation is authoritative; this schema exists for immediate UX feedback only.
 */
export const generateBillingSimulationSchema = z.object({
  encounterId: z.coerce.number().int().positive('An encounter must be selected.'),
});

export type GenerateBillingSimulationInput = z.infer<typeof generateBillingSimulationSchema>;

export const finalizeBillingSimulationSchema = z.object({
  id: z.coerce.number().int().positive(),
});

export type FinalizeBillingSimulationInput = z.infer<typeof finalizeBillingSimulationSchema>;

export const BILLING_ITEM_SOURCES = ['room', 'treatment', 'inventory', 'operating_room'] as const;
export type BillingItemSource = (typeof BILLING_ITEM_SOURCES)[number];

export const BILLING_SIMULATION_STATUSES = ['draft', 'finalized'] as const;
export type BillingSimulationStatus = (typeof BILLING_SIMULATION_STATUSES)[number];

export interface BillingSimulation {
  id: number;
  encounterId: number;
  status: BillingSimulationStatus;
  totalAmount: number;
  generatedByUserId: number;
  createdAt: string;
  updatedAt: string | null;
}

export interface BillingItem {
  id: number;
  billingSimulationId: number;
  description: string;
  source: BillingItemSource;
  sourceEntityId: number | null;
  amount: number;
  createdAt: string;
}

export interface BillingSimulationDetail {
  simulation: BillingSimulation;
  items: BillingItem[];
}
