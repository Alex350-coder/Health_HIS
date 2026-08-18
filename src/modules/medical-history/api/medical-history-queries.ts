import { useQuery, type UseQueryResult } from '@tanstack/react-query';

import { callCommand } from '@shared/lib/api-client';

import type { MedicalHistoryBundle } from '../types/medical-history-schemas';

export function useMedicalHistory(
  patientId: number,
  enabled: boolean,
): UseQueryResult<MedicalHistoryBundle> {
  return useQuery({
    queryKey: ['medical-history', 'by-patient', patientId],
    queryFn: () =>
      callCommand<MedicalHistoryBundle>('medical_history_get_by_patient', {
        input: { patientId },
      }),
    enabled,
  });
}
