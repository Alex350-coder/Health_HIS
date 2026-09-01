import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { useBillingSimulation } from './billing-queries';

import type { BillingSimulationDetail } from '../types/billing-schemas';
import type { ReactNode } from 'react';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

function wrapper({ children }: { children: ReactNode }): JSX.Element {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

const DETAIL: BillingSimulationDetail = {
  simulation: {
    id: 1,
    encounterId: 1,
    status: 'draft',
    totalAmount: 1255,
    generatedByUserId: 1,
    createdAt: '2026-01-01T00:00:00Z',
    updatedAt: null,
  },
  items: [
    {
      id: 1,
      billingSimulationId: 1,
      description: 'Room charge',
      source: 'room',
      sourceEntityId: 1,
      amount: 150,
      createdAt: '2026-01-01T00:00:00Z',
    },
  ],
};

describe('useBillingSimulation', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('calls billing_get_simulation with the given encounterId', async () => {
    mockedInvoke.mockResolvedValueOnce(DETAIL);
    const { result } = renderHook(() => useBillingSimulation(1), { wrapper });

    await waitFor(() => {
      expect(result.current.data).toEqual(DETAIL);
    });
    expect(mockedInvoke).toHaveBeenCalledWith('billing_get_simulation', { encounterId: 1 });
  });

  it('resolves to null when no simulation exists yet, instead of erroring', async () => {
    mockedInvoke.mockRejectedValueOnce({ type: 'NotFound', entity: 'billing_simulation', id: 1 });
    const { result } = renderHook(() => useBillingSimulation(1), { wrapper });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });
    expect(result.current.data).toBeNull();
  });

  it('still rejects on a non-NotFound error', async () => {
    mockedInvoke.mockRejectedValueOnce({
      type: 'Unexpected',
      message: 'boom',
      correlationId: 'x',
    });
    const { result } = renderHook(() => useBillingSimulation(1), { wrapper });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });
  });

  it('does not call the command when no encounterId is given', () => {
    renderHook(() => useBillingSimulation(undefined), { wrapper });

    expect(mockedInvoke).not.toHaveBeenCalled();
  });
});
