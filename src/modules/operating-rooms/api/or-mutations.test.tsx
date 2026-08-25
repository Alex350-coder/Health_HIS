import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { renderHook, act } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import {
  useCancelOrReservation,
  useCreateOperatingRoom,
  useReserveOr,
  useUpdateOrReservation,
} from './or-mutations';

import type {
  CancelOrReservationInput,
  CreateOperatingRoomInput,
  CreateOrReservationInput,
  OperatingRoom,
  OrReservation,
  UpdateOrReservationInput,
} from '../types/or-schemas';
import type { ReactNode } from 'react';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

function wrapper({ children }: { children: ReactNode }): JSX.Element {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

const OPERATING_ROOM: OperatingRoom = {
  id: 1,
  roomId: 1,
  name: 'OR Suite',
  createdAt: '2026-01-01T00:00:00Z',
};

const RESERVATION: OrReservation = {
  id: 1,
  operatingRoomId: 1,
  patientId: 1,
  encounterId: 1,
  procedureDescription: 'Appendectomy',
  scheduledStart: '2026-01-01T08:00:00',
  scheduledEnd: '2026-01-01T10:00:00',
  status: 'scheduled',
  scheduledByUserId: 1,
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: null,
};

describe('or-mutations', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('useCreateOperatingRoom calls operating_rooms_create with the input', async () => {
    const input: CreateOperatingRoomInput = { roomId: 1 };
    mockedInvoke.mockResolvedValueOnce(OPERATING_ROOM);
    const { result } = renderHook(() => useCreateOperatingRoom(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockedInvoke).toHaveBeenCalledWith('operating_rooms_create', { input });
  });

  it('useReserveOr calls operating_rooms_reserve with the input', async () => {
    const input: CreateOrReservationInput = {
      operatingRoomId: 1,
      patientId: 1,
      encounterId: 1,
      procedureDescription: 'Appendectomy',
      scheduledStart: '2026-01-01T08:00:00',
      scheduledEnd: '2026-01-01T10:00:00',
    };
    mockedInvoke.mockResolvedValueOnce(RESERVATION);
    const { result } = renderHook(() => useReserveOr(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockedInvoke).toHaveBeenCalledWith('operating_rooms_reserve', { input });
  });

  it('useUpdateOrReservation calls operating_rooms_update_reservation with the input', async () => {
    const input: UpdateOrReservationInput = {
      id: 1,
      procedureDescription: 'Appendectomy — revised',
      scheduledStart: '2026-01-01T09:00:00',
      scheduledEnd: '2026-01-01T11:00:00',
    };
    mockedInvoke.mockResolvedValueOnce({
      ...RESERVATION,
      procedureDescription: input.procedureDescription,
    });
    const { result } = renderHook(() => useUpdateOrReservation(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockedInvoke).toHaveBeenCalledWith('operating_rooms_update_reservation', { input });
  });

  it('useCancelOrReservation calls operating_rooms_cancel_reservation with the input', async () => {
    const input: CancelOrReservationInput = { id: 1 };
    mockedInvoke.mockResolvedValueOnce({ ...RESERVATION, status: 'cancelled' });
    const { result } = renderHook(() => useCancelOrReservation(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockedInvoke).toHaveBeenCalledWith('operating_rooms_cancel_reservation', { input });
  });
});
