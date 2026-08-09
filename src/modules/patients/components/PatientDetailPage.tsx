import { Outlet, useParams } from '@tanstack/react-router';

import { ErrorBoundary } from '@shared/components/ErrorBoundary';
import { mutationErrorMessage } from '@shared/errors/error-messages';
import { ErrorState } from '@shared/ui/ErrorState';
import { Skeleton } from '@shared/ui/Skeleton';

import { usePatientDetail } from '../api/patient-queries';

import { PatientSummaryCard } from './PatientSummaryCard';

/**
 * Hub page — tabs into Medical History and Billing for this patient via nested routes.
 * `patientId` arrives already parsed to a positive integer by the route's own `params.parse`
 * (router.tsx), so no re-parsing happens here.
 */
export default function PatientDetailPage(): JSX.Element {
  const { patientId } = useParams({ strict: false });
  const hasId = typeof patientId === 'number';
  const query = usePatientDetail(hasId ? patientId : 0, hasId);

  return (
    <ErrorBoundary>
      <main className="flex flex-col gap-4 p-6">
        <PatientDetailContent query={query} />
        <Outlet />
      </main>
    </ErrorBoundary>
  );
}

function PatientDetailContent({
  query,
}: {
  query: ReturnType<typeof usePatientDetail>;
}): JSX.Element {
  if (query.isLoading) {
    return <Skeleton className="h-64 w-full" />;
  }
  if (!query.isSuccess) {
    return (
      <ErrorState
        description={mutationErrorMessage(query.error) ?? 'Could not load this patient.'}
        onRetry={() => void query.refetch()}
      />
    );
  }
  return <PatientSummaryCard patient={query.data} />;
}
