import { useQuery } from '@tanstack/react-query';
import { useParams } from '@tanstack/react-router';

import { ErrorBoundary } from '@shared/components/ErrorBoundary';
import { mutationErrorMessage } from '@shared/errors/error-messages';
import { callCommand } from '@shared/lib/api-client';
import { Badge } from '@shared/ui/Badge';
import { Button } from '@shared/ui/Button';
import { EmptyState } from '@shared/ui/EmptyState';
import { ErrorState } from '@shared/ui/ErrorState';
import { Skeleton } from '@shared/ui/Skeleton';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@shared/ui/Table';

import type {
  Encounter,
  MedicalHistoryBundle,
} from '@modules/medical-history/types/medical-history-schemas';

import {
  useFinalizeBillingSimulation,
  useGenerateBillingSimulation,
} from '../api/billing-mutations';
import { useBillingSimulation } from '../api/billing-queries';

import type { BillingSimulationDetail } from '../types/billing-schemas';

const currencyFormatter = new Intl.NumberFormat('en-US', {
  style: 'currency',
  currency: 'USD',
});

/**
 * Route-mounted (patients/$patientId/billing). Cross-module data is fetched by re-issuing the
 * relevant IPC command directly rather than importing Medical History's hooks (CLAUDE.md
 * Section 5 — no direct imports between modules' component/hook trees), mirroring
 * `PatientBedTab.tsx`.
 */
export default function BillingTab(): JSX.Element {
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

  return (
    <ErrorBoundary>
      <section className="flex flex-col gap-4">
        <h2 className="text-xl font-semibold text-text-primary">Billing</h2>
        <BillingContent historyQuery={historyQuery} />
      </section>
    </ErrorBoundary>
  );
}

function BillingContent({
  historyQuery,
}: {
  historyQuery: ReturnType<typeof useQuery<MedicalHistoryBundle>>;
}): JSX.Element {
  if (historyQuery.isLoading) {
    return <Skeleton className="h-48 w-full" />;
  }
  if (!historyQuery.isSuccess) {
    return (
      <ErrorState
        description={
          mutationErrorMessage(historyQuery.error) ?? 'Could not load the medical history.'
        }
        onRetry={() => void historyQuery.refetch()}
      />
    );
  }

  const targetEncounter = selectTargetEncounter(historyQuery.data.encounters);
  if (!targetEncounter) {
    return (
      <EmptyState
        title="No encounters yet"
        description="Billing can only be simulated once the patient has an encounter."
      />
    );
  }

  return <EncounterBilling encounterId={targetEncounter.id} />;
}

/** Prefers the open encounter; otherwise the most recently admitted one. */
function selectTargetEncounter(encounters: Encounter[]): Encounter | undefined {
  const openEncounter = encounters.find((encounter) => encounter.status === 'open');
  if (openEncounter) {
    return openEncounter;
  }
  return [...encounters].sort((a, b) => b.admittedAt.localeCompare(a.admittedAt))[0];
}

function EncounterBilling({ encounterId }: { encounterId: number }): JSX.Element {
  const simulationQuery = useBillingSimulation(encounterId);

  if (simulationQuery.isLoading) {
    return <Skeleton className="h-48 w-full" />;
  }
  if (!simulationQuery.isSuccess) {
    return (
      <ErrorState
        description={
          mutationErrorMessage(simulationQuery.error) ?? 'Could not load the billing simulation.'
        }
        onRetry={() => void simulationQuery.refetch()}
      />
    );
  }

  if (!simulationQuery.data) {
    return <NoSimulationYet encounterId={encounterId} />;
  }

  return <SimulationView detail={simulationQuery.data} />;
}

function NoSimulationYet({ encounterId }: { encounterId: number }): JSX.Element {
  const generateSimulation = useGenerateBillingSimulation();

  return (
    <div className="flex flex-col gap-4">
      <SimulationOnlyNotice />
      <EmptyState
        title="No billing simulation yet"
        description="Generate a simulation to itemize room, treatment, inventory, and operating room charges for this encounter."
        action={
          <Button
            disabled={generateSimulation.isPending}
            onClick={() => generateSimulation.mutate({ encounterId })}
          >
            {generateSimulation.isPending ? 'Generating…' : 'Generate Simulation'}
          </Button>
        }
      />
      {generateSimulation.isError ? (
        <ErrorState
          description={
            mutationErrorMessage(generateSimulation.error) ??
            'Could not generate the billing simulation.'
          }
        />
      ) : null}
    </div>
  );
}

function SimulationView({ detail }: { detail: BillingSimulationDetail }): JSX.Element {
  const finalizeSimulation = useFinalizeBillingSimulation();

  return (
    <div className="flex flex-col gap-4">
      <SimulationOnlyNotice />
      <SimulationStatusBar simulation={detail.simulation} finalizeSimulation={finalizeSimulation} />
      <BillingItemsTable items={detail.items} />
      <SimulationTotal totalAmount={detail.simulation.totalAmount} />
      {finalizeSimulation.isError ? (
        <ErrorState
          description={
            mutationErrorMessage(finalizeSimulation.error) ??
            'Could not finalize the billing simulation.'
          }
        />
      ) : null}
    </div>
  );
}

function SimulationStatusBar({
  simulation,
  finalizeSimulation,
}: {
  simulation: BillingSimulationDetail['simulation'];
  finalizeSimulation: ReturnType<typeof useFinalizeBillingSimulation>;
}): JSX.Element {
  const isFinalized = simulation.status === 'finalized';

  return (
    <div className="flex items-center justify-between">
      <Badge status={isFinalized ? 'success' : 'neutral'}>
        {isFinalized ? 'Finalized' : 'Draft'}
      </Badge>
      {!isFinalized ? (
        <Button
          disabled={finalizeSimulation.isPending}
          onClick={() => finalizeSimulation.mutate({ id: simulation.id })}
        >
          {finalizeSimulation.isPending ? 'Finalizing…' : 'Finalize'}
        </Button>
      ) : null}
    </div>
  );
}

function BillingItemsTable({ items }: { items: BillingSimulationDetail['items'] }): JSX.Element {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Description</TableHead>
          <TableHead>Source</TableHead>
          <TableHead className="text-right">Amount</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {items.map((item) => (
          <TableRow key={item.id}>
            <TableCell>{item.description}</TableCell>
            <TableCell>{item.source.replace('_', ' ')}</TableCell>
            <TableCell className="text-right">{currencyFormatter.format(item.amount)}</TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}

function SimulationTotal({ totalAmount }: { totalAmount: number }): JSX.Element {
  return (
    <div className="flex items-center justify-end gap-2 border-t border-border-default pt-3">
      <span className="text-sm font-semibold text-text-secondary">Total</span>
      <span className="text-base font-semibold text-text-primary">
        {currencyFormatter.format(totalAmount)}
      </span>
    </div>
  );
}

function SimulationOnlyNotice(): JSX.Element {
  return (
    <p className="rounded-lg border border-border-default bg-surface p-3 text-sm text-text-secondary">
      Simulation Only — Not a real invoice.
    </p>
  );
}
