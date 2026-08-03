import { invoke } from '@tauri-apps/api/core';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithQueryClient } from '@shared/test/render-with-query-client';

import { UserList } from './UserList';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

const ACTIVE_USER = {
  id: 1,
  fullName: 'Ada Lovelace',
  username: 'ada',
  role: 'admin',
  isActive: true,
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: null,
};

describe('UserList', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('shows a loading state, then an empty state when there are no users', async () => {
    mockedInvoke.mockResolvedValueOnce([]);
    renderWithQueryClient(<UserList />);

    expect(screen.getByText('Loading users…')).toBeInTheDocument();
    expect(await screen.findByText('No users yet.')).toBeInTheDocument();
  });

  it('renders each user row with their role and status', async () => {
    mockedInvoke.mockResolvedValueOnce([ACTIVE_USER]);
    renderWithQueryClient(<UserList />);

    expect(await screen.findByText(/Ada Lovelace \(ada\) — admin — active/)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Deactivate' })).toBeInTheDocument();
  });

  it('shows an error message when the list fails to load', async () => {
    mockedInvoke.mockRejectedValueOnce({ type: 'Unexpected', message: 'boom', correlationId: 'x' });
    renderWithQueryClient(<UserList />);

    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Something went wrong. (Error reference: x)',
    );
  });

  it('creates a new user through the embedded form', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockResolvedValueOnce([]);
    mockedInvoke.mockResolvedValueOnce({
      ...ACTIVE_USER,
      id: 2,
      username: 'nurse.jane',
      role: 'nurse',
    });
    mockedInvoke.mockResolvedValueOnce([
      ACTIVE_USER,
      { ...ACTIVE_USER, id: 2, username: 'nurse.jane' },
    ]);
    renderWithQueryClient(<UserList />);
    await screen.findByText('No users yet.');

    await user.type(screen.getByLabelText('Full name'), 'Jane Nurse');
    await user.type(screen.getByLabelText('Username'), 'nurse.jane');
    await user.type(screen.getByLabelText('Password'), 'Sup3r-Secret-Pass');
    await user.selectOptions(screen.getByLabelText('Role'), 'nurse');
    await user.click(screen.getByRole('button', { name: 'Create user' }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith('auth_create_user', {
        input: {
          fullName: 'Jane Nurse',
          username: 'nurse.jane',
          password: 'Sup3r-Secret-Pass',
          role: 'nurse',
        },
      });
    });
  });

  it('deactivates a user', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockResolvedValueOnce([ACTIVE_USER]);
    mockedInvoke.mockResolvedValueOnce({ ...ACTIVE_USER, isActive: false });
    renderWithQueryClient(<UserList />);
    await screen.findByRole('button', { name: 'Deactivate' });

    await user.click(screen.getByRole('button', { name: 'Deactivate' }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith('auth_deactivate_user', { input: { id: 1 } });
    });
  });
});
