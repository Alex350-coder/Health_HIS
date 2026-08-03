import { mutationErrorMessage } from '@shared/errors/error-messages';

import { useAuditList } from '../api/audit-queries';

import { AuditLogRow } from './AuditLogRow';
import { IntegrityCheck } from './IntegrityCheck';

const DEFAULT_PAGE_SIZE = 50;

/** Read-only audit trail view + integrity check. Loading/empty/error triad per UI.md Section 6. */
export function AuditLogView(): JSX.Element {
  const auditList = useAuditList({ limit: DEFAULT_PAGE_SIZE, offset: 0 });

  return (
    <section aria-label="Audit log">
      <h2>Audit log</h2>
      <IntegrityCheck />

      {auditList.isLoading ? <p aria-busy="true">Loading audit entries…</p> : null}

      {auditList.isError ? (
        <p role="alert">
          {mutationErrorMessage(auditList.error) ?? 'Something went wrong loading the audit log.'}
        </p>
      ) : null}

      {auditList.isSuccess && auditList.data.length === 0 ? <p>No audit entries yet.</p> : null}

      {auditList.isSuccess && auditList.data.length > 0 ? (
        <table>
          <thead>
            <tr>
              <th>Timestamp</th>
              <th>User</th>
              <th>Action</th>
              <th>Entity</th>
              <th>Result</th>
            </tr>
          </thead>
          <tbody>
            {auditList.data.map((entry) => (
              <AuditLogRow key={entry.id} entry={entry} />
            ))}
          </tbody>
        </table>
      ) : null}
    </section>
  );
}
