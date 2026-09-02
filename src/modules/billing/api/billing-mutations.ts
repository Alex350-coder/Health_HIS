import { useMutation, useQueryClient, type UseMutationResult } from '@tanstack/react-query';

import { callCommand } from '@shared/lib/api-client';

import type {
  BillingSimulationDetail,
  FinalizeBillingSimulationInput,
  GenerateBillingSimulationInput,
} from '../types/billing-schemas';

// No optimistic update on either mutation (StateManagement.md Section 5 — billing is explicitly
// excluded from optimistic updates); both wait for the server response before the UI changes.

export function useGenerateBillingSimulation(): UseMutationResult<
  BillingSimulationDetail,
  unknown,
  GenerateBillingSimulationInput
> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: GenerateBillingSimulationInput) =>
      callCommand<BillingSimulationDetail>('billing_generate_simulation', { input }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['billing'] });
    },
  });
}

export function useFinalizeBillingSimulation(): UseMutationResult<
  BillingSimulationDetail,
  unknown,
  FinalizeBillingSimulationInput
> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: FinalizeBillingSimulationInput) =>
      callCommand<BillingSimulationDetail>('billing_finalize_simulation', { input }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['billing'] });
    },
  });
}
