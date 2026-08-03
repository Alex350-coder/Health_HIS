import { useQuery, type UseQueryResult } from '@tanstack/react-query';

import { callCommand } from '@shared/lib/api-client';

import type { AuditListFilter, AuditLogEntry } from '../types/audit-types';

export function useAuditList(filter: AuditListFilter): UseQueryResult<AuditLogEntry[]> {
  return useQuery({
    queryKey: ['audit', 'list', filter],
    queryFn: () => callCommand<AuditLogEntry[]>('audit_list', { input: filter }),
  });
}
