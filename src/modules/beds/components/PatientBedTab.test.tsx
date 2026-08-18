import { invoke } from '@tauri-apps/api/core';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithQueryClient } from '@shared/test/render-with-query-client';

import PatientBedTab from './PatientBedTab';

import type { BedSummary } from '../types/bed-schemas';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

vi.mock('@tanstack/react-router', () => ({
  useParams: () => ({ patientId: 1 }),
}));

const mockedInvoke = vi.mocked(invoke);

const AVAILABLE_BED: BedSummary = {
  id: 1,
  roomId: 1,
  label: 'Bed 1A',
  status: 'available',
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: null,
  activeAssignment: null,
};

const OCCUPIED_BED: BedSummary = {
  id: 2,
  roomId: 1,
  label: 'Bed 1B',
  status: 'occupied',
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: null,
  activeAssignment: { id: 9, patientId: 1, encounterId: 42 },
};

function historyWith(encounters: { id: number; status: string }[]): unknown {
  return { encounters, diagnoses: [], treatments: [], evolutions: [] };
}

describe('PatientBedTab', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('shows an empty state when the patient has no open encounter', async () => {
    mockedInvoke.mockImplementation((command: string) => {
      if (command === 'medical_history_get_by_patient') {
        return Promise.resolve(historyWith([{ id: 1, status: 'closed' }]));
      }
      if (command === 'beds_list') return Promise.resolve([AVAILABLE_BED]);
      return Promise.reject(new Error(`unexpected command ${command}`));
    });

    renderWithQueryClient(<PatientBedTab />);

    expect(await screen.findByText('No open encounter')).toBeInTheDocument();
  });

  it('offers the assign-bed dialog when there is an open encounter with no bed', async () => {
    mockedInvoke.mockImplementation((command: string) => {
      if (command === 'medical_history_get_by_patient') {
        return Promise.resolve(historyWith([{ id: 42, status: 'open' }]));
      }
      if (command === 'beds_list') return Promise.resolve([AVAILABLE_BED]);
      return Promise.reject(new Error(`unexpected command ${command}`));
    });

    renderWithQueryClient(<PatientBedTab />);

    expect(await screen.findByRole('button', { name: 'Assign bed' })).toBeInTheDocument();
  });

  it('submits the assign-bed form for the selected bed', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockImplementation((command: string) => {
      if (command === 'medical_history_get_by_patient') {
        return Promise.resolve(historyWith([{ id: 42, status: 'open' }]));
      }
      if (command === 'beds_list') return Promise.resolve([AVAILABLE_BED]);
      if (command === 'beds_assign') {
        return Promise.resolve({
          id: 9,
          bedId: 1,
          patientId: 1,
          encounterId: 42,
          assignedAt: '2026-01-01T00:00:00Z',
          releasedAt: null,
        });
      }
      return Promise.reject(new Error(`unexpected command ${command}`));
    });

    renderWithQueryClient(<PatientBedTab />);

    await user.click(await screen.findByRole('button', { name: 'Assign bed' }));
    await user.selectOptions(await screen.findByLabelText('Bed'), 'Bed 1A');
    await user.click(screen.getByRole('button', { name: 'Assign' }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith('beds_assign', {
        input: { patientId: 1, encounterId: 42, bedId: 1 },
      });
    });
  });

  it('shows the current bed and releases it', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockImplementation((command: string) => {
      if (command === 'medical_history_get_by_patient') {
        return Promise.resolve(historyWith([{ id: 42, status: 'open' }]));
      }
      if (command === 'beds_list') return Promise.resolve([OCCUPIED_BED]);
      if (command === 'beds_release') {
        return Promise.resolve({
          id: 9,
          bedId: 2,
          patientId: 1,
          encounterId: 42,
          assignedAt: '2026-01-01T00:00:00Z',
          releasedAt: '2026-01-02T00:00:00Z',
        });
      }
      return Promise.reject(new Error(`unexpected command ${command}`));
    });

    renderWithQueryClient(<PatientBedTab />);

    expect(await screen.findByText('Bed 1B')).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Release' }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith('beds_release', {
        input: { bedAssignmentId: 9 },
      });
    });
  });
});
