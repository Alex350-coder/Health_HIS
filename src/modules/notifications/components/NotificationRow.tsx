import { Badge } from '@shared/ui/Badge';
import { Button } from '@shared/ui/Button';

import type { Notification } from '../types/notification-schemas';

interface NotificationRowProps {
  notification: Notification;
  onMarkRead: () => void;
  markReadPending: boolean;
}

/**
 * One notification's presentation (message, timestamp, mark-read action). Unread state is flagged
 * with both a badge and explicit text (Rule 19.2 — no color-only signaling).
 */
export function NotificationRow({
  notification,
  onMarkRead,
  markReadPending,
}: NotificationRowProps): JSX.Element {
  return (
    <li className="flex items-start justify-between gap-4 border-b border-border-default p-3 last:border-b-0">
      <div className="flex flex-col gap-1">
        <div className="flex items-center gap-2">
          {notification.isRead ? (
            <Badge status="neutral">Read</Badge>
          ) : (
            <Badge status="info">Unread</Badge>
          )}
          <span className="text-sm text-text-secondary">{notification.createdAt}</span>
        </div>
        <p className="text-sm font-medium text-text-primary">{notification.message}</p>
      </div>
      {notification.isRead ? null : (
        <Button intent="secondary" size="sm" onClick={onMarkRead} disabled={markReadPending}>
          Mark read
        </Button>
      )}
    </li>
  );
}
