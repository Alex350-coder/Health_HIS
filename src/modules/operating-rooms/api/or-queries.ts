import { useQuery, type UseQueryResult } from '@tanstack/react-query';

import { callCommand } from '@shared/lib/api-client';

import type { OperatingRoom, OrReservation } from '../types/or-schemas';

export function useOperatingRoomsList(): UseQueryResult<OperatingRoom[]> {
  return useQuery({
    queryKey: ['operating-rooms', 'list'],
    queryFn: () => callCommand<OperatingRoom[]>('operating_rooms_list', {}),
  });
}

export function useOrReservationsList(
  operatingRoomId?: number | null,
): UseQueryResult<OrReservation[]> {
  return useQuery({
    queryKey: ['operating-rooms', 'reservations', operatingRoomId ?? null],
    queryFn: () =>
      callCommand<OrReservation[]>('operating_rooms_list_reservations', {
        operatingRoomId: operatingRoomId ?? null,
      }),
  });
}
