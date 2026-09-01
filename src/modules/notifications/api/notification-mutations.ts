import {
  useMutation,
  useQueryClient,
  type QueryKey,
  type UseMutationResult,
} from '@tanstack/react-query';

import { callCommand } from '@shared/lib/api-client';

import type { MarkNotificationReadInput, Notification } from '../types/notification-schemas';

interface MarkNotificationReadContext {
  previousLists: [QueryKey, Notification[] | undefined][];
}

// Optimistic update exception (StateManagement.md Section 5): marking a notification read has no
// clinical/workflow-correctness impact, so the list is patched locally before the round-trip
// completes and rolled back on failure.
export function useMarkNotificationRead(): UseMutationResult<
  Notification,
  unknown,
  MarkNotificationReadInput,
  MarkNotificationReadContext
> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: MarkNotificationReadInput) =>
      callCommand<Notification>('notifications_mark_read', { input }),
    onMutate: async (input: MarkNotificationReadInput) => {
      await queryClient.cancelQueries({ queryKey: ['notifications', 'list'] });
      const previousLists = queryClient.getQueriesData<Notification[]>({
        queryKey: ['notifications', 'list'],
      });
      for (const [queryKey, notifications] of previousLists) {
        if (notifications === undefined) continue;
        queryClient.setQueryData<Notification[]>(
          queryKey,
          notifications.map((notification) =>
            notification.id === input.id ? { ...notification, isRead: true } : notification,
          ),
        );
      }
      return { previousLists };
    },
    onError: (_error, _input, context) => {
      for (const [queryKey, notifications] of context?.previousLists ?? []) {
        queryClient.setQueryData(queryKey, notifications);
      }
    },
    onSettled: () => {
      void queryClient.invalidateQueries({ queryKey: ['notifications', 'list'] });
    },
  });
}
