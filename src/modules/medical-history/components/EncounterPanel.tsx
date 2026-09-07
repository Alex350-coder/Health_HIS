import { Badge } from '@shared/ui/Badge';
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@shared/ui/Card';

import { useInventoryItemsList } from '@modules/inventory/api/inventory-queries';

import { DiagnosisForm } from './DiagnosisForm';
import { DiagnosisList } from './DiagnosisList';
import { DischargeDialog } from './DischargeDialog';
import { EvolutionForm } from './EvolutionForm';
import { EvolutionList } from './EvolutionList';
import { TreatmentForm } from './TreatmentForm';
import { TreatmentList } from './TreatmentList';

import type { Diagnosis, Encounter, Evolution, Treatment } from '../types/medical-history-schemas';

interface EncounterPanelProps {
  encounter: Encounter;
  diagnoses: Diagnosis[];
  treatments: Treatment[];
  evolutions: Evolution[];
  patientId: number;
}

/** Timeline + inline forms for the currently open encounter. */
export function EncounterPanel({
  encounter,
  diagnoses,
  treatments,
  evolutions,
  patientId,
}: EncounterPanelProps): JSX.Element {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle>
          Open encounter — admitted {new Date(encounter.admittedAt).toLocaleString()}
        </CardTitle>
        <Badge>{encounter.status}</Badge>
      </CardHeader>
      <EncounterTimeline
        encounter={encounter}
        diagnoses={diagnoses}
        treatments={treatments}
        evolutions={evolutions}
        patientId={patientId}
      />
      <CardFooter>
        <DischargeDialog encounterId={encounter.id} patientId={patientId} />
      </CardFooter>
    </Card>
  );
}

function EncounterTimeline({
  encounter,
  diagnoses,
  treatments,
  evolutions,
}: EncounterPanelProps): JSX.Element {
  return (
    <CardContent className="flex flex-col gap-6">
      <EncounterSection title="Diagnoses">
        <DiagnosisList diagnoses={diagnoses} />
        <DiagnosisForm encounterId={encounter.id} existingDiagnoses={diagnoses} />
      </EncounterSection>
      <EncounterSection title="Treatments">
        <TreatmentList treatments={treatments} />
        <TreatmentFormWithInventory
          encounterId={encounter.id}
          diagnoses={diagnoses}
          treatments={treatments}
        />
      </EncounterSection>
      <EncounterSection title="Evolution notes">
        <EvolutionList evolutions={evolutions} />
        <EvolutionForm encounterId={encounter.id} />
      </EncounterSection>
    </CardContent>
  );
}

/** Supplies the current inventory list to `TreatmentForm` so it can offer a consumption field. */
function TreatmentFormWithInventory({
  encounterId,
  diagnoses,
  treatments,
}: {
  encounterId: number;
  diagnoses: Diagnosis[];
  treatments: Treatment[];
}): JSX.Element {
  const { data: items } = useInventoryItemsList();

  return (
    <TreatmentForm
      encounterId={encounterId}
      existingDiagnoses={diagnoses}
      existingTreatments={treatments}
      existingItems={items ?? []}
    />
  );
}

function EncounterSection({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}): JSX.Element {
  return (
    <section className="flex flex-col gap-3">
      <h3 className="text-base font-semibold text-text-primary">{title}</h3>
      {children}
    </section>
  );
}
