import { useMutation, useQueryClient, type UseMutationResult } from '@tanstack/react-query';

import { callCommand } from '@shared/lib/api-client';

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

function useInvalidateMedicalHistory(): () => void {
  const queryClient = useQueryClient();
  return () => {
    void queryClient.invalidateQueries({ queryKey: ['medical-history'] });
  };
}

export function useCreateEncounter(): UseMutationResult<Encounter, unknown, CreateEncounterInput> {
  const invalidate = useInvalidateMedicalHistory();
  return useMutation({
    mutationFn: (input: CreateEncounterInput) =>
      callCommand<Encounter>('medical_history_create_encounter', { input }),
    onSuccess: invalidate,
  });
}

export function useDischargeEncounter(): UseMutationResult<
  Encounter,
  unknown,
  DischargeEncounterInput
> {
  const invalidate = useInvalidateMedicalHistory();
  return useMutation({
    mutationFn: (input: DischargeEncounterInput) =>
      callCommand<Encounter>('medical_history_discharge_encounter', { input }),
    onSuccess: invalidate,
  });
}

export function useCreateDiagnosis(): UseMutationResult<Diagnosis, unknown, CreateDiagnosisInput> {
  const invalidate = useInvalidateMedicalHistory();
  return useMutation({
    mutationFn: (input: CreateDiagnosisInput) =>
      callCommand<Diagnosis>('medical_history_create_diagnosis', { input }),
    onSuccess: invalidate,
  });
}

export function useCreateTreatment(): UseMutationResult<Treatment, unknown, CreateTreatmentInput> {
  const invalidate = useInvalidateMedicalHistory();
  return useMutation({
    mutationFn: (input: CreateTreatmentInput) =>
      callCommand<Treatment>('medical_history_create_treatment', { input }),
    onSuccess: invalidate,
  });
}

export function useCreateEvolution(): UseMutationResult<Evolution, unknown, CreateEvolutionInput> {
  const invalidate = useInvalidateMedicalHistory();
  return useMutation({
    mutationFn: (input: CreateEvolutionInput) =>
      callCommand<Evolution>('medical_history_create_evolution', { input }),
    onSuccess: invalidate,
  });
}
