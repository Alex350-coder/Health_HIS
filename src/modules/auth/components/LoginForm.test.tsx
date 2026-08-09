import { invoke } from '@tauri-apps/api/core';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithQueryClient } from '@shared/test/render-with-query-client';

import { useSessionStore } from '../hooks/use-session-store';

import { LoginForm } from './LoginForm';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

describe('LoginForm', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
    useSessionStore.getState().clearSession();
  });

  it('shows validation errors when submitted empty', async () => {
    const user = userEvent.setup();
    renderWithQueryClient(<LoginForm />);

    await user.click(screen.getByRole('button', { name: 'Log in' }));

    expect(await screen.findByText('Username is required.')).toBeInTheDocument();
    expect(screen.getByText('Password is required.')).toBeInTheDocument();
    expect(mockedInvoke).not.toHaveBeenCalled();
  });

  it('logs in and stores the session on success', async () => {
    const user = userEvent.setup();
    const responseUser = {
      id: 1,
      fullName: 'Ada Lovelace',
      username: 'ada',
      role: 'admin',
      isActive: true,
      createdAt: '2026-01-01T00:00:00Z',
      updatedAt: null,
    };
    mockedInvoke.mockResolvedValueOnce({ token: 'session-token', user: responseUser });
    renderWithQueryClient(<LoginForm />);

    await user.type(screen.getByLabelText('Username'), 'ada');
    await user.type(screen.getByLabelText('Password'), 'super-secret-1');
    await user.click(screen.getByRole('button', { name: 'Log in' }));

    await waitFor(() => {
      expect(useSessionStore.getState().token).toBe('session-token');
    });
    expect(useSessionStore.getState().user).toEqual(responseUser);
    expect(mockedInvoke).toHaveBeenCalledWith('auth_login', {
      input: { username: 'ada', password: 'super-secret-1' },
    });
  });

  it('shows an account-locked countdown dialog on a failed login', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockRejectedValueOnce({ type: 'AccountLocked', retryAfterSecs: 60 });
    renderWithQueryClient(<LoginForm />);

    await user.type(screen.getByLabelText('Username'), 'ada');
    await user.type(screen.getByLabelText('Password'), 'wrong-password');
    await user.click(screen.getByRole('button', { name: 'Log in' }));

    expect(await screen.findByRole('dialog')).toHaveTextContent('Try again in 60 seconds.');
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
    expect(useSessionStore.getState().token).toBeNull();
  });
});
