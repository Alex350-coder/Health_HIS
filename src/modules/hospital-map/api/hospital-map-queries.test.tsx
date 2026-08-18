import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { useHospitalMapLayout, useRoomStatus } from './hospital-map-queries';

import type { FloorLayout, RoomStatus } from '../types/hospital-map-schemas';
import type { ReactNode } from 'react';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

function wrapper({ children }: { children: ReactNode }): JSX.Element {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

const LAYOUT: FloorLayout[] = [
  {
    id: 1,
    name: 'Ground Floor',
    levelOrder: 0,
    createdAt: '2026-01-01T00:00:00Z',
    rooms: [
      {
        id: 1,
        floorId: 1,
        name: 'Ward A',
        roomType: 'ward',
        mapX: 0.5,
        mapY: 0.5,
        createdAt: '2026-01-01T00:00:00Z',
      },
    ],
  },
];

const ROOM_STATUS: RoomStatus = {
  room: {
    id: 1,
    floorId: 1,
    name: 'Ward A',
    roomType: 'ward',
    mapX: 0.5,
    mapY: 0.5,
    createdAt: '2026-01-01T00:00:00Z',
  },
  beds: [],
  availableCount: 0,
  occupiedCount: 0,
  maintenanceCount: 0,
};

describe('hospital-map-queries', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('useHospitalMapLayout calls hospital_map_get_layout', async () => {
    mockedInvoke.mockResolvedValueOnce(LAYOUT);
    const { result } = renderHook(() => useHospitalMapLayout(), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(mockedInvoke).toHaveBeenCalledWith('hospital_map_get_layout', {});
    expect(result.current.data).toEqual(LAYOUT);
  });

  it('useRoomStatus calls hospital_map_get_room_status with the room id', async () => {
    mockedInvoke.mockResolvedValueOnce(ROOM_STATUS);
    const { result } = renderHook(() => useRoomStatus(1), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(mockedInvoke).toHaveBeenCalledWith('hospital_map_get_room_status', { roomId: 1 });
    expect(result.current.data).toEqual(ROOM_STATUS);
  });

  it('useRoomStatus does not call the command when roomId is null', () => {
    renderHook(() => useRoomStatus(null), { wrapper });

    expect(mockedInvoke).not.toHaveBeenCalled();
  });
});
