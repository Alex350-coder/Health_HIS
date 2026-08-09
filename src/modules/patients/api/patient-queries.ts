import { useQuery, type UseQueryResult } from '@tanstack/react-query';

import { callCommand } from '@shared/lib/api-client';

import type { Patient } from '../types/patient-schemas';

export interface PatientsListParams {
  search?: string;
  limit: number;
  offset: number;
}

export function usePatientsList(
  params: PatientsListParams,
  enabled: boolean,
): UseQueryResult<Patient[]> {
  return useQuery({
    queryKey: ['patients', 'list', params],
    queryFn: () =>
      callCommand<Patient[]>('patients_list', {
        input: { search: params.search ?? null, limit: params.limit, offset: params.offset },
      }),
    enabled,
  });
}

export function usePatientDetail(id: number, enabled: boolean): UseQueryResult<Patient> {
  return useQuery({
    queryKey: ['patients', 'detail', id],
    queryFn: () => callCommand<Patient>('patients_get', { input: { id } }),
    enabled,
  });
}
