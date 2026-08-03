import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import {
  fetchBootstrapStatus,
  useBootstrapStatus,
  useCurrentUser,
  useListUsers,
} from './auth-queries';

import type { ReactNode } from 'react';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

function wrapper({ children }: { children: ReactNode }): JSX.Element {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

describe('auth-queries', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('fetchBootstrapStatus calls the auth_bootstrap_status command', async () => {
    mockedInvoke.mockResolvedValueOnce({ needsBootstrap: true });

    await expect(fetchBootstrapStatus()).resolves.toEqual({ needsBootstrap: true });
    expect(mockedInvoke).toHaveBeenCalledWith('auth_bootstrap_status', undefined);
  });

  it('useBootstrapStatus resolves with the bootstrap status', async () => {
    mockedInvoke.mockResolvedValueOnce({ needsBootstrap: false });

    const { result } = renderHook(() => useBootstrapStatus(), { wrapper });

    await waitFor(() => {
      expect(result.current.data).toEqual({ needsBootstrap: false });
    });
  });

  it('useCurrentUser does not fetch when disabled', () => {
    renderHook(() => useCurrentUser(false), { wrapper });

    expect(mockedInvoke).not.toHaveBeenCalled();
  });

  it('useCurrentUser fetches the current user when enabled', async () => {
    const user = {
      id: 1,
      fullName: 'Ada Lovelace',
      username: 'ada',
      role: 'admin',
      isActive: true,
      createdAt: '2026-01-01T00:00:00Z',
      updatedAt: null,
    };
    mockedInvoke.mockResolvedValueOnce({ user });

    const { result } = renderHook(() => useCurrentUser(true), { wrapper });

    await waitFor(() => {
      expect(result.current.data).toEqual({ user });
    });
    expect(mockedInvoke).toHaveBeenCalledWith('auth_current_user', undefined);
  });

  it('useListUsers fetches the user list when enabled', async () => {
    mockedInvoke.mockResolvedValueOnce([]);

    const { result } = renderHook(() => useListUsers(true), { wrapper });

    await waitFor(() => {
      expect(result.current.data).toEqual([]);
    });
  });
});
