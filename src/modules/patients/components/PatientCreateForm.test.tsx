import { invoke } from '@tauri-apps/api/core';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithQueryClient } from '@shared/test/render-with-query-client';

import { PatientCreateForm } from './PatientCreateForm';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const navigateMock = vi.fn();
vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => navigateMock,
}));

const mockedInvoke = vi.mocked(invoke);

describe('PatientCreateForm', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
    navigateMock.mockReset();
  });

  it('shows validation errors when submitted empty', async () => {
    const user = userEvent.setup();
    renderWithQueryClient(<PatientCreateForm />);

    await user.click(screen.getByRole('button', { name: 'Register patient' }));

    expect(await screen.findByText('Medical record number is required.')).toBeInTheDocument();
    expect(screen.getByText('Full name is required.')).toBeInTheDocument();
    expect(mockedInvoke).not.toHaveBeenCalled();
  });

  it('submits the full field set and navigates to the new patient on success', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockResolvedValueOnce({
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
    });
    renderWithQueryClient(<PatientCreateForm />);

    await user.type(screen.getByLabelText('Medical record number'), 'MRN-0001');
    await user.type(screen.getByLabelText('Full name'), 'Ada Lovelace');
    await user.type(screen.getByLabelText('Date of birth'), '1990-01-01');
    await user.selectOptions(screen.getByLabelText('Sex'), 'female');
    await user.click(screen.getByRole('button', { name: 'Register patient' }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith('patients_create', {
        input: {
          medicalRecordNumber: 'MRN-0001',
          fullName: 'Ada Lovelace',
          dateOfBirth: '1990-01-01',
          sex: 'female',
        },
      });
    });
    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({ to: '/patients/1' });
    });
  });
});
