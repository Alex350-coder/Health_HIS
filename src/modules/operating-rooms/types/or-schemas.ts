import { z } from 'zod';

/**
 * Mirrors `src-tauri/src/validation/operating_room_validation.rs` exactly (Rule 17.4).
 * Server-side validation is authoritative; this schema exists for immediate UX feedback only.
 */
export const createOperatingRoomSchema = z.object({
  roomId: z.coerce.number().int().positive('A room must be selected.'),
});

export type CreateOperatingRoomInput = z.infer<typeof createOperatingRoomSchema>;
export type CreateOperatingRoomFormValues = z.input<typeof createOperatingRoomSchema>;

const scheduledRangeRefinement = (data: { scheduledStart: string; scheduledEnd: string }) =>
  data.scheduledStart < data.scheduledEnd;

export const createOrReservationSchema = z
  .object({
    operatingRoomId: z.coerce.number().int().positive('An operating room must be selected.'),
    patientId: z.coerce.number().int().positive(),
    encounterId: z.coerce.number().int().positive('An open encounter is required.'),
    procedureDescription: z
      .string()
      .min(1, 'Procedure description is required.')
      .max(2000, 'Procedure description is too long.'),
    scheduledStart: z.string().min(1, 'Start time is required.'),
    scheduledEnd: z.string().min(1, 'End time is required.'),
  })
  .refine(scheduledRangeRefinement, {
    message: 'End time must be after start time.',
    path: ['scheduledEnd'],
  });

export type CreateOrReservationInput = z.infer<typeof createOrReservationSchema>;
export type CreateOrReservationFormValues = z.input<typeof createOrReservationSchema>;

export const updateOrReservationSchema = z
  .object({
    id: z.coerce.number().int().positive(),
    procedureDescription: z
      .string()
      .min(1, 'Procedure description is required.')
      .max(2000, 'Procedure description is too long.'),
    scheduledStart: z.string().min(1, 'Start time is required.'),
    scheduledEnd: z.string().min(1, 'End time is required.'),
  })
  .refine(scheduledRangeRefinement, {
    message: 'End time must be after start time.',
    path: ['scheduledEnd'],
  });

export type UpdateOrReservationInput = z.infer<typeof updateOrReservationSchema>;
export type UpdateOrReservationFormValues = z.input<typeof updateOrReservationSchema>;

export const cancelOrReservationSchema = z.object({
  id: z.coerce.number().int().positive(),
});

export type CancelOrReservationInput = z.infer<typeof cancelOrReservationSchema>;
export type CancelOrReservationFormValues = z.input<typeof cancelOrReservationSchema>;

export interface OperatingRoom {
  id: number;
  roomId: number;
  name: string;
  createdAt: string;
}

export type OrReservationStatus = 'scheduled' | 'in_progress' | 'completed' | 'cancelled';

export interface OrReservation {
  id: number;
  operatingRoomId: number;
  patientId: number;
  encounterId: number;
  procedureDescription: string;
  scheduledStart: string;
  scheduledEnd: string;
  status: OrReservationStatus;
  scheduledByUserId: number;
  createdAt: string;
  updatedAt: string | null;
}
