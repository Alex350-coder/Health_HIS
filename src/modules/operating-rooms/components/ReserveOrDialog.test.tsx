import { invoke } from '@tauri-apps/api/core';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithQueryClient } from '@shared/test/render-with-query-client';

import { ReserveOrDialog } from './ReserveOrDialog';

import type { OrReservation } from '../types/or-schemas';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

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

describe('ReserveOrDialog', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('submits the reservation form for the given operating room', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockResolvedValueOnce(RESERVATION);

    renderWithQueryClient(<ReserveOrDialog operatingRoomId={1} />);

    await user.click(screen.getByRole('button', { name: 'New reservation' }));
    await user.type(await screen.findByLabelText('Patient ID'), '1');
    await user.type(screen.getByLabelText('Encounter ID'), '1');
    await user.type(screen.getByLabelText('Procedure description'), 'Appendectomy');
    await user.type(screen.getByLabelText('Start time'), '2026-01-01T08:00');
    await user.type(screen.getByLabelText('End time'), '2026-01-01T10:00');
    await user.click(screen.getByRole('button', { name: 'Reserve' }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith('operating_rooms_reserve', {
        input: {
          operatingRoomId: 1,
          patientId: 1,
          encounterId: 1,
          procedureDescription: 'Appendectomy',
          scheduledStart: '2026-01-01T08:00',
          scheduledEnd: '2026-01-01T10:00',
        },
      });
    });
  });
});
