import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';

import { formErrorMessage } from '@shared/errors/error-messages';
import { Button } from '@shared/ui/Button';
import { Dialog, DialogContent, DialogFooter, DialogTrigger } from '@shared/ui/Dialog';
import { FormField } from '@shared/ui/FormField';

import { useAssignBed } from '../api/bed-mutations';
import { useBedsList } from '../api/bed-queries';
import {
  assignBedSchema,
  type AssignBedFormValues,
  type AssignBedInput,
} from '../types/bed-schemas';

interface BedAssignmentDialogProps {
  patientId: number;
  encounterId: number;
}

/** Offered from a patient's Beds tab for their open encounter (Plan.md Phase 7). */
export function BedAssignmentDialog({
  patientId,
  encounterId,
}: BedAssignmentDialogProps): JSX.Element {
  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button size="sm">Assign bed</Button>
      </DialogTrigger>
      <DialogContent title="Assign bed" description="Choose an available bed for this patient.">
        <AssignBedForm patientId={patientId} encounterId={encounterId} />
      </DialogContent>
    </Dialog>
  );
}

function AssignBedForm({ patientId, encounterId }: BedAssignmentDialogProps): JSX.Element {
  const bedsQuery = useBedsList();
  const availableBeds = (bedsQuery.data ?? []).filter((bed) => bed.activeAssignment === null);

  const assignBed = useAssignBed();
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<AssignBedFormValues, unknown, AssignBedInput>({
    resolver: zodResolver(assignBedSchema),
    defaultValues: { patientId, encounterId },
  });

  const onSubmit = handleSubmit((input) => {
    assignBed.mutate(input, { onSuccess: () => reset({ patientId, encounterId }) });
  });

  const errorMessage = formErrorMessage(assignBed.error);

  return (
    <form onSubmit={(event) => void onSubmit(event)} noValidate aria-label="Assign bed">
      <input type="hidden" {...register('patientId')} />
      <input type="hidden" {...register('encounterId')} />
      <FormField id="assign-bed-id" label="Bed" error={errors.bedId?.message}>
        <select id="assign-bed-id" {...register('bedId')} disabled={availableBeds.length === 0}>
          <option value="">Select a bed…</option>
          {availableBeds.map((bed) => (
            <option key={bed.id} value={bed.id}>
              {bed.label}
            </option>
          ))}
        </select>
      </FormField>
      {errorMessage ? <p role="alert">{errorMessage}</p> : null}
      <DialogFooter>
        <Button type="submit" disabled={assignBed.isPending || availableBeds.length === 0}>
          {assignBed.isPending ? 'Assigning…' : 'Assign'}
        </Button>
      </DialogFooter>
    </form>
  );
}
