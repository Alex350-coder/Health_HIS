import { useParams } from '@tanstack/react-router';

import { ErrorBoundary } from '@shared/components/ErrorBoundary';
import { mutationErrorMessage } from '@shared/errors/error-messages';
import { Button } from '@shared/ui/Button';
import { EmptyState } from '@shared/ui/EmptyState';
import { ErrorState } from '@shared/ui/ErrorState';
import { Skeleton } from '@shared/ui/Skeleton';

import { useCreateEncounter } from '../api/medical-history-mutations';
import { useMedicalHistory } from '../api/medical-history-queries';

import { EncounterPanel } from './EncounterPanel';
import { PastEncounterCard } from './PastEncounterCard';

import type { MedicalHistoryBundle } from '../types/medical-history-schemas';

export default function MedicalHistoryTab(): JSX.Element {
  const { patientId } = useParams({ strict: false });
  const hasId = typeof patientId === 'number';
  const query = useMedicalHistory(hasId ? patientId : 0, hasId);

  return (
    <ErrorBoundary>
      <section className="flex flex-col gap-4">
        <h2 className="text-xl font-semibold text-text-primary">Medical History</h2>
        <MedicalHistoryContent query={query} patientId={hasId ? patientId : 0} />
      </section>
    </ErrorBoundary>
  );
}

function MedicalHistoryContent({
  query,
  patientId,
}: {
  query: ReturnType<typeof useMedicalHistory>;
  patientId: number;
}): JSX.Element {
  if (query.isLoading) {
    return <Skeleton className="h-64 w-full" />;
  }
  if (!query.isSuccess) {
    return (
      <ErrorState
        description={mutationErrorMessage(query.error) ?? 'Could not load the medical history.'}
        onRetry={() => void query.refetch()}
      />
    );
  }
  return <MedicalHistoryBody bundle={query.data} patientId={patientId} />;
}

function MedicalHistoryBody({
  bundle,
  patientId,
}: {
  bundle: MedicalHistoryBundle;
  patientId: number;
}): JSX.Element {
  const openEncounter = bundle.encounters.find((encounter) => encounter.status === 'open');
  const pastEncounters = bundle.encounters
    .filter((encounter) => encounter.status !== 'open')
    .sort((a, b) => b.admittedAt.localeCompare(a.admittedAt));

  return (
    <div className="flex flex-col gap-6">
      {openEncounter ? (
        <EncounterPanel
          encounter={openEncounter}
          diagnoses={bundle.diagnoses.filter((d) => d.encounterId === openEncounter.id)}
          treatments={bundle.treatments.filter((t) => t.encounterId === openEncounter.id)}
          evolutions={bundle.evolutions.filter((e) => e.encounterId === openEncounter.id)}
        />
      ) : (
        <StartEncounterAction patientId={patientId} />
      )}
      {pastEncounters.length > 0 ? (
        <div className="flex flex-col gap-4">
          <h3 className="text-base font-semibold text-text-primary">Past encounters</h3>
          {pastEncounters.map((encounter) => (
            <PastEncounterCard
              key={encounter.id}
              encounter={encounter}
              diagnoses={bundle.diagnoses.filter((d) => d.encounterId === encounter.id)}
              treatments={bundle.treatments.filter((t) => t.encounterId === encounter.id)}
              evolutions={bundle.evolutions.filter((e) => e.encounterId === encounter.id)}
            />
          ))}
        </div>
      ) : null}
    </div>
  );
}

function StartEncounterAction({ patientId }: { patientId: number }): JSX.Element {
  const createEncounter = useCreateEncounter();

  return (
    <EmptyState
      title="No open encounter"
      description="Start an encounter to begin recording diagnoses, treatments, and evolution notes."
      action={
        <Button
          size="sm"
          disabled={createEncounter.isPending}
          onClick={() => createEncounter.mutate({ patientId })}
        >
          {createEncounter.isPending ? 'Starting…' : 'Start encounter'}
        </Button>
      }
    />
  );
}
