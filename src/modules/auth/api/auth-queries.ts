import { useQuery, type UseQueryResult } from '@tanstack/react-query';

import { callCommand } from '@shared/lib/api-client';

import type { User } from '../types/auth-schemas';

interface BootstrapStatusResponse {
  needsBootstrap: boolean;
}

interface CurrentUserResponse {
  user: User;
}

export const bootstrapStatusQueryKey = ['auth', 'bootstrapStatus'] as const;

export function fetchBootstrapStatus(): Promise<BootstrapStatusResponse> {
  return callCommand<BootstrapStatusResponse>('auth_bootstrap_status');
}

export function useBootstrapStatus(): UseQueryResult<BootstrapStatusResponse> {
  return useQuery({
    queryKey: bootstrapStatusQueryKey,
    queryFn: fetchBootstrapStatus,
  });
}

export function useCurrentUser(enabled: boolean): UseQueryResult<CurrentUserResponse> {
  return useQuery({
    queryKey: ['auth', 'currentUser'],
    queryFn: () => callCommand<CurrentUserResponse>('auth_current_user'),
    enabled,
    retry: false,
  });
}

export function useListUsers(enabled: boolean): UseQueryResult<User[]> {
  return useQuery({
    queryKey: ['auth', 'users'],
    queryFn: () => callCommand<User[]>('auth_list_users'),
    enabled,
  });
}
