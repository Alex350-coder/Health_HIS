import { useQuery } from '@tanstack/react-query';
import { Link } from '@tanstack/react-router';
import { useState } from 'react';

import { AppErrorException } from '@shared/errors/app-error';
import { callCommand } from '@shared/lib/api-client';
import { Button } from '@shared/ui/Button';
import { Dialog, DialogContent, DialogTrigger } from '@shared/ui/Dialog';

import type { BillingSimulationDetail } from '@modules/billing/types/billing-schemas';

import { DischargeForm } from './DischargeForm';

interface DischargeDialogProps {
  encounterId: number;
  patientId: number;
}

/**
 * Destructive-action confirmation for discharge (UI.md Section 4 — Dialog, never
 * `window.confirm`). Surfaces the encounter's billing-finalization status as an informational
 * prompt (Tasks.md 13.2) — it does not block discharge, since Billing is a no-payment-processor
 * simulation (CLAUDE.md Section 2).
 */
export function DischargeDialog({ encounterId, patientId }: DischargeDialogProps): JSX.Element {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <Dialog open={isOpen} onOpenChange={setIsOpen}>
      <DialogTrigger asChild>
        <Button intent="danger" size="sm">
          Discharge
        </Button>
      </DialogTrigger>
      <DialogContent
        title="Discharge patient"
        description="This closes the encounter and releases any bed still held for it."
      >
        <div className="flex flex-col gap-4">
          <BillingFinalizationNotice
            encounterId={encounterId}
            patientId={patientId}
            isOpen={isOpen}
          />
          <DischargeForm encounterId={encounterId} onDone={() => setIsOpen(false)} />
        </div>
      </DialogContent>
    </Dialog>
  );
}

/**
 * Re-issues `billing_get_simulation` directly rather than importing Billing's hooks (CLAUDE.md
 * Section 5 — no direct imports between modules' component/hook trees), mirroring how
 * `BillingTab.tsx` reads Medical History data in reverse. Shares `useBillingSimulation`'s query
 * key so the cache is a single source, not a duplicate. Gated on `isOpen` so it only fires while
 * the dialog is actually open.
 */
function useEncounterBillingStatus(encounterId: number, isOpen: boolean) {
  return useQuery({
    queryKey: ['billing', encounterId],
    queryFn: async () => {
      try {
        return await callCommand<BillingSimulationDetail>('billing_get_simulation', {
          encounterId,
        });
      } catch (error) {
        if (error instanceof AppErrorException && error.appError.type === 'NotFound') {
          return null;
        }
        throw error;
      }
    },
    enabled: isOpen,
  });
}

function BillingFinalizationNotice({
  encounterId,
  patientId,
  isOpen,
}: {
  encounterId: number;
  patientId: number;
  isOpen: boolean;
}): JSX.Element | null {
  const simulationQuery = useEncounterBillingStatus(encounterId, isOpen);

  if (!simulationQuery.isSuccess || simulationQuery.data?.simulation.status === 'finalized') {
    return null;
  }

  const notice = simulationQuery.data
    ? 'The billing simulation is still a draft.'
    : 'No billing simulation has been generated yet.';

  return (
    <p className="rounded-lg border border-border-default bg-surface p-3 text-sm text-text-secondary">
      {notice}{' '}
      <Link
        to="/patients/$patientId/billing"
        params={{ patientId }}
        className="font-medium text-accent underline"
      >
        Review billing
      </Link>
      .
    </p>
  );
}
