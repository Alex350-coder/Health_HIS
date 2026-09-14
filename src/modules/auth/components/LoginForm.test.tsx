import { invoke } from '@tauri-apps/api/core';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithRouterAndQueryClient } from '@shared/test/render-with-router';

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
    renderWithRouterAndQueryClient(<LoginForm />);

    await user.click(await screen.findByRole('button', { name: 'Log in' }));

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
    renderWithRouterAndQueryClient(<LoginForm />, { initialPath: '/login', destinationPath: '/' });

    await user.type(await screen.findByLabelText('Username'), 'ada');
    await user.type(screen.getByLabelText('Password'), 'super-secret-1');
    await user.click(screen.getByRole('button', { name: 'Log in' }));

    await waitFor(() => {
      expect(useSessionStore.getState().token).toBe('session-token');
    });
    expect(useSessionStore.getState().user).toEqual(responseUser);
    expect(mockedInvoke).toHaveBeenCalledWith('auth_login', {
      input: { username: 'ada', password: 'super-secret-1' },
    });
    expect(await screen.findByText('destination reached')).toBeInTheDocument();
  });

  it('shows an account-locked countdown dialog on a failed login and does not navigate', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockRejectedValueOnce({ type: 'AccountLocked', retryAfterSecs: 60 });
    renderWithRouterAndQueryClient(<LoginForm />, { initialPath: '/login', destinationPath: '/' });

    await user.type(await screen.findByLabelText('Username'), 'ada');
    await user.type(screen.getByLabelText('Password'), 'wrong-password');
    await user.click(screen.getByRole('button', { name: 'Log in' }));

    expect(await screen.findByRole('dialog')).toHaveTextContent('Try again in 60 seconds.');
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
    expect(useSessionStore.getState().token).toBeNull();
    expect(screen.queryByText('destination reached')).not.toBeInTheDocument();
  });
});
