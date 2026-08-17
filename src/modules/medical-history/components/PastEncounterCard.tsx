import { Badge } from '@shared/ui/Badge';
import { Card, CardContent, CardHeader, CardTitle } from '@shared/ui/Card';

import { DiagnosisList } from './DiagnosisList';
import { EvolutionList } from './EvolutionList';
import { TreatmentList } from './TreatmentList';

import type { Diagnosis, Encounter, Evolution, Treatment } from '../types/medical-history-schemas';

interface PastEncounterCardProps {
  encounter: Encounter;
  diagnoses: Diagnosis[];
  treatments: Treatment[];
  evolutions: Evolution[];
}

/** Read-only summary of a discharged encounter — no forms, records are closed. */
export function PastEncounterCard({
  encounter,
  diagnoses,
  treatments,
  evolutions,
}: PastEncounterCardProps): JSX.Element {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle>
          {new Date(encounter.admittedAt).toLocaleDateString()} —{' '}
          {encounter.dischargedAt !== null
            ? new Date(encounter.dischargedAt).toLocaleDateString()
            : '—'}
        </CardTitle>
        <Badge status="neutral">{encounter.status}</Badge>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        {encounter.dischargeSummary !== null ? (
          <p className="text-sm text-text-primary">{encounter.dischargeSummary}</p>
        ) : null}
        <DiagnosisList diagnoses={diagnoses} />
        <TreatmentList treatments={treatments} />
        <EvolutionList evolutions={evolutions} />
      </CardContent>
    </Card>
  );
}
