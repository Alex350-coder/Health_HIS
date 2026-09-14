import { ErrorBoundary } from '@shared/components/ErrorBoundary';

import { AuditLogView } from './AuditLogView';

export default function AuditLogPage(): JSX.Element {
  return (
    <ErrorBoundary>
      <main className="flex flex-col gap-6 p-6">
        <AuditLogView />
      </main>
    </ErrorBoundary>
  );
}
