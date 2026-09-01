import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';

import { formErrorMessage } from '@shared/errors/error-messages';
import { Button } from '@shared/ui/Button';
import { Dialog, DialogContent, DialogFooter, DialogTrigger } from '@shared/ui/Dialog';
import { FormField } from '@shared/ui/FormField';

import { useScheduleMaintenance } from '../api/inventory-mutations';
import {
  scheduleMaintenanceSchema,
  type ScheduleMaintenanceFormValues,
  type ScheduleMaintenanceInput,
} from '../types/inventory-schemas';

import type { FieldErrors, UseFormRegister } from 'react-hook-form';

interface ScheduleMaintenanceDialogProps {
  inventoryItemId: number;
}

interface ScheduleMaintenanceFieldsProps {
  register: UseFormRegister<ScheduleMaintenanceFormValues>;
  errors: FieldErrors<ScheduleMaintenanceFormValues>;
}

function ScheduleMaintenanceFields({
  register,
  errors,
}: ScheduleMaintenanceFieldsProps): JSX.Element {
  return (
    <>
      <FormField
        id="maintenance-scheduled-date"
        label="Scheduled date"
        error={errors.scheduledDate?.message}
      >
        <input id="maintenance-scheduled-date" type="date" {...register('scheduledDate')} />
      </FormField>
      <FormField id="maintenance-notes" label="Notes" error={errors.notes?.message}>
        <textarea id="maintenance-notes" {...register('notes')} />
      </FormField>
    </>
  );
}

/** Offered from the item detail page to schedule an equipment maintenance visit. */
export function ScheduleMaintenanceDialog({
  inventoryItemId,
}: ScheduleMaintenanceDialogProps): JSX.Element {
  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button size="sm">Schedule maintenance</Button>
      </DialogTrigger>
      <DialogContent
        title="Schedule maintenance"
        description="Plan a maintenance visit for this equipment."
      >
        <ScheduleMaintenanceForm inventoryItemId={inventoryItemId} />
      </DialogContent>
    </Dialog>
  );
}

function ScheduleMaintenanceForm({ inventoryItemId }: ScheduleMaintenanceDialogProps): JSX.Element {
  const scheduleMaintenance = useScheduleMaintenance();
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<ScheduleMaintenanceFormValues, unknown, ScheduleMaintenanceInput>({
    resolver: zodResolver(scheduleMaintenanceSchema),
    defaultValues: { inventoryItemId },
  });

  const onSubmit = handleSubmit((input) => {
    scheduleMaintenance.mutate(input, { onSuccess: () => reset({ inventoryItemId }) });
  });

  const errorMessage = formErrorMessage(scheduleMaintenance.error);

  return (
    <form onSubmit={(event) => void onSubmit(event)} noValidate aria-label="Schedule maintenance">
      <input type="hidden" {...register('inventoryItemId')} />
      <ScheduleMaintenanceFields register={register} errors={errors} />
      {errorMessage ? <p role="alert">{errorMessage}</p> : null}
      <DialogFooter>
        <Button type="submit" disabled={scheduleMaintenance.isPending}>
          {scheduleMaintenance.isPending ? 'Scheduling…' : 'Schedule'}
        </Button>
      </DialogFooter>
    </form>
  );
}
