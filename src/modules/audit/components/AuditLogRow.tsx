import { TableCell, TableRow } from '@shared/ui/Table';

import type { AuditLogEntry } from '../types/audit-types';

export function AuditLogRow({ entry }: { entry: AuditLogEntry }): JSX.Element {
  return (
    <TableRow>
      <TableCell>{entry.timestamp}</TableCell>
      <TableCell>{entry.userId ?? '—'}</TableCell>
      <TableCell>{entry.action}</TableCell>
      <TableCell>
        {entry.entityType}
        {entry.entityId !== null ? ` #${String(entry.entityId)}` : ''}
      </TableCell>
      <TableCell>{entry.result}</TableCell>
    </TableRow>
  );
}
