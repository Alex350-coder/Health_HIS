import { invoke } from '@tauri-apps/api/core';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithQueryClient } from '@shared/test/render-with-query-client';

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
    renderWithQueryClient(<BootstrapForm />);

    await user.click(screen.getByRole('button', { name: 'Create administrator account' }));

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
    renderWithQueryClient(<BootstrapForm />);

    await user.type(screen.getByLabelText('Full name'), 'Ada Lovelace');
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
  });

  it('shows the server error message on a failed bootstrap', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockRejectedValueOnce({
      type: 'Conflict',
      message: 'an administrator already exists',
    });
    renderWithQueryClient(<BootstrapForm />);

    await user.type(screen.getByLabelText('Full name'), 'Ada Lovelace');
    await user.type(screen.getByLabelText('Username'), 'ada.admin');
    await user.type(screen.getByLabelText('Password'), 'Sup3r-Secret-Pass');
    await user.click(screen.getByRole('button', { name: 'Create administrator account' }));

    expect(await screen.findByRole('alert')).toHaveTextContent('an administrator already exists');
  });
});
