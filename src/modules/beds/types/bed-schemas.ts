import { z } from 'zod';

/**
 * Mirrors `src-tauri/src/validation/bed_validation.rs` exactly (Rule 17.4). Server-side
 * validation is authoritative; this schema exists for immediate UX feedback only.
 */
const VALID_ROOM_TYPES = [
  'ward',
  'operating_room',
  'pharmacy',
  'laboratory',
  'admin',
  'other',
] as const;

/** `available`/`maintenance` only — occupancy is set by the Phase 7 assign/release flow. */
const SETTABLE_BED_STATUSES = ['available', 'maintenance'] as const;

export const roomTypeSchema = z.enum(VALID_ROOM_TYPES);

export const createFloorSchema = z.object({
  name: z.string().min(1, 'Name is required.').max(200, 'Name is too long.'),
  levelOrder: z.coerce.number().int().min(0, 'Level order cannot be negative.'),
});

export type CreateFloorInput = z.infer<typeof createFloorSchema>;
export type CreateFloorFormValues = z.input<typeof createFloorSchema>;

export const createRoomSchema = z.object({
  floorId: z.coerce.number().int().positive('A floor must be selected.'),
  name: z.string().min(1, 'Name is required.').max(200, 'Name is too long.'),
  roomType: roomTypeSchema,
  mapX: z.coerce
    .number()
    .min(0, 'Map X must be between 0 and 1.')
    .max(1, 'Map X must be between 0 and 1.'),
  mapY: z.coerce
    .number()
    .min(0, 'Map Y must be between 0 and 1.')
    .max(1, 'Map Y must be between 0 and 1.'),
});

export type CreateRoomInput = z.infer<typeof createRoomSchema>;
export type CreateRoomFormValues = z.input<typeof createRoomSchema>;

export const createBedSchema = z.object({
  roomId: z.coerce.number().int().positive('A room must be selected.'),
  label: z.string().min(1, 'Label is required.').max(50, 'Label is too long.'),
});

export type CreateBedInput = z.infer<typeof createBedSchema>;
export type CreateBedFormValues = z.input<typeof createBedSchema>;

export const setBedStatusSchema = z.object({
  bedId: z.coerce.number().int().positive(),
  status: z.enum(SETTABLE_BED_STATUSES),
});

export type SetBedStatusInput = z.infer<typeof setBedStatusSchema>;
export type SetBedStatusFormValues = z.input<typeof setBedStatusSchema>;

export interface Floor {
  id: number;
  name: string;
  levelOrder: number;
  createdAt: string;
}

export interface Room {
  id: number;
  floorId: number;
  name: string;
  roomType: string;
  mapX: number;
  mapY: number;
  createdAt: string;
}

export interface Bed {
  id: number;
  roomId: number;
  label: string;
  status: 'available' | 'occupied' | 'maintenance';
  createdAt: string;
  updatedAt: string | null;
}

export interface BedAssignment {
  id: number;
  bedId: number;
  patientId: number;
  encounterId: number;
  assignedAt: string;
  releasedAt: string | null;
}

export interface ActiveAssignmentRef {
  id: number;
  patientId: number;
  encounterId: number;
}

export interface BedSummary extends Bed {
  activeAssignment: ActiveAssignmentRef | null;
}

export const assignBedSchema = z.object({
  bedId: z.coerce.number().int().positive('A bed must be selected.'),
  patientId: z.coerce.number().int().positive(),
  encounterId: z.coerce.number().int().positive('An open encounter is required.'),
});

export type AssignBedInput = z.infer<typeof assignBedSchema>;
export type AssignBedFormValues = z.input<typeof assignBedSchema>;

export const releaseBedSchema = z.object({
  bedAssignmentId: z.coerce.number().int().positive(),
});

export type ReleaseBedInput = z.infer<typeof releaseBedSchema>;
export type ReleaseBedFormValues = z.input<typeof releaseBedSchema>;
