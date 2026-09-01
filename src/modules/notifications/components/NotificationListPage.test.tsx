import { invoke } from '@tauri-apps/api/core';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithQueryClient } from '@shared/test/render-with-query-client';

import NotificationListPage from './NotificationListPage';

import type { Notification } from '../types/notification-schemas';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

const UNREAD_NOTIFICATION: Notification = {
  id: 1,
  type: 'low_stock',
  targetRole: null,
  message: 'Ibuprofen 400mg is at or below its reorder threshold',
  relatedEntityType: 'inventory_item',
  relatedEntityId: 5,
  isRead: false,
  createdAt: '2026-01-01T00:00:00Z',
};

function mockCommands(notifications: Notification[]): void {
  mockedInvoke.mockImplementation((command: string) => {
    if (command === 'notifications_list') return Promise.resolve(notifications);
    return Promise.reject(new Error(`unexpected command ${command}`));
  });
}

describe('NotificationListPage', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('shows an empty state when there are no notifications', async () => {
    mockCommands([]);
    renderWithQueryClient(<NotificationListPage />);

    expect(await screen.findByText('No notifications')).toBeInTheDocument();
  });

  it('flags an unread notification with a badge and explicit text', async () => {
    mockCommands([UNREAD_NOTIFICATION]);
    renderWithQueryClient(<NotificationListPage />);

    expect(
      await screen.findByText('Ibuprofen 400mg is at or below its reorder threshold'),
    ).toBeInTheDocument();
    expect(screen.getByText('Unread')).toBeInTheDocument();
  });

  it('marks a notification as read through the row action', async () => {
    const user = userEvent.setup();
    mockCommands([UNREAD_NOTIFICATION]);
    mockedInvoke.mockImplementation((command: string) => {
      if (command === 'notifications_list') return Promise.resolve([UNREAD_NOTIFICATION]);
      if (command === 'notifications_mark_read') {
        return Promise.resolve({ ...UNREAD_NOTIFICATION, isRead: true });
      }
      return Promise.reject(new Error(`unexpected command ${command}`));
    });
    renderWithQueryClient(<NotificationListPage />);

    await user.click(await screen.findByRole('button', { name: 'Mark read' }));

    expect(mockedInvoke).toHaveBeenCalledWith('notifications_mark_read', {
      input: { id: UNREAD_NOTIFICATION.id },
    });
  });
});
