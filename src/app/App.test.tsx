import { invoke } from '@tauri-apps/api/core';
import { render, screen } from '@testing-library/react';
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
    expect(screen.getByText('Ada Lovelace')).toBeInTheDocument();
  });
});
