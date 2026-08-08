import { useQueryClient } from '@tanstack/react-query';
import { listen } from '@tauri-apps/api/event';
import { useEffect } from 'react';

import { EVENT_QUERY_MAP } from '@shared/lib/event-query-map';

/**
 * Subscribes to every event in `EVENT_QUERY_MAP` at the `App.tsx` root (StateManagement.md
 * Section 4) and invalidates the mapped query keys on receipt, so any mutation visible in more
 * than one open view propagates immediately everywhere (Rule 13.2).
 */
export function useEventSubscription(): void {
  const queryClient = useQueryClient();

  useEffect(() => {
    const unlistenPromises = Object.entries(EVENT_QUERY_MAP).map(([eventName, queryKeys]) =>
      listen(eventName, () => {
        queryKeys.forEach((queryKey) => {
          void queryClient.invalidateQueries({ queryKey });
        });
      }),
    );

    return () => {
      unlistenPromises.forEach((unlistenPromise) => {
        void unlistenPromise.then((unlisten) => unlisten());
      });
    };
  }, [queryClient]);
}
