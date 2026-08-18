import { Link, Outlet, useParams } from '@tanstack/react-router';

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
        {hasId ? <PatientDetailNav patientId={patientId} /> : null}
        <Outlet />
      </main>
    </ErrorBoundary>
  );
}

const TAB_LINK_CLASSNAME =
  'px-3 py-2 text-sm font-medium text-text-secondary hover:text-text-primary';
const TAB_LINK_ACTIVE_CLASSNAME = 'text-text-primary border-b-2 border-accent';

function PatientDetailNav({ patientId }: { patientId: number }): JSX.Element {
  return (
    <nav className="flex gap-2 border-b border-border-default" aria-label="Patient tabs">
      <Link
        to="/patients/$patientId/medical-history"
        params={{ patientId }}
        className={TAB_LINK_CLASSNAME}
        activeProps={{ className: TAB_LINK_ACTIVE_CLASSNAME }}
      >
        Medical History
      </Link>
      <Link
        to="/patients/$patientId/beds"
        params={{ patientId }}
        className={TAB_LINK_CLASSNAME}
        activeProps={{ className: TAB_LINK_ACTIVE_CLASSNAME }}
      >
        Beds
      </Link>
      <Link
        to="/patients/$patientId/billing"
        params={{ patientId }}
        className={TAB_LINK_CLASSNAME}
        activeProps={{ className: TAB_LINK_ACTIVE_CLASSNAME }}
      >
        Billing
      </Link>
    </nav>
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
