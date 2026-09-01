import { z } from 'zod';

/**
 * Mirrors `src-tauri/src/validation/notification_validation.rs` exactly (Rule 17.4).
 * Server-side validation is authoritative; this schema exists for immediate UX feedback only.
 */
export const NOTIFICATION_TYPES = [
  'medicine_expiration',
  'maintenance_due',
  'low_stock',
  'or_schedule',
  'other',
] as const;
export type NotificationType = (typeof NOTIFICATION_TYPES)[number];

export const markNotificationReadSchema = z.object({
  id: z.coerce.number().int().positive(),
});

export type MarkNotificationReadInput = z.infer<typeof markNotificationReadSchema>;
export type MarkNotificationReadFormValues = z.input<typeof markNotificationReadSchema>;

export interface Notification {
  id: number;
  type: NotificationType;
  targetRole: string | null;
  message: string;
  relatedEntityType: string | null;
  relatedEntityId: number | null;
  isRead: boolean;
  createdAt: string;
}
