import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { renderHook, act } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import {
  useCreateDiagnosis,
  useCreateEncounter,
  useCreateEvolution,
  useCreateTreatment,
  useDischargeEncounter,
} from './medical-history-mutations';

import type {
  CreateDiagnosisInput,
  CreateEncounterInput,
  CreateEvolutionInput,
  CreateTreatmentInput,
  Diagnosis,
  DischargeEncounterInput,
  Encounter,
  Evolution,
  Treatment,
} from '../types/medical-history-schemas';
import type { ReactNode } from 'react';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

function wrapper({ children }: { children: ReactNode }): JSX.Element {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

const ENCOUNTER: Encounter = {
  id: 1,
  patientId: 1,
  status: 'open',
  admittedAt: '2026-01-01T00:00:00Z',
  dischargedAt: null,
  dischargeSummary: null,
  createdByUserId: 1,
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: null,
};

const DIAGNOSIS: Diagnosis = {
  id: 1,
  encounterId: 1,
  description: 'Type 2 diabetes',
  icdCode: null,
  registeredByUserId: 1,
  correctsDiagnosisId: null,
  createdAt: '2026-01-01T00:00:00Z',
};

const TREATMENT: Treatment = {
  id: 1,
  encounterId: 1,
  diagnosisId: null,
  description: 'Metformin 500mg',
  dosage: null,
  registeredByUserId: 1,
  correctsTreatmentId: null,
  createdAt: '2026-01-01T00:00:00Z',
};

const EVOLUTION: Evolution = {
  id: 1,
  encounterId: 1,
  note: 'Patient improving.',
  registeredByUserId: 1,
  createdAt: '2026-01-01T00:00:00Z',
};

describe('medical-history-mutations', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('useCreateEncounter calls medical_history_create_encounter with the input', async () => {
    mockedInvoke.mockResolvedValueOnce(ENCOUNTER);
    const { result } = renderHook(() => useCreateEncounter(), { wrapper });
    const input: CreateEncounterInput = { patientId: 1 };

    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockedInvoke).toHaveBeenCalledWith('medical_history_create_encounter', { input });
  });

  it('useDischargeEncounter calls medical_history_discharge_encounter with the input', async () => {
    mockedInvoke.mockResolvedValueOnce(ENCOUNTER);
    const { result } = renderHook(() => useDischargeEncounter(), { wrapper });
    const input: DischargeEncounterInput = { encounterId: 1 };

    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockedInvoke).toHaveBeenCalledWith('medical_history_discharge_encounter', { input });
  });

  it('useCreateDiagnosis calls medical_history_create_diagnosis with the input', async () => {
    mockedInvoke.mockResolvedValueOnce(DIAGNOSIS);
    const { result } = renderHook(() => useCreateDiagnosis(), { wrapper });
    const input: CreateDiagnosisInput = { encounterId: 1, description: 'Type 2 diabetes' };

    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockedInvoke).toHaveBeenCalledWith('medical_history_create_diagnosis', { input });
  });

  it('useCreateTreatment calls medical_history_create_treatment with the input', async () => {
    mockedInvoke.mockResolvedValueOnce(TREATMENT);
    const { result } = renderHook(() => useCreateTreatment(), { wrapper });
    const input: CreateTreatmentInput = { encounterId: 1, description: 'Metformin 500mg' };

    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockedInvoke).toHaveBeenCalledWith('medical_history_create_treatment', { input });
  });

  it('useCreateEvolution calls medical_history_create_evolution with the input', async () => {
    mockedInvoke.mockResolvedValueOnce(EVOLUTION);
    const { result } = renderHook(() => useCreateEvolution(), { wrapper });
    const input: CreateEvolutionInput = { encounterId: 1, note: 'Patient improving.' };

    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockedInvoke).toHaveBeenCalledWith('medical_history_create_evolution', { input });
  });
});
