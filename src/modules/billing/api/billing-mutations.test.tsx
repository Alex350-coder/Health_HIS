import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { act, renderHook } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { useFinalizeBillingSimulation, useGenerateBillingSimulation } from './billing-mutations';

import type {
  BillingSimulationDetail,
  FinalizeBillingSimulationInput,
  GenerateBillingSimulationInput,
} from '../types/billing-schemas';
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
  items: [],
};

describe('billing-mutations', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('useGenerateBillingSimulation calls billing_generate_simulation with the input', async () => {
    const input: GenerateBillingSimulationInput = { encounterId: 1 };
    mockedInvoke.mockResolvedValueOnce(DETAIL);
    const { result } = renderHook(() => useGenerateBillingSimulation(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockedInvoke).toHaveBeenCalledWith('billing_generate_simulation', { input });
  });

  it('useFinalizeBillingSimulation calls billing_finalize_simulation with the input', async () => {
    const input: FinalizeBillingSimulationInput = { id: 1 };
    mockedInvoke.mockResolvedValueOnce({
      ...DETAIL,
      simulation: { ...DETAIL.simulation, status: 'finalized' },
    });
    const { result } = renderHook(() => useFinalizeBillingSimulation(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockedInvoke).toHaveBeenCalledWith('billing_finalize_simulation', { input });
  });
});
