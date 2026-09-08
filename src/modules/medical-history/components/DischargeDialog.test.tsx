import { invoke } from '@tauri-apps/api/core';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithQueryClient } from '@shared/test/render-with-query-client';

import { DischargeDialog } from './DischargeDialog';

import type { ReactNode } from 'react';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

vi.mock('@tanstack/react-router', () => ({
  Link: ({ children }: { children: ReactNode }) => children,
}));

const mockedInvoke = vi.mocked(invoke);

const NOT_FOUND_ERROR = { type: 'NotFound', entity: 'billing_simulation', id: 1 };

describe('DischargeDialog', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('shows a no-simulation-yet notice while the dialog is open', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockRejectedValueOnce(NOT_FOUND_ERROR);

    renderWithQueryClient(<DischargeDialog encounterId={1} patientId={1} />);
    await user.click(screen.getByRole('button', { name: 'Discharge' }));

    expect(
      await screen.findByText(/No billing simulation has been generated yet\./),
    ).toBeInTheDocument();
  });

  it('confirms discharge with the given encounter id', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockRejectedValueOnce(NOT_FOUND_ERROR).mockResolvedValueOnce({
      id: 1,
      patientId: 1,
      status: 'discharged',
      admittedAt: '2026-01-01T00:00:00Z',
      dischargedAt: '2026-01-02T00:00:00Z',
      dischargeSummary: null,
      createdByUserId: 1,
      createdAt: '2026-01-01T00:00:00Z',
      updatedAt: null,
    });

    renderWithQueryClient(<DischargeDialog encounterId={1} patientId={1} />);
    await user.click(screen.getByRole('button', { name: 'Discharge' }));
    await user.click(await screen.findByRole('button', { name: 'Confirm discharge' }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith('medical_history_discharge_encounter', {
        input: { encounterId: 1, dischargeSummary: undefined },
      });
    });
  });
});
