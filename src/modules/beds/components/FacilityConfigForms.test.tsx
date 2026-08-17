import { invoke } from '@tauri-apps/api/core';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithQueryClient } from '@shared/test/render-with-query-client';

import { CreateBedForm, CreateFloorForm, CreateRoomForm } from './FacilityConfigForms';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

describe('CreateFloorForm', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('shows a validation error when submitted empty', async () => {
    const user = userEvent.setup();
    renderWithQueryClient(<CreateFloorForm />);

    await user.click(screen.getByRole('button', { name: 'Add floor' }));

    expect(await screen.findByText('Name is required.')).toBeInTheDocument();
    expect(mockedInvoke).not.toHaveBeenCalled();
  });

  it('submits a valid floor', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockResolvedValueOnce({
      id: 1,
      name: 'Ground Floor',
      levelOrder: 0,
      createdAt: '2026-01-01T00:00:00Z',
    });
    renderWithQueryClient(<CreateFloorForm />);

    await user.type(screen.getByLabelText('Floor name'), 'Ground Floor');
    await user.clear(screen.getByLabelText('Level order'));
    await user.type(screen.getByLabelText('Level order'), '0');
    await user.click(screen.getByRole('button', { name: 'Add floor' }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith('beds_create_floor', {
        input: { name: 'Ground Floor', levelOrder: 0 },
      });
    });
  });
});

describe('CreateRoomForm', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('submits a valid room', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockResolvedValueOnce({
      id: 1,
      floorId: 1,
      name: 'Ward A',
      roomType: 'ward',
      mapX: 0.5,
      mapY: 0.5,
      createdAt: '2026-01-01T00:00:00Z',
    });
    renderWithQueryClient(<CreateRoomForm />);

    await user.type(screen.getByLabelText('Floor ID'), '1');
    await user.type(screen.getByLabelText('Room name'), 'Ward A');
    await user.selectOptions(screen.getByLabelText('Room type'), 'ward');
    await user.type(screen.getByLabelText('Map X (0–1)'), '0.5');
    await user.type(screen.getByLabelText('Map Y (0–1)'), '0.5');
    await user.click(screen.getByRole('button', { name: 'Add room' }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith('beds_create_room', {
        input: { floorId: 1, name: 'Ward A', roomType: 'ward', mapX: 0.5, mapY: 0.5 },
      });
    });
  });
});

describe('CreateBedForm', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('submits a valid bed', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockResolvedValueOnce({
      id: 1,
      roomId: 1,
      label: 'Bed 1A',
      status: 'available',
      createdAt: '2026-01-01T00:00:00Z',
      updatedAt: null,
    });
    renderWithQueryClient(<CreateBedForm />);

    await user.type(screen.getByLabelText('Room ID'), '1');
    await user.type(screen.getByLabelText('Bed label'), 'Bed 1A');
    await user.click(screen.getByRole('button', { name: 'Add bed' }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith('beds_create', {
        input: { roomId: 1, label: 'Bed 1A' },
      });
    });
  });
});
