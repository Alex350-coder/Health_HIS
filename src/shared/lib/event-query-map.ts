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
  'medical-history:encounter:created': [['medical-history'], ['patients', 'detail']],
  'medical-history:encounter:discharged': [['medical-history'], ['patients', 'detail']],
  'medical-history:diagnosis:created': [['medical-history']],
  'medical-history:treatment:created': [['medical-history']],
  'medical-history:evolution:created': [['medical-history']],
  // ['hospital-map'] (not ['hospital-map', 'layout']) so an open room-detail panel
  // (['hospital-map', 'room-status', roomId]) also refreshes on bed changes.
  'beds:facility:changed': [['beds'], ['hospital-map']],
  'beds:assignment:created': [['beds'], ['hospital-map'], ['medical-history']],
  'beds:assignment:released': [['beds'], ['hospital-map'], ['medical-history']],
};
