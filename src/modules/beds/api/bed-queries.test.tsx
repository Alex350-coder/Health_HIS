import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { useBedsList } from './bed-queries';

import type { BedSummary } from '../types/bed-schemas';
import type { ReactNode } from 'react';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

function wrapper({ children }: { children: ReactNode }): JSX.Element {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

const BED_SUMMARY: BedSummary = {
  id: 1,
  roomId: 1,
  label: 'Bed 1A',
  status: 'occupied',
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: null,
  activeAssignment: { id: 1, patientId: 1, encounterId: 1 },
};

describe('useBedsList', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('calls beds_list with a null roomId by default', async () => {
    mockedInvoke.mockResolvedValueOnce([BED_SUMMARY]);
    const { result } = renderHook(() => useBedsList(), { wrapper });

    await waitFor(() => {
      expect(result.current.data).toEqual([BED_SUMMARY]);
    });
    expect(mockedInvoke).toHaveBeenCalledWith('beds_list', { roomId: null });
  });

  it('calls beds_list with the given roomId', async () => {
    mockedInvoke.mockResolvedValueOnce([BED_SUMMARY]);
    const { result } = renderHook(() => useBedsList(1), { wrapper });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });
    expect(mockedInvoke).toHaveBeenCalledWith('beds_list', { roomId: 1 });
  });
});
