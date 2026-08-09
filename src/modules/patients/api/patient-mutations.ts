import { useMutation, useQueryClient, type UseMutationResult } from '@tanstack/react-query';

import { callCommand } from '@shared/lib/api-client';

import type { CreatePatientInput, Patient, UpdatePatientInput } from '../types/patient-schemas';

export function useCreatePatient(): UseMutationResult<Patient, unknown, CreatePatientInput> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: CreatePatientInput) => callCommand<Patient>('patients_create', { input }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['patients', 'list'] });
    },
  });
}

export function useUpdatePatient(): UseMutationResult<Patient, unknown, UpdatePatientInput> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: UpdatePatientInput) => callCommand<Patient>('patients_update', { input }),
    onSuccess: (patient) => {
      void queryClient.invalidateQueries({ queryKey: ['patients', 'list'] });
      void queryClient.invalidateQueries({ queryKey: ['patients', 'detail', patient.id] });
    },
  });
}
