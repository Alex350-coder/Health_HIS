import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { renderHook, act } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { useCreatePatient, useUpdatePatient } from './patient-mutations';

import type { CreatePatientInput, Patient, UpdatePatientInput } from '../types/patient-schemas';
import type { ReactNode } from 'react';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

function wrapper({ children }: { children: ReactNode }): JSX.Element {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

const CREATE_INPUT: CreatePatientInput = {
  medicalRecordNumber: 'MRN-0001',
  fullName: 'Ada Lovelace',
  dateOfBirth: '1990-01-01',
  sex: 'female',
};

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

describe('patient-mutations', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('useCreatePatient calls patients_create with the input', async () => {
    mockedInvoke.mockResolvedValueOnce(PATIENT);
    const { result } = renderHook(() => useCreatePatient(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync(CREATE_INPUT);
    });

    expect(mockedInvoke).toHaveBeenCalledWith('patients_create', { input: CREATE_INPUT });
  });

  it('useUpdatePatient calls patients_update with the input', async () => {
    const updateInput: UpdatePatientInput = { ...CREATE_INPUT, id: 1 };
    mockedInvoke.mockResolvedValueOnce(PATIENT);
    const { result } = renderHook(() => useUpdatePatient(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync(updateInput);
    });

    expect(mockedInvoke).toHaveBeenCalledWith('patients_update', { input: updateInput });
  });
});
