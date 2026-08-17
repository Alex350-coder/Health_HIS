import { useQuery, type UseQueryResult } from '@tanstack/react-query';

import { callCommand } from '@shared/lib/api-client';

import type { FloorLayout, RoomStatus } from '../types/hospital-map-schemas';

export function useHospitalMapLayout(): UseQueryResult<FloorLayout[]> {
  return useQuery({
    queryKey: ['hospital-map', 'layout'],
    queryFn: () => callCommand<FloorLayout[]>('hospital_map_get_layout', {}),
  });
}

export function useRoomStatus(roomId: number | null): UseQueryResult<RoomStatus> {
  return useQuery({
    queryKey: ['hospital-map', 'room-status', roomId],
    queryFn: () => callCommand<RoomStatus>('hospital_map_get_room_status', { roomId }),
    enabled: roomId !== null,
  });
}
