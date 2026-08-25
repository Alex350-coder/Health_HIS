import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';

import { formErrorMessage } from '@shared/errors/error-messages';
import { Button } from '@shared/ui/Button';
import { FormField } from '@shared/ui/FormField';

import { useCreateOperatingRoom } from '@modules/operating-rooms/api/or-mutations';
import {
  createOperatingRoomSchema,
  type CreateOperatingRoomFormValues,
  type CreateOperatingRoomInput,
} from '@modules/operating-rooms/types/or-schemas';

import { useCreateBed, useCreateFloor, useCreateRoom } from '../api/bed-mutations';
import {
  createBedSchema,
  createFloorSchema,
  createRoomSchema,
  roomTypeSchema,
  type CreateBedFormValues,
  type CreateBedInput,
  type CreateFloorFormValues,
  type CreateFloorInput,
  type CreateRoomFormValues,
  type CreateRoomInput,
} from '../types/bed-schemas';

import type { FieldErrors, UseFormRegister } from 'react-hook-form';

const ROOM_TYPES = roomTypeSchema.options;

interface CreateRoomFormFieldsProps {
  register: UseFormRegister<CreateRoomFormValues>;
  errors: FieldErrors<CreateRoomFormValues>;
}

function CreateRoomFormFields({ register, errors }: CreateRoomFormFieldsProps): JSX.Element {
  return (
    <>
      <FormField id="room-floor-id" label="Floor ID" error={errors.floorId?.message}>
        <input id="room-floor-id" type="number" {...register('floorId')} />
      </FormField>
      <FormField id="room-name" label="Room name" error={errors.name?.message}>
        <input id="room-name" type="text" {...register('name')} />
      </FormField>
      <FormField id="room-type" label="Room type" error={errors.roomType?.message}>
        <select id="room-type" {...register('roomType')}>
          {ROOM_TYPES.map((roomType) => (
            <option key={roomType} value={roomType}>
              {roomType}
            </option>
          ))}
        </select>
      </FormField>
      <FormField id="room-map-x" label="Map X (0–1)" error={errors.mapX?.message}>
        <input id="room-map-x" type="number" step="0.01" min="0" max="1" {...register('mapX')} />
      </FormField>
      <FormField id="room-map-y" label="Map Y (0–1)" error={errors.mapY?.message}>
        <input id="room-map-y" type="number" step="0.01" min="0" max="1" {...register('mapY')} />
      </FormField>
    </>
  );
}

export function CreateFloorForm(): JSX.Element {
  const createFloor = useCreateFloor();
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<CreateFloorFormValues, unknown, CreateFloorInput>({
    resolver: zodResolver(createFloorSchema),
  });

  const onSubmit = handleSubmit((input) => {
    createFloor.mutate(input, { onSuccess: () => reset() });
  });

  const errorMessage = formErrorMessage(createFloor.error);

  return (
    <form onSubmit={(event) => void onSubmit(event)} noValidate aria-label="Add floor">
      <FormField id="floor-name" label="Floor name" error={errors.name?.message}>
        <input id="floor-name" type="text" {...register('name')} />
      </FormField>
      <FormField id="floor-level-order" label="Level order" error={errors.levelOrder?.message}>
        <input id="floor-level-order" type="number" {...register('levelOrder')} />
      </FormField>
      {errorMessage ? <p role="alert">{errorMessage}</p> : null}
      <Button type="submit" disabled={createFloor.isPending}>
        {createFloor.isPending ? 'Adding…' : 'Add floor'}
      </Button>
    </form>
  );
}

export function CreateRoomForm(): JSX.Element {
  const createRoom = useCreateRoom();
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<CreateRoomFormValues, unknown, CreateRoomInput>({
    resolver: zodResolver(createRoomSchema),
  });

  const onSubmit = handleSubmit((input) => {
    createRoom.mutate(input, { onSuccess: () => reset() });
  });

  const errorMessage = formErrorMessage(createRoom.error);

  return (
    <form onSubmit={(event) => void onSubmit(event)} noValidate aria-label="Add room">
      <CreateRoomFormFields register={register} errors={errors} />
      {errorMessage ? <p role="alert">{errorMessage}</p> : null}
      <Button type="submit" disabled={createRoom.isPending}>
        {createRoom.isPending ? 'Adding…' : 'Add room'}
      </Button>
    </form>
  );
}

export function PromoteRoomToOperatingRoomForm(): JSX.Element {
  const createOperatingRoom = useCreateOperatingRoom();
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<CreateOperatingRoomFormValues, unknown, CreateOperatingRoomInput>({
    resolver: zodResolver(createOperatingRoomSchema),
  });

  const onSubmit = handleSubmit((input) => {
    createOperatingRoom.mutate(input, { onSuccess: () => reset() });
  });

  const errorMessage = formErrorMessage(createOperatingRoom.error);

  return (
    <form
      onSubmit={(event) => void onSubmit(event)}
      noValidate
      aria-label="Promote to operating room"
    >
      <FormField id="operating-room-room-id" label="Room ID" error={errors.roomId?.message}>
        <input id="operating-room-room-id" type="number" {...register('roomId')} />
      </FormField>
      {errorMessage ? <p role="alert">{errorMessage}</p> : null}
      <Button type="submit" disabled={createOperatingRoom.isPending}>
        {createOperatingRoom.isPending ? 'Promoting…' : 'Promote to OR'}
      </Button>
    </form>
  );
}

export function CreateBedForm(): JSX.Element {
  const createBed = useCreateBed();
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<CreateBedFormValues, unknown, CreateBedInput>({
    resolver: zodResolver(createBedSchema),
  });

  const onSubmit = handleSubmit((input) => {
    createBed.mutate(input, { onSuccess: () => reset() });
  });

  const errorMessage = formErrorMessage(createBed.error);

  return (
    <form onSubmit={(event) => void onSubmit(event)} noValidate aria-label="Add bed">
      <FormField id="bed-room-id" label="Room ID" error={errors.roomId?.message}>
        <input id="bed-room-id" type="number" {...register('roomId')} />
      </FormField>
      <FormField id="bed-label" label="Bed label" error={errors.label?.message}>
        <input id="bed-label" type="text" {...register('label')} />
      </FormField>
      {errorMessage ? <p role="alert">{errorMessage}</p> : null}
      <Button type="submit" disabled={createBed.isPending}>
        {createBed.isPending ? 'Adding…' : 'Add bed'}
      </Button>
    </form>
  );
}
