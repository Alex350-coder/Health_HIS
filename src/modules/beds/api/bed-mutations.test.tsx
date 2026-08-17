import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { renderHook, act } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { useCreateBed, useCreateFloor, useCreateRoom, useSetBedStatus } from './bed-mutations';

import type {
  Bed,
  CreateBedInput,
  CreateFloorInput,
  CreateRoomInput,
  Floor,
  Room,
  SetBedStatusInput,
} from '../types/bed-schemas';
import type { ReactNode } from 'react';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

function wrapper({ children }: { children: ReactNode }): JSX.Element {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

const FLOOR: Floor = {
  id: 1,
  name: 'Ground Floor',
  levelOrder: 0,
  createdAt: '2026-01-01T00:00:00Z',
};

const ROOM: Room = {
  id: 1,
  floorId: 1,
  name: 'Ward A',
  roomType: 'ward',
  mapX: 0.5,
  mapY: 0.5,
  createdAt: '2026-01-01T00:00:00Z',
};

const BED: Bed = {
  id: 1,
  roomId: 1,
  label: 'Bed 1A',
  status: 'available',
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: null,
};

describe('bed-mutations', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('useCreateFloor calls beds_create_floor with the input', async () => {
    const input: CreateFloorInput = { name: 'Ground Floor', levelOrder: 0 };
    mockedInvoke.mockResolvedValueOnce(FLOOR);
    const { result } = renderHook(() => useCreateFloor(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockedInvoke).toHaveBeenCalledWith('beds_create_floor', { input });
  });

  it('useCreateRoom calls beds_create_room with the input', async () => {
    const input: CreateRoomInput = {
      floorId: 1,
      name: 'Ward A',
      roomType: 'ward',
      mapX: 0.5,
      mapY: 0.5,
    };
    mockedInvoke.mockResolvedValueOnce(ROOM);
    const { result } = renderHook(() => useCreateRoom(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockedInvoke).toHaveBeenCalledWith('beds_create_room', { input });
  });

  it('useCreateBed calls beds_create with the input', async () => {
    const input: CreateBedInput = { roomId: 1, label: 'Bed 1A' };
    mockedInvoke.mockResolvedValueOnce(BED);
    const { result } = renderHook(() => useCreateBed(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockedInvoke).toHaveBeenCalledWith('beds_create', { input });
  });

  it('useSetBedStatus calls beds_set_status with the input', async () => {
    const input: SetBedStatusInput = { bedId: 1, status: 'maintenance' };
    mockedInvoke.mockResolvedValueOnce({ ...BED, status: 'maintenance' });
    const { result } = renderHook(() => useSetBedStatus(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockedInvoke).toHaveBeenCalledWith('beds_set_status', { input });
  });
});
