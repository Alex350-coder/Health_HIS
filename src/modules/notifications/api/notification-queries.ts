import { useQuery, type UseQueryResult } from '@tanstack/react-query';

import { callCommand } from '@shared/lib/api-client';

import type { Notification } from '../types/notification-schemas';

export interface NotificationsListParams {
  unreadOnly?: boolean;
}

export function useNotificationsList(
  params: NotificationsListParams = {},
): UseQueryResult<Notification[]> {
  const unreadOnly = params.unreadOnly ?? false;
  return useQuery({
    queryKey: ['notifications', 'list', unreadOnly],
    queryFn: () => callCommand<Notification[]>('notifications_list', { unreadOnly }),
  });
}
