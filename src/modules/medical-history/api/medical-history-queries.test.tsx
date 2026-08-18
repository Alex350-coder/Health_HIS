import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { useMedicalHistory } from './medical-history-queries';

import type { MedicalHistoryBundle } from '../types/medical-history-schemas';
import type { ReactNode } from 'react';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

function wrapper({ children }: { children: ReactNode }): JSX.Element {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

const BUNDLE: MedicalHistoryBundle = {
  encounters: [],
  diagnoses: [],
  treatments: [],
  evolutions: [],
};

describe('medical-history-queries', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('does not fetch when disabled', () => {
    renderHook(() => useMedicalHistory(1, false), { wrapper });

    expect(mockedInvoke).not.toHaveBeenCalled();
  });

  it('fetches the bundle for the given patient id when enabled', async () => {
    mockedInvoke.mockResolvedValueOnce(BUNDLE);

    const { result } = renderHook(() => useMedicalHistory(1, true), { wrapper });

    await waitFor(() => {
      expect(result.current.data).toEqual(BUNDLE);
    });
    expect(mockedInvoke).toHaveBeenCalledWith('medical_history_get_by_patient', {
      input: { patientId: 1 },
    });
  });
});
