import { z } from 'zod';

/**
 * Mirrors `src-tauri/src/validation/inventory_validation.rs` exactly (Rule 17.4).
 * Server-side validation is authoritative; this schema exists for immediate UX feedback only.
 */
export const INVENTORY_CATEGORY_KINDS = ['medicine', 'supply', 'equipment'] as const;
export type InventoryCategoryKind = (typeof INVENTORY_CATEGORY_KINDS)[number];

export const INVENTORY_TRANSACTION_REASONS = [
  'restock',
  'consumption',
  'adjustment',
  'disposal',
] as const;
export type InventoryTransactionReason = (typeof INVENTORY_TRANSACTION_REASONS)[number];

export const createInventoryCategorySchema = z.object({
  name: z.string().min(1, 'Name is required.').max(100, 'Name is too long.'),
  kind: z.enum(INVENTORY_CATEGORY_KINDS),
});

export type CreateInventoryCategoryInput = z.infer<typeof createInventoryCategorySchema>;
export type CreateInventoryCategoryFormValues = z.input<typeof createInventoryCategorySchema>;

export const createInventoryItemSchema = z.object({
  categoryId: z.coerce.number().int().positive('A category must be selected.'),
  name: z.string().min(1, 'Name is required.').max(200, 'Name is too long.'),
  unit: z.string().min(1, 'Unit is required.').max(20, 'Unit is too long.'),
  reorderThreshold: z.coerce.number().int().min(0, 'Reorder threshold cannot be negative.'),
  expirationDate: z.string().optional(),
  location: z.string().max(200, 'Location is too long.').optional(),
});

export type CreateInventoryItemInput = z.infer<typeof createInventoryItemSchema>;
export type CreateInventoryItemFormValues = z.input<typeof createInventoryItemSchema>;

export const createInventoryTransactionSchema = z.object({
  itemId: z.coerce.number().int().positive(),
  quantityDelta: z.coerce
    .number()
    .int()
    .refine((value) => value !== 0, {
      message: 'Quantity must not be zero.',
    }),
  reason: z.enum(INVENTORY_TRANSACTION_REASONS),
  encounterId: z.coerce.number().int().positive().optional(),
  treatmentId: z.coerce.number().int().positive().optional(),
});

export type CreateInventoryTransactionInput = z.infer<typeof createInventoryTransactionSchema>;
export type CreateInventoryTransactionFormValues = z.input<typeof createInventoryTransactionSchema>;

export const scheduleMaintenanceSchema = z.object({
  inventoryItemId: z.coerce.number().int().positive(),
  scheduledDate: z.string().min(1, 'Scheduled date is required.'),
  notes: z.string().max(2000, 'Notes are too long.').optional(),
});

export type ScheduleMaintenanceInput = z.infer<typeof scheduleMaintenanceSchema>;
export type ScheduleMaintenanceFormValues = z.input<typeof scheduleMaintenanceSchema>;

export interface InventoryCategory {
  id: number;
  name: string;
  kind: InventoryCategoryKind;
}

export interface InventoryItem {
  id: number;
  categoryId: number;
  name: string;
  quantity: number;
  unit: string;
  reorderThreshold: number;
  expirationDate: string | null;
  location: string | null;
  createdAt: string;
  updatedAt: string | null;
}

export interface InventoryTransaction {
  id: number;
  itemId: number;
  quantityDelta: number;
  reason: InventoryTransactionReason;
  encounterId: number | null;
  treatmentId: number | null;
  performedByUserId: number;
  createdAt: string;
}

export interface MaintenanceSchedule {
  id: number;
  inventoryItemId: number;
  scheduledDate: string;
  completedDate: string | null;
  notes: string | null;
  createdAt: string;
}
