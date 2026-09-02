import { useState } from 'react';

import { ErrorBoundary } from '@shared/components/ErrorBoundary';
import { mutationErrorMessage } from '@shared/errors/error-messages';
import { EmptyState } from '@shared/ui/EmptyState';
import { ErrorState } from '@shared/ui/ErrorState';
import { Skeleton } from '@shared/ui/Skeleton';

import { useMarkNotificationRead } from '../api/notification-mutations';
import { useNotificationsList } from '../api/notification-queries';

import { NotificationRow } from './NotificationRow';

import type { Notification } from '../types/notification-schemas';
import type { UseQueryResult } from '@tanstack/react-query';

function NotificationResults({
  query,
  onMarkRead,
  markReadPending,
}: {
  query: UseQueryResult<Notification[]>;
  onMarkRead: (id: number) => void;
  markReadPending: boolean;
}): JSX.Element {
  if (query.isLoading) {
    return <Skeleton className="h-64 w-full" />;
  }
  if (query.isError) {
    return (
      <ErrorState
        description={mutationErrorMessage(query.error) ?? 'Could not load notifications.'}
        onRetry={() => void query.refetch()}
      />
    );
  }
  if (!query.isSuccess || query.data.length === 0) {
    return <EmptyState title="No notifications" description="You're all caught up." />;
  }
  return (
    <ul className="flex flex-col rounded-lg border border-border-default">
      {query.data.map((notification) => (
        <NotificationRow
          key={notification.id}
          notification={notification}
          onMarkRead={() => onMarkRead(notification.id)}
          markReadPending={markReadPending}
        />
      ))}
    </ul>
  );
}

function NotificationFilters({
  unreadOnly,
  onUnreadOnlyChange,
}: {
  unreadOnly: boolean;
  onUnreadOnlyChange: (value: boolean) => void;
}): JSX.Element {
  return (
    <label className="flex items-center gap-2 text-sm font-medium text-text-primary">
      <input
        type="checkbox"
        checked={unreadOnly}
        onChange={(event) => onUnreadOnlyChange(event.target.checked)}
      />
      Show unread only
    </label>
  );
}

export default function NotificationListPage(): JSX.Element {
  const [unreadOnly, setUnreadOnly] = useState(false);
  const notificationsQuery = useNotificationsList({ unreadOnly });
  const markRead = useMarkNotificationRead();

  return (
    <ErrorBoundary>
      <main className="flex flex-col gap-6 p-6">
        <div className="flex items-center justify-between">
          <h1 className="text-xl font-semibold text-text-primary">Notifications</h1>
          <NotificationFilters unreadOnly={unreadOnly} onUnreadOnlyChange={setUnreadOnly} />
        </div>
        <NotificationResults
          query={notificationsQuery}
          onMarkRead={(id) => markRead.mutate({ id })}
          markReadPending={markRead.isPending}
        />
      </main>
    </ErrorBoundary>
  );
}
