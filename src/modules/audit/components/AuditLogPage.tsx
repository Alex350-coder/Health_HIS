import { ErrorBoundary } from '@shared/components/ErrorBoundary';

import { AuditLogView } from './AuditLogView';

export default function AuditLogPage(): JSX.Element {
  return (
    <ErrorBoundary>
      <AuditLogView />
    </ErrorBoundary>
  );
}
