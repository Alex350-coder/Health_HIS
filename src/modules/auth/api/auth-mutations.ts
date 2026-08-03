import { useMutation, useQueryClient, type UseMutationResult } from '@tanstack/react-query';

import { callCommand } from '@shared/lib/api-client';

import type { CreateUserInput, LoginInput, SessionResponse, User } from '../types/auth-schemas';

export function useBootstrapAdmin(): UseMutationResult<SessionResponse, unknown, CreateUserInput> {
  return useMutation({
    mutationFn: (input: CreateUserInput) =>
      callCommand<SessionResponse>('auth_bootstrap_admin', { input }),
  });
}

export function useLogin(): UseMutationResult<SessionResponse, unknown, LoginInput> {
  return useMutation({
    mutationFn: (input: LoginInput) => callCommand<SessionResponse>('auth_login', { input }),
  });
}

export function useLogout(): UseMutationResult<void, unknown, void> {
  return useMutation({
    mutationFn: () => callCommand<void>('auth_logout'),
  });
}

export function useCreateUser(): UseMutationResult<User, unknown, CreateUserInput> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateUserInput) => callCommand<User>('auth_create_user', { input }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['auth', 'users'] });
    },
  });
}

export function useDeactivateUser(): UseMutationResult<User, unknown, number> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => callCommand<User>('auth_deactivate_user', { input: { id } }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['auth', 'users'] });
    },
  });
}
