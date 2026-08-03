import type { AuditLogEntry } from '../types/audit-types';

export function AuditLogRow({ entry }: { entry: AuditLogEntry }): JSX.Element {
  return (
    <tr>
      <td>{entry.timestamp}</td>
      <td>{entry.userId ?? '—'}</td>
      <td>{entry.action}</td>
      <td>
        {entry.entityType}
        {entry.entityId !== null ? ` #${String(entry.entityId)}` : ''}
      </td>
      <td>{entry.result}</td>
    </tr>
  );
}
