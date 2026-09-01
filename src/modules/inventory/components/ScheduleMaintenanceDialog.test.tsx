import { invoke } from '@tauri-apps/api/core';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithQueryClient } from '@shared/test/render-with-query-client';

import { ScheduleMaintenanceDialog } from './ScheduleMaintenanceDialog';

import type { MaintenanceSchedule } from '../types/inventory-schemas';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

const SCHEDULE: MaintenanceSchedule = {
  id: 1,
  inventoryItemId: 1,
  scheduledDate: '2026-03-01',
  completedDate: null,
  notes: null,
  createdAt: '2026-01-01T00:00:00Z',
};

describe('ScheduleMaintenanceDialog', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('submits a maintenance schedule for the given item', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockResolvedValueOnce(SCHEDULE);

    renderWithQueryClient(<ScheduleMaintenanceDialog inventoryItemId={1} />);

    await user.click(screen.getByRole('button', { name: 'Schedule maintenance' }));
    await user.type(await screen.findByLabelText('Scheduled date'), '2026-03-01');
    await user.click(screen.getByRole('button', { name: 'Schedule' }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith('inventory_schedule_maintenance', {
        input: {
          inventoryItemId: 1,
          scheduledDate: '2026-03-01',
          notes: '',
        },
      });
    });
  });
});
