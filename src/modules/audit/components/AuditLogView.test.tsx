import { invoke } from '@tauri-apps/api/core';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithQueryClient } from '@shared/test/render-with-query-client';

import { AuditLogView } from './AuditLogView';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

const ENTRY = {
  id: 1,
  timestamp: '2026-01-01T00:00:00Z',
  userId: 1,
  action: 'auth.login',
  entityType: 'user',
  entityId: 1,
  beforeState: null,
  afterState: null,
  result: 'success',
  prevHash: 'GENESIS',
  rowHash: 'abc123',
};

describe('AuditLogView', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('shows a loading state, then an empty state when there are no entries', async () => {
    mockedInvoke.mockResolvedValueOnce([]);
    renderWithQueryClient(<AuditLogView />);

    expect(screen.getByText('Loading audit entries…')).toBeInTheDocument();
    expect(await screen.findByText('No audit entries yet.')).toBeInTheDocument();
  });

  it('renders each audit entry as a table row', async () => {
    mockedInvoke.mockResolvedValueOnce([ENTRY]);
    renderWithQueryClient(<AuditLogView />);

    expect(await screen.findByText('auth.login')).toBeInTheDocument();
    expect(screen.getByText('success')).toBeInTheDocument();
    expect(screen.getByText('user #1')).toBeInTheDocument();
  });

  it('shows an error message when the list fails to load', async () => {
    mockedInvoke.mockRejectedValueOnce({ type: 'Unauthorized' });
    renderWithQueryClient(<AuditLogView />);

    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Your session has expired. Please log in again.',
    );
  });

  it('verifies chain integrity and reports the result', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockResolvedValueOnce([]);
    mockedInvoke.mockResolvedValueOnce({ isValid: true, brokenAtId: null });
    renderWithQueryClient(<AuditLogView />);
    await screen.findByText('No audit entries yet.');

    await user.click(screen.getByRole('button', { name: 'Verify integrity' }));

    await waitFor(() => {
      expect(screen.getByRole('status')).toHaveTextContent('The audit trail is intact.');
    });
    expect(mockedInvoke).toHaveBeenCalledWith('audit_verify_integrity', undefined);
  });

  it('reports tampering when the chain is broken', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockResolvedValueOnce([]);
    mockedInvoke.mockResolvedValueOnce({ isValid: false, brokenAtId: 7 });
    renderWithQueryClient(<AuditLogView />);
    await screen.findByText('No audit entries yet.');

    await user.click(screen.getByRole('button', { name: 'Verify integrity' }));

    expect(await screen.findByText('Tampering detected at audit entry 7.')).toBeInTheDocument();
  });
});
