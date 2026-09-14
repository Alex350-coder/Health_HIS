import { invoke } from '@tauri-apps/api/core';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { useSessionStore } from '@modules/auth/hooks/use-session-store';

import { App } from './App';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

const USER = {
  id: 1,
  fullName: 'Ada Lovelace',
  username: 'ada',
  role: 'admin',
  isActive: true,
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: null,
};

describe('App', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
    useSessionStore.getState().clearSession();
    window.history.pushState({}, '', '/');
  });

  it('redirects to /login when there is no session and bootstrap is already done', async () => {
    mockedInvoke.mockResolvedValue({ needsBootstrap: false });

    render(<App />);

    expect(await screen.findByRole('form', { name: 'Log in' })).toBeInTheDocument();
  });

  it('redirects to /setup when the app still needs its first administrator', async () => {
    mockedInvoke.mockResolvedValue({ needsBootstrap: true });

    render(<App />);

    expect(
      await screen.findByRole('form', { name: 'Create the first administrator account' }),
    ).toBeInTheDocument();
  });

  it('renders the dashboard for an authenticated session', async () => {
    mockedInvoke.mockResolvedValue({ needsBootstrap: false });
    useSessionStore.getState().setSession('token-123', USER);

    render(<App />);

    expect(
      await screen.findByRole('heading', { name: 'Hospital Information System' }),
    ).toBeInTheDocument();
    expect(screen.getAllByText('Ada Lovelace')[0]).toBeInTheDocument();
  });

  it('navigates from /login to the authenticated dashboard on a successful login', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockImplementation((command: string) => {
      if (command === 'auth_bootstrap_status') return Promise.resolve({ needsBootstrap: false });
      if (command === 'auth_login') return Promise.resolve({ token: 'token-123', user: USER });
      if (command === 'notifications_list') return Promise.resolve([]);
      return Promise.reject(new Error(`unexpected command: ${command}`));
    });

    render(<App />);

    await user.type(await screen.findByLabelText('Username'), 'ada');
    await user.type(screen.getByLabelText('Password'), 'super-secret-1');
    await user.click(screen.getByRole('button', { name: 'Log in' }));

    expect(
      await screen.findByRole('heading', { name: 'Hospital Information System' }),
    ).toBeInTheDocument();
    expect(useSessionStore.getState().token).toBe('token-123');
  });

  it('navigates from /setup to the authenticated dashboard on a successful bootstrap', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockImplementation((command: string) => {
      if (command === 'auth_bootstrap_status') return Promise.resolve({ needsBootstrap: true });
      if (command === 'auth_bootstrap_admin') {
        return Promise.resolve({ token: 'bootstrap-token', user: USER });
      }
      if (command === 'notifications_list') return Promise.resolve([]);
      return Promise.reject(new Error(`unexpected command: ${command}`));
    });

    render(<App />);

    await user.type(await screen.findByLabelText('Full name'), 'Ada Lovelace');
    await user.type(screen.getByLabelText('Username'), 'ada');
    await user.type(screen.getByLabelText('Password'), 'Sup3r-Secret-Pass');
    await user.click(screen.getByRole('button', { name: 'Create administrator account' }));

    expect(
      await screen.findByRole('heading', { name: 'Hospital Information System' }),
    ).toBeInTheDocument();
    expect(useSessionStore.getState().token).toBe('bootstrap-token');
  });

  it('does not navigate away from /login when the login mutation fails', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockResolvedValueOnce({ needsBootstrap: false });
    mockedInvoke.mockRejectedValueOnce({
      type: 'Validation',
      field: 'password',
      message: 'invalid credentials',
    });

    render(<App />);

    await user.type(await screen.findByLabelText('Username'), 'ada');
    await user.type(screen.getByLabelText('Password'), 'wrong-password');
    await user.click(screen.getByRole('button', { name: 'Log in' }));

    expect(await screen.findByRole('alert')).toHaveTextContent('invalid credentials');
    expect(screen.getByRole('form', { name: 'Log in' })).toBeInTheDocument();
    expect(useSessionStore.getState().token).toBeNull();
  });
});
