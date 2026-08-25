import { invoke } from '@tauri-apps/api/core';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithQueryClient } from '@shared/test/render-with-query-client';

import OrSchedulePage from './OrSchedulePage';

import type { OperatingRoom, OrReservation } from '../types/or-schemas';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

const OPERATING_ROOM: OperatingRoom = {
  id: 1,
  roomId: 1,
  name: 'OR Suite',
  createdAt: '2026-01-01T00:00:00Z',
};

const RESERVATION: OrReservation = {
  id: 1,
  operatingRoomId: 1,
  patientId: 1,
  encounterId: 1,
  procedureDescription: 'Appendectomy',
  scheduledStart: '2026-01-01T08:00:00',
  scheduledEnd: '2026-01-01T10:00:00',
  status: 'scheduled',
  scheduledByUserId: 1,
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: null,
};

describe('OrSchedulePage', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('shows an empty state when there are no operating rooms', async () => {
    mockedInvoke.mockImplementation((command: string) => {
      if (command === 'operating_rooms_list') return Promise.resolve([]);
      return Promise.reject(new Error(`unexpected command ${command}`));
    });

    renderWithQueryClient(<OrSchedulePage />);

    expect(await screen.findByText('No operating rooms yet')).toBeInTheDocument();
  });

  it('lists reservations for the selected operating room and cancels one', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockImplementation((command: string) => {
      if (command === 'operating_rooms_list') return Promise.resolve([OPERATING_ROOM]);
      if (command === 'operating_rooms_list_reservations') return Promise.resolve([RESERVATION]);
      if (command === 'operating_rooms_cancel_reservation') {
        return Promise.resolve({ ...RESERVATION, status: 'cancelled' });
      }
      return Promise.reject(new Error(`unexpected command ${command}`));
    });

    renderWithQueryClient(<OrSchedulePage />);

    await user.selectOptions(await screen.findByLabelText('Operating room'), 'OR Suite');
    expect(await screen.findByText('Appendectomy')).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Cancel' }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith('operating_rooms_cancel_reservation', {
        input: { id: 1 },
      });
    });
  });
});
