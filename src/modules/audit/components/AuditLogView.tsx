import { mutationErrorMessage } from '@shared/errors/error-messages';
import { Table, TableBody, TableHead, TableHeader, TableRow } from '@shared/ui/Table';

import { useAuditList } from '../api/audit-queries';

import { AuditLogRow } from './AuditLogRow';
import { IntegrityCheck } from './IntegrityCheck';

const DEFAULT_PAGE_SIZE = 50;

/** Read-only audit trail view + integrity check. Loading/empty/error triad per UI.md Section 6. */
export function AuditLogView(): JSX.Element {
  const auditList = useAuditList({ limit: DEFAULT_PAGE_SIZE, offset: 0 });

  return (
    <section aria-label="Audit log" className="flex flex-col gap-4">
      <h1 className="text-xl font-semibold text-text-primary">Audit log</h1>
      <IntegrityCheck />

      {auditList.isLoading ? <p aria-busy="true">Loading audit entries…</p> : null}

      {auditList.isError ? (
        <p role="alert">
          {mutationErrorMessage(auditList.error) ?? 'Something went wrong loading the audit log.'}
        </p>
      ) : null}

      {auditList.isSuccess && auditList.data.length === 0 ? <p>No audit entries yet.</p> : null}

      {auditList.isSuccess && auditList.data.length > 0 ? (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Timestamp</TableHead>
              <TableHead>User</TableHead>
              <TableHead>Action</TableHead>
              <TableHead>Entity</TableHead>
              <TableHead>Result</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {auditList.data.map((entry) => (
              <AuditLogRow key={entry.id} entry={entry} />
            ))}
          </TableBody>
        </Table>
      ) : null}
    </section>
  );
}
