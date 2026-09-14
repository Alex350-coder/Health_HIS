import { invoke } from '@tauri-apps/api/core';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { ToastRegion } from '@shared/components/ToastRegion';
import { renderWithRouterAndQueryClient } from '@shared/test/render-with-router';

import { useSessionStore } from '../hooks/use-session-store';

import { BootstrapForm } from './BootstrapForm';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

describe('BootstrapForm', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
    useSessionStore.getState().clearSession();
  });

  it('shows validation errors when submitted empty', async () => {
    const user = userEvent.setup();
    renderWithRouterAndQueryClient(<BootstrapForm />);

    await user.click(await screen.findByRole('button', { name: 'Create administrator account' }));

    expect(await screen.findByText('Full name is required.')).toBeInTheDocument();
    expect(mockedInvoke).not.toHaveBeenCalled();
  });

  it('bootstraps the first admin and stores the session on success', async () => {
    const user = userEvent.setup();
    const adminUser = {
      id: 1,
      fullName: 'Ada Lovelace',
      username: 'ada.admin',
      role: 'admin',
      isActive: true,
      createdAt: '2026-01-01T00:00:00Z',
      updatedAt: null,
    };
    mockedInvoke.mockResolvedValueOnce({ token: 'bootstrap-token', user: adminUser });
    renderWithRouterAndQueryClient(<BootstrapForm />, {
      initialPath: '/setup',
      destinationPath: '/',
    });

    await user.type(await screen.findByLabelText('Full name'), 'Ada Lovelace');
    await user.type(screen.getByLabelText('Username'), 'ada.admin');
    await user.type(screen.getByLabelText('Password'), 'Sup3r-Secret-Pass');
    await user.click(screen.getByRole('button', { name: 'Create administrator account' }));

    await waitFor(() => {
      expect(useSessionStore.getState().token).toBe('bootstrap-token');
    });
    expect(mockedInvoke).toHaveBeenCalledWith('auth_bootstrap_admin', {
      input: {
        fullName: 'Ada Lovelace',
        username: 'ada.admin',
        password: 'Sup3r-Secret-Pass',
        role: 'admin',
      },
    });
    expect(await screen.findByText('destination reached')).toBeInTheDocument();
  });

  it('shows a toast with the server error message on a failed bootstrap and does not navigate', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockRejectedValueOnce({
      type: 'Conflict',
      message: 'an administrator already exists',
    });
    renderWithRouterAndQueryClient(
      <>
        <BootstrapForm />
        <ToastRegion />
      </>,
      { initialPath: '/setup', destinationPath: '/' },
    );

    await user.type(await screen.findByLabelText('Full name'), 'Ada Lovelace');
    await user.type(screen.getByLabelText('Username'), 'ada.admin');
    await user.type(screen.getByLabelText('Password'), 'Sup3r-Secret-Pass');
    await user.click(screen.getByRole('button', { name: 'Create administrator account' }));

    expect(await screen.findByText('an administrator already exists')).toBeInTheDocument();
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
    expect(screen.queryByText('destination reached')).not.toBeInTheDocument();
  });
});
