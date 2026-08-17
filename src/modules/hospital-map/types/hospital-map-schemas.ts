import type { Bed, Floor, Room } from '@modules/beds/types/bed-schemas';

/**
 * Read-only entity shapes matching `hospital_map_get_layout`/`hospital_map_get_room_status`
 * (IPC.md Section 2). No Zod schemas here — nothing in this module ever submits a form.
 */
export interface FloorLayout extends Floor {
  rooms: Room[];
}

export interface RoomStatus {
  room: Room;
  beds: Bed[];
  availableCount: number;
  occupiedCount: number;
  maintenanceCount: number;
}
