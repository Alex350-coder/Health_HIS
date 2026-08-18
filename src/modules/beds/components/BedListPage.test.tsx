import { invoke } from '@tauri-apps/api/core';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithQueryClient } from '@shared/test/render-with-query-client';

import BedListPage from './BedListPage';

import type { BedSummary } from '../types/bed-schemas';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

const OCCUPIED_BED: BedSummary = {
  id: 1,
  roomId: 1,
  label: 'Bed 1A',
  status: 'occupied',
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: null,
  activeAssignment: { id: 5, patientId: 1, encounterId: 1 },
};

const AVAILABLE_BED: BedSummary = {
  id: 2,
  roomId: 1,
  label: 'Bed 1B',
  status: 'available',
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: null,
  activeAssignment: null,
};

describe('BedListPage', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('lists beds with a Release action only for occupied beds', async () => {
    mockedInvoke.mockResolvedValueOnce([OCCUPIED_BED, AVAILABLE_BED]);
    renderWithQueryClient(<BedListPage />);

    expect(await screen.findByText('Bed 1A')).toBeInTheDocument();
    expect(screen.getByText('Bed 1B')).toBeInTheDocument();
    expect(screen.getAllByRole('button', { name: 'Release' })).toHaveLength(1);
  });

  it('releases the bed when Release is clicked', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockResolvedValueOnce([OCCUPIED_BED]);
    mockedInvoke.mockResolvedValueOnce({
      id: 5,
      bedId: 1,
      patientId: 1,
      encounterId: 1,
      assignedAt: '2026-01-01T00:00:00Z',
      releasedAt: '2026-01-02T00:00:00Z',
    });
    renderWithQueryClient(<BedListPage />);

    await user.click(await screen.findByRole('button', { name: 'Release' }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith('beds_release', {
        input: { bedAssignmentId: 5 },
      });
    });
  });
});
