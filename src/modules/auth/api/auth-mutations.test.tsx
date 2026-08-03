import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { renderHook, act } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import {
  useBootstrapAdmin,
  useCreateUser,
  useDeactivateUser,
  useLogin,
  useLogout,
} from './auth-mutations';

import type { ReactNode } from 'react';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

function wrapper({ children }: { children: ReactNode }): JSX.Element {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

const INPUT = {
  fullName: 'Ada',
  username: 'ada',
  password: 'Sup3r-Secret-Pass',
  role: 'admin' as const,
};

describe('auth-mutations', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('useBootstrapAdmin calls auth_bootstrap_admin with the input', async () => {
    mockedInvoke.mockResolvedValueOnce({ token: 't', user: INPUT });
    const { result } = renderHook(() => useBootstrapAdmin(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync(INPUT);
    });

    expect(mockedInvoke).toHaveBeenCalledWith('auth_bootstrap_admin', { input: INPUT });
  });

  it('useLogin calls auth_login with the input', async () => {
    mockedInvoke.mockResolvedValueOnce({ token: 't', user: INPUT });
    const { result } = renderHook(() => useLogin(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync({ username: 'ada', password: 'x' });
    });

    expect(mockedInvoke).toHaveBeenCalledWith('auth_login', {
      input: { username: 'ada', password: 'x' },
    });
  });

  it('useLogout calls auth_logout', async () => {
    mockedInvoke.mockResolvedValueOnce(undefined);
    const { result } = renderHook(() => useLogout(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync();
    });

    expect(mockedInvoke).toHaveBeenCalledWith('auth_logout', undefined);
  });

  it('useCreateUser invalidates the user list on success', async () => {
    mockedInvoke.mockResolvedValueOnce(INPUT);
    mockedInvoke.mockResolvedValueOnce([]);
    const { result: mutation } = renderHook(() => useCreateUser(), { wrapper });

    await act(async () => {
      await mutation.current.mutateAsync(INPUT);
    });

    expect(mockedInvoke).toHaveBeenCalledWith('auth_create_user', { input: INPUT });
  });

  it('useDeactivateUser calls auth_deactivate_user with the id', async () => {
    mockedInvoke.mockResolvedValueOnce({ ...INPUT, isActive: false });
    const { result } = renderHook(() => useDeactivateUser(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync(1);
    });

    expect(mockedInvoke).toHaveBeenCalledWith('auth_deactivate_user', { input: { id: 1 } });
  });
});
