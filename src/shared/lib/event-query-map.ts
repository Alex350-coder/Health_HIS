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
  // A treatment may optionally consume inventory stock (Database.md 3.5's `treatment_id`
  // linkage) — invalidate ['inventory'] too so an open Inventory view reflects the consumption.
  'medical-history:treatment:created': [['medical-history'], ['inventory']],
  'medical-history:evolution:created': [['medical-history']],
  // ['hospital-map'] (not ['hospital-map', 'layout']) so an open room-detail panel
  // (['hospital-map', 'room-status', roomId]) also refreshes on bed changes.
  'beds:facility:changed': [['beds'], ['hospital-map']],
  'beds:assignment:created': [['beds'], ['hospital-map'], ['medical-history']],
  'beds:assignment:released': [['beds'], ['hospital-map'], ['medical-history']],
  'operating-rooms:room:created': [['operating-rooms'], ['hospital-map']],
  'operating-rooms:reservation:created': [['operating-rooms'], ['hospital-map']],
  'operating-rooms:reservation:updated': [['operating-rooms'], ['hospital-map']],
  'operating-rooms:reservation:cancelled': [['operating-rooms'], ['hospital-map']],
  'inventory:category:created': [['inventory', 'categories']],
  'inventory:item:created': [['inventory', 'items']],
  'inventory:transaction:created': [
    ['inventory', 'items'],
    ['inventory', 'transactions'],
  ],
  'inventory:maintenance:created': [['inventory', 'maintenance']],
};
