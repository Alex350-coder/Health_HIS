import type { QueryKey } from '@tanstack/react-query';

/**
 * Central event-name -> query-keys-to-invalidate map (StateManagement.md Section 4). Every
 * mutation visible in more than one open view must have an entry here — populated per-module
 * starting Phase 4 as each service starts emitting its events (IPC.md Section 3).
 */
export const EVENT_QUERY_MAP: Record<string, QueryKey[]> = {
  'patients:record:created': [['patients', 'list']],
  'patients:record:updated': [
    ['patients', 'list'],
    ['patients', 'detail'],
  ],
};
