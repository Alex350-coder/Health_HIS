import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { usePatientDetail, usePatientsList } from './patient-queries';

import type { Patient } from '../types/patient-schemas';
import type { ReactNode } from 'react';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

function wrapper({ children }: { children: ReactNode }): JSX.Element {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

const PATIENT: Patient = {
  id: 1,
  medicalRecordNumber: 'MRN-0001',
  fullName: 'Ada Lovelace',
  dateOfBirth: '1990-01-01',
  sex: 'female',
  nationalId: null,
  phone: null,
  address: null,
  emergencyContactName: null,
  emergencyContactPhone: null,
  bloodType: null,
  allergies: null,
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: null,
};

describe('patient-queries', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('usePatientsList does not fetch when disabled', () => {
    renderHook(() => usePatientsList({ limit: 20, offset: 0 }, false), { wrapper });

    expect(mockedInvoke).not.toHaveBeenCalled();
  });

  it('usePatientsList fetches with the search/limit/offset input when enabled', async () => {
    mockedInvoke.mockResolvedValueOnce([PATIENT]);

    const { result } = renderHook(
      () => usePatientsList({ search: 'Ada', limit: 20, offset: 0 }, true),
      { wrapper },
    );

    await waitFor(() => {
      expect(result.current.data).toEqual([PATIENT]);
    });
    expect(mockedInvoke).toHaveBeenCalledWith('patients_list', {
      input: { search: 'Ada', limit: 20, offset: 0 },
    });
  });

  it('usePatientDetail does not fetch when disabled', () => {
    renderHook(() => usePatientDetail(1, false), { wrapper });

    expect(mockedInvoke).not.toHaveBeenCalled();
  });

  it('usePatientDetail fetches the patient by id when enabled', async () => {
    mockedInvoke.mockResolvedValueOnce(PATIENT);

    const { result } = renderHook(() => usePatientDetail(1, true), { wrapper });

    await waitFor(() => {
      expect(result.current.data).toEqual(PATIENT);
    });
    expect(mockedInvoke).toHaveBeenCalledWith('patients_get', { input: { id: 1 } });
  });
});
