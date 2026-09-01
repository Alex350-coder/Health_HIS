import { useQuery, type UseQueryResult } from '@tanstack/react-query';

import { AppErrorException } from '@shared/errors/app-error';
import { callCommand } from '@shared/lib/api-client';

import type { BillingSimulationDetail } from '../types/billing-schemas';

/**
 * `null` means "no simulation has been generated for this encounter yet" (`AppError::NotFound`
 * from `billing_get_simulation`) — an expected, unexceptional state, not a hard error. Any other
 * error still rejects the query normally.
 */
export function useBillingSimulation(
  encounterId: number | undefined,
): UseQueryResult<BillingSimulationDetail | null> {
  const hasId = typeof encounterId === 'number';
  return useQuery({
    queryKey: ['billing', hasId ? encounterId : 0],
    queryFn: async () => {
      try {
        return await callCommand<BillingSimulationDetail>('billing_get_simulation', {
          encounterId: hasId ? encounterId : 0,
        });
      } catch (error) {
        if (error instanceof AppErrorException && error.appError.type === 'NotFound') {
          return null;
        }
        throw error;
      }
    },
    enabled: hasId,
  });
}
