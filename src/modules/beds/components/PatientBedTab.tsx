import { useQuery } from '@tanstack/react-query';
import { useParams } from '@tanstack/react-router';

import { ErrorBoundary } from '@shared/components/ErrorBoundary';
import { mutationErrorMessage } from '@shared/errors/error-messages';
import { callCommand } from '@shared/lib/api-client';
import { Button } from '@shared/ui/Button';
import { EmptyState } from '@shared/ui/EmptyState';
import { ErrorState } from '@shared/ui/ErrorState';
import { Skeleton } from '@shared/ui/Skeleton';

import type {
  Encounter,
  MedicalHistoryBundle,
} from '@modules/medical-history/types/medical-history-schemas';

import { useReleaseBed } from '../api/bed-mutations';
import { useBedsList } from '../api/bed-queries';

import { BedAssignmentDialog } from './BedAssignmentDialog';

import type { BedSummary } from '../types/bed-schemas';

/**
 * Route-mounted (patients/$patientId/beds). Cross-module data is fetched by re-issuing the
 * relevant IPC commands directly rather than importing Medical History's hooks (CLAUDE.md
 * Section 5 — no direct imports between modules' component/hook trees).
 */
export default function PatientBedTab(): JSX.Element {
  const { patientId } = useParams({ strict: false });
  const hasId = typeof patientId === 'number';

  const historyQuery = useQuery({
    queryKey: ['medical-history', 'by-patient', hasId ? patientId : 0],
    queryFn: () =>
      callCommand<MedicalHistoryBundle>('medical_history_get_by_patient', {
        input: { patientId: hasId ? patientId : 0 },
      }),
    enabled: hasId,
  });
  const bedsQuery = useBedsList();

  return (
    <ErrorBoundary>
      <section className="flex flex-col gap-4">
        <h2 className="text-xl font-semibold text-text-primary">Beds</h2>
        <PatientBedContent
          historyQuery={historyQuery}
          bedsQuery={bedsQuery}
          patientId={hasId ? patientId : 0}
        />
      </section>
    </ErrorBoundary>
  );
}

function PatientBedContent({
  historyQuery,
  bedsQuery,
  patientId,
}: {
  historyQuery: ReturnType<typeof useQuery<MedicalHistoryBundle>>;
  bedsQuery: ReturnType<typeof useBedsList>;
  patientId: number;
}): JSX.Element {
  if (historyQuery.isLoading || bedsQuery.isLoading) {
    return <Skeleton className="h-48 w-full" />;
  }
  if (!historyQuery.isSuccess) {
    return <HistoryLoadError query={historyQuery} />;
  }
  if (!bedsQuery.isSuccess) {
    return <BedsLoadError query={bedsQuery} />;
  }

  const openEncounter = historyQuery.data.encounters.find(
    (encounter: Encounter) => encounter.status === 'open',
  );
  if (!openEncounter) {
    return (
      <EmptyState
        title="No open encounter"
        description="A bed can only be assigned while the patient has an open encounter."
      />
    );
  }

  const currentBed = bedsQuery.data.find(
    (bed: BedSummary) => bed.activeAssignment?.encounterId === openEncounter.id,
  );
  if (currentBed?.activeAssignment) {
    return <CurrentBedCard bed={currentBed} assignmentId={currentBed.activeAssignment.id} />;
  }

  return <BedAssignmentDialog patientId={patientId} encounterId={openEncounter.id} />;
}

function HistoryLoadError({
  query,
}: {
  query: ReturnType<typeof useQuery<MedicalHistoryBundle>>;
}): JSX.Element {
  return (
    <ErrorState
      description={mutationErrorMessage(query.error) ?? 'Could not load the medical history.'}
      onRetry={() => void query.refetch()}
    />
  );
}

function BedsLoadError({ query }: { query: ReturnType<typeof useBedsList> }): JSX.Element {
  return (
    <ErrorState
      description={mutationErrorMessage(query.error) ?? 'Could not load the beds.'}
      onRetry={() => void query.refetch()}
    />
  );
}

function CurrentBedCard({
  bed,
  assignmentId,
}: {
  bed: BedSummary;
  assignmentId: number;
}): JSX.Element {
  const releaseBed = useReleaseBed();

  return (
    <div className="flex items-center justify-between rounded-lg border border-border-default p-4">
      <div>
        <p className="text-base font-semibold text-text-primary">{bed.label}</p>
        <p className="text-sm text-text-secondary">Status: {bed.status}</p>
      </div>
      <Button
        intent="secondary"
        disabled={releaseBed.isPending}
        onClick={() => releaseBed.mutate({ bedAssignmentId: assignmentId })}
      >
        {releaseBed.isPending ? 'Releasing…' : 'Release'}
      </Button>
    </div>
  );
}
