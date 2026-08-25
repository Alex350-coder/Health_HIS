import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';

import { formErrorMessage } from '@shared/errors/error-messages';
import { Button } from '@shared/ui/Button';
import { Dialog, DialogContent, DialogFooter, DialogTrigger } from '@shared/ui/Dialog';
import { FormField } from '@shared/ui/FormField';

import { useReserveOr } from '../api/or-mutations';
import {
  createOrReservationSchema,
  type CreateOrReservationFormValues,
  type CreateOrReservationInput,
} from '../types/or-schemas';

import type { FieldErrors, UseFormRegister } from 'react-hook-form';

interface ReserveOrDialogProps {
  operatingRoomId: number;
}

interface ReserveOrFieldsProps {
  register: UseFormRegister<CreateOrReservationFormValues>;
  errors: FieldErrors<CreateOrReservationFormValues>;
}

function ReserveOrSubjectFields({ register, errors }: ReserveOrFieldsProps): JSX.Element {
  return (
    <>
      <FormField id="reserve-or-patient-id" label="Patient ID" error={errors.patientId?.message}>
        <input id="reserve-or-patient-id" type="number" {...register('patientId')} />
      </FormField>
      <FormField
        id="reserve-or-encounter-id"
        label="Encounter ID"
        error={errors.encounterId?.message}
      >
        <input id="reserve-or-encounter-id" type="number" {...register('encounterId')} />
      </FormField>
      <FormField
        id="reserve-or-procedure-description"
        label="Procedure description"
        error={errors.procedureDescription?.message}
      >
        <textarea id="reserve-or-procedure-description" {...register('procedureDescription')} />
      </FormField>
    </>
  );
}

function ReserveOrScheduleFields({ register, errors }: ReserveOrFieldsProps): JSX.Element {
  return (
    <>
      <FormField
        id="reserve-or-scheduled-start"
        label="Start time"
        error={errors.scheduledStart?.message}
      >
        <input
          id="reserve-or-scheduled-start"
          type="datetime-local"
          {...register('scheduledStart')}
        />
      </FormField>
      <FormField
        id="reserve-or-scheduled-end"
        label="End time"
        error={errors.scheduledEnd?.message}
      >
        <input id="reserve-or-scheduled-end" type="datetime-local" {...register('scheduledEnd')} />
      </FormField>
    </>
  );
}

/** Offered from the OR schedule page for the currently selected operating room. */
export function ReserveOrDialog({ operatingRoomId }: ReserveOrDialogProps): JSX.Element {
  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button size="sm">New reservation</Button>
      </DialogTrigger>
      <DialogContent
        title="Reserve operating room"
        description="Schedule a procedure for this operating room."
      >
        <ReserveOrForm operatingRoomId={operatingRoomId} />
      </DialogContent>
    </Dialog>
  );
}

function ReserveOrForm({ operatingRoomId }: ReserveOrDialogProps): JSX.Element {
  const reserveOr = useReserveOr();
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<CreateOrReservationFormValues, unknown, CreateOrReservationInput>({
    resolver: zodResolver(createOrReservationSchema),
    defaultValues: { operatingRoomId },
  });

  const onSubmit = handleSubmit((input) => {
    reserveOr.mutate(input, { onSuccess: () => reset({ operatingRoomId }) });
  });

  const errorMessage = formErrorMessage(reserveOr.error);

  return (
    <form onSubmit={(event) => void onSubmit(event)} noValidate aria-label="Reserve operating room">
      <input type="hidden" {...register('operatingRoomId')} />
      <ReserveOrSubjectFields register={register} errors={errors} />
      <ReserveOrScheduleFields register={register} errors={errors} />
      {errorMessage ? <p role="alert">{errorMessage}</p> : null}
      <DialogFooter>
        <Button type="submit" disabled={reserveOr.isPending}>
          {reserveOr.isPending ? 'Reserving…' : 'Reserve'}
        </Button>
      </DialogFooter>
    </form>
  );
}
