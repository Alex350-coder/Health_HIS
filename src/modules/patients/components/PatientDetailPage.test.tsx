import { invoke } from '@tauri-apps/api/core';
import { screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithQueryClient } from '@shared/test/render-with-query-client';

import PatientDetailPage from './PatientDetailPage';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

vi.mock('@tanstack/react-router', () => ({
  useParams: () => ({ patientId: 1 }),
  useNavigate: () => vi.fn(),
  Outlet: () => null,
}));

const mockedInvoke = vi.mocked(invoke);

const PATIENT = {
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

describe('PatientDetailPage', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('renders the patient summary once loaded', async () => {
    mockedInvoke.mockResolvedValueOnce(PATIENT);
    renderWithQueryClient(<PatientDetailPage />);

    expect(await screen.findByText('Ada Lovelace')).toBeInTheDocument();
    expect(screen.getByText('MRN-0001')).toBeInTheDocument();
  });

  it('shows an error state when the patient fails to load', async () => {
    mockedInvoke.mockRejectedValueOnce({ type: 'NotFound' });
    renderWithQueryClient(<PatientDetailPage />);

    expect(await screen.findByRole('alert')).toBeInTheDocument();
  });
});
