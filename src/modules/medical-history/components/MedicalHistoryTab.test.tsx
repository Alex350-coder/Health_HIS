import { invoke } from '@tauri-apps/api/core';
import { screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithQueryClient } from '@shared/test/render-with-query-client';

import MedicalHistoryTab from './MedicalHistoryTab';

import type { MedicalHistoryBundle } from '../types/medical-history-schemas';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

vi.mock('@tanstack/react-router', () => ({
  useParams: () => ({ patientId: 1 }),
}));

const mockedInvoke = vi.mocked(invoke);

const EMPTY_BUNDLE: MedicalHistoryBundle = {
  encounters: [],
  diagnoses: [],
  treatments: [],
  evolutions: [],
};

const OPEN_ENCOUNTER_BUNDLE: MedicalHistoryBundle = {
  encounters: [
    {
      id: 1,
      patientId: 1,
      status: 'open',
      admittedAt: '2026-01-01T00:00:00Z',
      dischargedAt: null,
      dischargeSummary: null,
      createdByUserId: 1,
      createdAt: '2026-01-01T00:00:00Z',
      updatedAt: null,
    },
  ],
  diagnoses: [],
  treatments: [],
  evolutions: [],
};

describe('MedicalHistoryTab', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('shows a start-encounter action when there is no open encounter', async () => {
    mockedInvoke.mockResolvedValueOnce(EMPTY_BUNDLE);
    renderWithQueryClient(<MedicalHistoryTab />);

    expect(await screen.findByText('No open encounter')).toBeInTheDocument();
  });

  it('shows the open encounter panel when one exists', async () => {
    mockedInvoke.mockResolvedValueOnce(OPEN_ENCOUNTER_BUNDLE);
    renderWithQueryClient(<MedicalHistoryTab />);

    expect(await screen.findByText(/Open encounter/)).toBeInTheDocument();
  });

  it('shows an error state when the bundle fails to load', async () => {
    mockedInvoke.mockRejectedValueOnce({ type: 'Unexpected', message: 'boom', correlationId: 'x' });
    renderWithQueryClient(<MedicalHistoryTab />);

    expect(await screen.findByRole('alert')).toBeInTheDocument();
  });
});
