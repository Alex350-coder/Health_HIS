import { invoke } from '@tauri-apps/api/core';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithQueryClient } from '@shared/test/render-with-query-client';

import { PatientSummaryCard } from './PatientSummaryCard';

import type { Patient } from '../types/patient-schemas';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

const PATIENT: Patient = {
  id: 1,
  medicalRecordNumber: 'MRN-0001',
  fullName: 'Ada Lovelace',
  dateOfBirth: '1990-01-01',
  sex: 'female',
  nationalId: null,
  phone: null,
  address: null,
  emergencyContactName: null,
  emergencyContactPhone: null,
  bloodType: null,
  allergies: null,
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: null,
};

describe('PatientSummaryCard', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('renders the patient fields and an Edit button', () => {
    renderWithQueryClient(<PatientSummaryCard patient={PATIENT} />);

    expect(screen.getByText('Ada Lovelace')).toBeInTheDocument();
    expect(screen.getByText('MRN-0001')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Edit' })).toBeInTheDocument();
  });

  it('switches to the edit form, pre-filled, when Edit is clicked', async () => {
    const user = userEvent.setup();
    renderWithQueryClient(<PatientSummaryCard patient={PATIENT} />);

    await user.click(screen.getByRole('button', { name: 'Edit' }));

    expect(screen.getByRole('form', { name: 'Edit patient' })).toBeInTheDocument();
    expect(screen.getByLabelText('Full name')).toHaveValue('Ada Lovelace');
  });

  it('returns to the display view when Cancel is clicked', async () => {
    const user = userEvent.setup();
    renderWithQueryClient(<PatientSummaryCard patient={PATIENT} />);

    await user.click(screen.getByRole('button', { name: 'Edit' }));
    await user.click(screen.getByRole('button', { name: 'Cancel' }));

    expect(screen.getByRole('button', { name: 'Edit' })).toBeInTheDocument();
    expect(screen.queryByRole('form', { name: 'Edit patient' })).not.toBeInTheDocument();
  });

  it('returns to the display view after a successful save', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockResolvedValueOnce({ ...PATIENT, fullName: 'Ada Byron' });
    renderWithQueryClient(<PatientSummaryCard patient={PATIENT} />);

    await user.click(screen.getByRole('button', { name: 'Edit' }));
    await user.click(screen.getByRole('button', { name: 'Save changes' }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith('patients_update', {
        input: {
          id: 1,
          medicalRecordNumber: 'MRN-0001',
          fullName: 'Ada Lovelace',
          dateOfBirth: '1990-01-01',
          sex: 'female',
        },
      });
    });
    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Edit' })).toBeInTheDocument();
    });
  });
});
