import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { useOperatingRoomsList, useOrReservationsList } from './or-queries';

import type { OperatingRoom, OrReservation } from '../types/or-schemas';
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

describe('useOperatingRoomsList', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('calls operating_rooms_list with no arguments', async () => {
    mockedInvoke.mockResolvedValueOnce([OPERATING_ROOM]);
    const { result } = renderHook(() => useOperatingRoomsList(), { wrapper });

    await waitFor(() => {
      expect(result.current.data).toEqual([OPERATING_ROOM]);
    });
    expect(mockedInvoke).toHaveBeenCalledWith('operating_rooms_list', {});
  });
});

describe('useOrReservationsList', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('calls operating_rooms_list_reservations with a null operatingRoomId by default', async () => {
    mockedInvoke.mockResolvedValueOnce([RESERVATION]);
    const { result } = renderHook(() => useOrReservationsList(), { wrapper });

    await waitFor(() => {
      expect(result.current.data).toEqual([RESERVATION]);
    });
    expect(mockedInvoke).toHaveBeenCalledWith('operating_rooms_list_reservations', {
      operatingRoomId: null,
    });
  });

  it('calls operating_rooms_list_reservations with the given operatingRoomId', async () => {
    mockedInvoke.mockResolvedValueOnce([RESERVATION]);
    const { result } = renderHook(() => useOrReservationsList(1), { wrapper });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });
    expect(mockedInvoke).toHaveBeenCalledWith('operating_rooms_list_reservations', {
      operatingRoomId: 1,
    });
  });
});
