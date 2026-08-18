import { useQuery, type UseQueryResult } from '@tanstack/react-query';

import { callCommand } from '@shared/lib/api-client';

import type { Bed, BedSummary, Floor, Room } from '../types/bed-schemas';

/**
 * Beds owns the write path for floors/rooms/beds but reads the same read-only commands the
 * Hospital Map module exposes (Architecture.md Section 3). Cross-module imports between
 * component/hook trees are forbidden (CLAUDE.md Section 5), so this hook re-issues the same
 * `hospital_map_get_*` calls rather than importing `@modules/hospital-map`'s hooks.
 */
export interface FacilityFloor extends Floor {
  rooms: Room[];
}

export interface FacilityRoomStatus {
  room: Room;
  beds: Bed[];
  availableCount: number;
  occupiedCount: number;
  maintenanceCount: number;
}

export function useFacilityLayout(): UseQueryResult<FacilityFloor[]> {
  return useQuery({
    queryKey: ['beds', 'facility-layout'],
    queryFn: () => callCommand<FacilityFloor[]>('hospital_map_get_layout', {}),
  });
}

export function useFacilityRoomStatus(roomId: number | null): UseQueryResult<FacilityRoomStatus> {
  return useQuery({
    queryKey: ['beds', 'facility-room-status', roomId],
    queryFn: () => callCommand<FacilityRoomStatus>('hospital_map_get_room_status', { roomId }),
    enabled: roomId !== null,
  });
}

export function useBedsList(roomId?: number | null): UseQueryResult<BedSummary[]> {
  return useQuery({
    queryKey: ['beds', 'list', roomId ?? null],
    queryFn: () => callCommand<BedSummary[]>('beds_list', { roomId: roomId ?? null }),
  });
}
