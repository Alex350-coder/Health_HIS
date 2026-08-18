import { invoke } from '@tauri-apps/api/core';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithQueryClient } from '@shared/test/render-with-query-client';

import { FacilityStructureView } from './FacilityStructureView';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

const LAYOUT = [
  {
    id: 1,
    name: 'Ground Floor',
    levelOrder: 0,
    createdAt: '2026-01-01T00:00:00Z',
    rooms: [
      {
        id: 1,
        floorId: 1,
        name: 'Ward A',
        roomType: 'ward',
        mapX: 0.5,
        mapY: 0.5,
        createdAt: '2026-01-01T00:00:00Z',
      },
    ],
  },
];

const ROOM_STATUS = {
  room: LAYOUT[0]?.rooms[0],
  beds: [
    {
      id: 1,
      roomId: 1,
      label: 'Bed 1A',
      status: 'available',
      createdAt: '2026-01-01T00:00:00Z',
      updatedAt: null,
    },
  ],
  availableCount: 1,
  occupiedCount: 0,
  maintenanceCount: 0,
};

describe('FacilityStructureView', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('shows an empty state when there is no facility configured yet', async () => {
    mockedInvoke.mockResolvedValueOnce([]);
    renderWithQueryClient(<FacilityStructureView />);

    expect(await screen.findByText('No facility configured yet')).toBeInTheDocument();
  });

  it('shows an error state when the layout fails to load', async () => {
    mockedInvoke.mockRejectedValueOnce({ type: 'Unexpected', message: 'boom', correlationId: 'x' });
    renderWithQueryClient(<FacilityStructureView />);

    expect(await screen.findByRole('alert')).toBeInTheDocument();
  });

  it('renders floors and rooms, and reveals beds with a status form when a room is expanded', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockResolvedValueOnce(LAYOUT);
    renderWithQueryClient(<FacilityStructureView />);

    expect(await screen.findByText('Ground Floor')).toBeInTheDocument();

    mockedInvoke.mockResolvedValueOnce(ROOM_STATUS);
    await user.click(screen.getByRole('button', { name: /Show beds — Ward A/ }));

    expect(await screen.findByText('Bed 1A')).toBeInTheDocument();
    expect(screen.getByRole('form', { name: 'Set status for bed 1' })).toBeInTheDocument();
  });
});
