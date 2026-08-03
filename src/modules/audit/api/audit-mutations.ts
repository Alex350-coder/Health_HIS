import { useMutation, type UseMutationResult } from '@tanstack/react-query';

import { callCommand } from '@shared/lib/api-client';

import type { ChainVerification } from '../types/audit-types';

export function useVerifyIntegrity(): UseMutationResult<ChainVerification, unknown, void> {
  return useMutation({
    mutationFn: () => callCommand<ChainVerification>('audit_verify_integrity'),
  });
}
