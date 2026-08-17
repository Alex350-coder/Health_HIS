import { zodResolver } from '@hookform/resolvers/zod';
import { useState } from 'react';
import { useForm } from 'react-hook-form';

import { formErrorMessage } from '@shared/errors/error-messages';
import { Badge } from '@shared/ui/Badge';
import { Button } from '@shared/ui/Button';
import { EmptyState } from '@shared/ui/EmptyState';
import { ErrorState } from '@shared/ui/ErrorState';
import { FormField } from '@shared/ui/FormField';
import { Skeleton } from '@shared/ui/Skeleton';

import { useSetBedStatus } from '../api/bed-mutations';
import { useFacilityLayout, useFacilityRoomStatus } from '../api/bed-queries';
import {
  setBedStatusSchema,
  type Bed,
  type SetBedStatusFormValues,
  type SetBedStatusInput,
} from '../types/bed-schemas';

import type { UseFormRegister } from 'react-hook-form';

const SETTABLE_STATUSES = setBedStatusSchema.shape.status.options;

const BED_STATUS_INTENT: Record<Bed['status'], 'success' | 'warning' | 'info'> = {
  available: 'success',
  occupied: 'info',
  maintenance: 'warning',
};

function BedStatusBadge({ status }: { status: Bed['status'] }): JSX.Element {
  return <Badge status={BED_STATUS_INTENT[status]}>{status}</Badge>;
}

interface BedStatusFieldProps {
  bedId: number;
  register: UseFormRegister<SetBedStatusFormValues>;
  error?: string | undefined;
}

function BedStatusField({ bedId, register, error }: BedStatusFieldProps): JSX.Element {
  return (
    <FormField id={`bed-status-${String(bedId)}`} label="Status" error={error}>
      <select id={`bed-status-${String(bedId)}`} {...register('status')}>
        {SETTABLE_STATUSES.map((status) => (
          <option key={status} value={status}>
            {status}
          </option>
        ))}
      </select>
    </FormField>
  );
}

function SetBedStatusForm({ bedId }: { bedId: number }): JSX.Element {
  const setStatus = useSetBedStatus();
  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<SetBedStatusFormValues, unknown, SetBedStatusInput>({
    resolver: zodResolver(setBedStatusSchema),
    defaultValues: { bedId },
  });

  const onSubmit = handleSubmit((input) => {
    setStatus.mutate(input);
  });

  const errorMessage = formErrorMessage(setStatus.error);

  return (
    <form
      onSubmit={(event) => void onSubmit(event)}
      noValidate
      aria-label={`Set status for bed ${String(bedId)}`}
      className="flex items-end gap-2"
    >
      <input type="hidden" {...register('bedId')} value={bedId} />
      <BedStatusField bedId={bedId} register={register} error={errors.status?.message} />
      {errorMessage ? <p role="alert">{errorMessage}</p> : null}
      <Button type="submit" size="sm" disabled={setStatus.isPending}>
        {setStatus.isPending ? 'Saving…' : 'Update'}
      </Button>
    </form>
  );
}

function RoomBeds({ roomId }: { roomId: number }): JSX.Element {
  const query = useFacilityRoomStatus(roomId);

  if (query.isLoading) {
    return <Skeleton className="h-16 w-full" />;
  }
  if (query.isError) {
    return (
      <ErrorState description="Could not load bed status." onRetry={() => void query.refetch()} />
    );
  }
  if (!query.isSuccess || query.data.beds.length === 0) {
    return <EmptyState title="No beds in this room" />;
  }

  return (
    <ul className="flex flex-col gap-2">
      {query.data.beds.map((bed) => (
        <li key={bed.id} className="flex items-center gap-3">
          <span>{bed.label}</span>
          <BedStatusBadge status={bed.status} />
          <SetBedStatusForm bedId={bed.id} />
        </li>
      ))}
    </ul>
  );
}

function RoomRow({ roomId, roomName }: { roomId: number; roomName: string }): JSX.Element {
  const [expanded, setExpanded] = useState(false);

  return (
    <li>
      <Button intent="secondary" size="sm" onClick={() => setExpanded((value) => !value)}>
        {expanded ? 'Hide' : 'Show'} beds — {roomName}
      </Button>
      {expanded ? <RoomBeds roomId={roomId} /> : null}
    </li>
  );
}

/** Read display of the current facility structure, sourced from the shared read-only layout query. */
export function FacilityStructureView(): JSX.Element {
  const query = useFacilityLayout();

  if (query.isLoading) {
    return <Skeleton className="h-64 w-full" />;
  }
  if (query.isError) {
    return (
      <ErrorState
        description="Could not load the facility layout."
        onRetry={() => void query.refetch()}
      />
    );
  }
  if (!query.isSuccess || query.data.length === 0) {
    return (
      <EmptyState title="No facility configured yet" description="Add a floor to get started." />
    );
  }

  return (
    <ul className="flex flex-col gap-4">
      {query.data.map((floor) => (
        <li key={floor.id}>
          <h2 className="text-base font-semibold text-text-primary">{floor.name}</h2>
          {floor.rooms.length === 0 ? (
            <p className="text-sm text-text-secondary">No rooms on this floor yet.</p>
          ) : (
            <ul className="flex flex-col gap-2 pl-4">
              {floor.rooms.map((room) => (
                <RoomRow key={room.id} roomId={room.id} roomName={room.name} />
              ))}
            </ul>
          )}
        </li>
      ))}
    </ul>
  );
}
