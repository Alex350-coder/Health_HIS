import { invoke } from '@tauri-apps/api/core';
import { screen } from '@testing-library/react';
import { beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithQueryClient } from '@shared/test/render-with-query-client';

import { PatientList } from './PatientList';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const navigateMock = vi.fn();
vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => navigateMock,
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

beforeAll(() => {
  for (const property of ['clientHeight', 'offsetHeight'] as const) {
    Object.defineProperty(HTMLElement.prototype, property, {
      configurable: true,
      value: 480,
    });
  }
  HTMLElement.prototype.getBoundingClientRect = (): DOMRect => ({
    x: 0,
    y: 0,
    width: 1024,
    height: 480,
    top: 0,
    left: 0,
    right: 1024,
    bottom: 480,
    toJSON(): unknown {
      return this;
    },
  });
});

describe('PatientList', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
    navigateMock.mockReset();
  });

  it('shows an empty state when there are no patients', async () => {
    mockedInvoke.mockResolvedValueOnce([]);
    renderWithQueryClient(<PatientList />);

    expect(await screen.findByText('No patients found')).toBeInTheDocument();
  });

  it('renders a row for each patient', async () => {
    mockedInvoke.mockResolvedValueOnce([PATIENT]);
    renderWithQueryClient(<PatientList />);

    expect(await screen.findByText('Ada Lovelace')).toBeInTheDocument();
    expect(screen.getByText('MRN-0001')).toBeInTheDocument();
  });

  it('shows an error state when the list fails to load', async () => {
    mockedInvoke.mockRejectedValueOnce({ type: 'Unexpected', message: 'boom', correlationId: 'x' });
    renderWithQueryClient(<PatientList />);

    expect(await screen.findByRole('alert')).toBeInTheDocument();
  });

  it('navigates to the patient detail route when a row is clicked', async () => {
    mockedInvoke.mockResolvedValueOnce([PATIENT]);
    renderWithQueryClient(<PatientList />);

    const cell = await screen.findByText('Ada Lovelace');
    cell.click();

    expect(navigateMock).toHaveBeenCalledWith({ to: '/patients/1' });
  });
});
