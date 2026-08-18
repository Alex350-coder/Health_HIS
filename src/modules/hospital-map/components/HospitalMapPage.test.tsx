import { invoke } from '@tauri-apps/api/core';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithQueryClient } from '@shared/test/render-with-query-client';

import HospitalMapPage from './HospitalMapPage';

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
      status: 'occupied',
      createdAt: '2026-01-01T00:00:00Z',
      updatedAt: null,
    },
  ],
  availableCount: 0,
  occupiedCount: 1,
  maintenanceCount: 0,
};

describe('HospitalMapPage', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('shows an empty state when there is no facility configured yet', async () => {
    mockedInvoke.mockResolvedValueOnce([]);
    renderWithQueryClient(<HospitalMapPage />);

    expect(await screen.findByText('No facility configured yet')).toBeInTheDocument();
  });

  it('shows an error state when the layout fails to load', async () => {
    mockedInvoke.mockRejectedValueOnce({ type: 'Unexpected', message: 'boom', correlationId: 'x' });
    renderWithQueryClient(<HospitalMapPage />);

    expect(await screen.findByRole('alert')).toBeInTheDocument();
  });

  it('renders the floor map and shows a read-only detail panel when a room is selected', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockResolvedValueOnce(LAYOUT);
    renderWithQueryClient(<HospitalMapPage />);

    const roomMarker = await screen.findByRole('button', { name: 'Ward A' });

    mockedInvoke.mockResolvedValueOnce(ROOM_STATUS);
    await user.click(roomMarker);

    expect(await screen.findByRole('heading', { name: 'Ward A' })).toBeInTheDocument();
    expect(screen.getByText('Bed 1A')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /Update/ })).not.toBeInTheDocument();
    expect(screen.queryByRole('textbox')).not.toBeInTheDocument();
  });
});
