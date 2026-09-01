import { invoke } from '@tauri-apps/api/core';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithQueryClient } from '@shared/test/render-with-query-client';

import BillingTab from './BillingTab';

import type { BillingSimulationDetail } from '../types/billing-schemas';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

vi.mock('@tanstack/react-router', () => ({
  useParams: () => ({ patientId: 1 }),
}));

const mockedInvoke = vi.mocked(invoke);

function historyWith(encounters: { id: number; status: string; admittedAt: string }[]): unknown {
  return { encounters, diagnoses: [], treatments: [], evolutions: [] };
}

/** `invoke` rejects with a plain `AppError`-shaped object, not an `Error` (see `app-error.ts`). */
const notFoundRejection = (): Promise<never> =>
  // eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors
  Promise.reject({ type: 'NotFound', entity: 'billing_simulation', id: 42 });

const DRAFT_DETAIL: BillingSimulationDetail = {
  simulation: {
    id: 1,
    encounterId: 42,
    status: 'draft',
    totalAmount: 1255,
    generatedByUserId: 1,
    createdAt: '2026-01-01T00:00:00Z',
    updatedAt: null,
  },
  items: [
    {
      id: 1,
      billingSimulationId: 1,
      description: 'Room charge',
      source: 'room',
      sourceEntityId: 1,
      amount: 150,
      createdAt: '2026-01-01T00:00:00Z',
    },
  ],
};

describe('BillingTab', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('shows an empty state when the patient has no encounters at all', async () => {
    mockedInvoke.mockImplementation((command: string) => {
      if (command === 'medical_history_get_by_patient') return Promise.resolve(historyWith([]));
      return Promise.reject(new Error(`unexpected command ${command}`));
    });

    renderWithQueryClient(<BillingTab />);

    expect(await screen.findByText('No encounters yet')).toBeInTheDocument();
  });

  it('offers to generate a simulation when none exists yet for the open encounter', async () => {
    mockedInvoke.mockImplementation((command: string) => {
      if (command === 'medical_history_get_by_patient') {
        return Promise.resolve(
          historyWith([{ id: 42, status: 'open', admittedAt: '2026-01-01T00:00:00Z' }]),
        );
      }
      if (command === 'billing_get_simulation') {
        return notFoundRejection();
      }
      return Promise.reject(new Error(`unexpected command ${command}`));
    });

    renderWithQueryClient(<BillingTab />);

    expect(await screen.findByText('No billing simulation yet')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Generate Simulation' })).toBeInTheDocument();
    expect(screen.getByText('Simulation Only — Not a real invoice.')).toBeInTheDocument();
  });

  it('generates a simulation for the target encounter', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockImplementation((command: string) => {
      if (command === 'medical_history_get_by_patient') {
        return Promise.resolve(
          historyWith([{ id: 42, status: 'open', admittedAt: '2026-01-01T00:00:00Z' }]),
        );
      }
      if (command === 'billing_get_simulation') {
        return notFoundRejection();
      }
      if (command === 'billing_generate_simulation') return Promise.resolve(DRAFT_DETAIL);
      return Promise.reject(new Error(`unexpected command ${command}`));
    });

    renderWithQueryClient(<BillingTab />);

    await user.click(await screen.findByRole('button', { name: 'Generate Simulation' }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith('billing_generate_simulation', {
        input: { encounterId: 42 },
      });
    });
  });

  it('shows the itemized draft simulation with a Finalize button', async () => {
    mockedInvoke.mockImplementation((command: string) => {
      if (command === 'medical_history_get_by_patient') {
        return Promise.resolve(
          historyWith([{ id: 42, status: 'open', admittedAt: '2026-01-01T00:00:00Z' }]),
        );
      }
      if (command === 'billing_get_simulation') return Promise.resolve(DRAFT_DETAIL);
      return Promise.reject(new Error(`unexpected command ${command}`));
    });

    renderWithQueryClient(<BillingTab />);

    expect(await screen.findByText('Room charge')).toBeInTheDocument();
    expect(screen.getByText('Draft')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Finalize' })).toBeInTheDocument();
  });

  it('finalizes the simulation and shows the Finalized badge', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockImplementation((command: string) => {
      if (command === 'medical_history_get_by_patient') {
        return Promise.resolve(
          historyWith([{ id: 42, status: 'open', admittedAt: '2026-01-01T00:00:00Z' }]),
        );
      }
      if (command === 'billing_get_simulation') return Promise.resolve(DRAFT_DETAIL);
      if (command === 'billing_finalize_simulation') {
        return Promise.resolve({
          ...DRAFT_DETAIL,
          simulation: { ...DRAFT_DETAIL.simulation, status: 'finalized' },
        });
      }
      return Promise.reject(new Error(`unexpected command ${command}`));
    });

    renderWithQueryClient(<BillingTab />);

    await user.click(await screen.findByRole('button', { name: 'Finalize' }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith('billing_finalize_simulation', {
        input: { id: 1 },
      });
    });
  });
});
