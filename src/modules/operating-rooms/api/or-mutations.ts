import { useMutation, useQueryClient, type UseMutationResult } from '@tanstack/react-query';

import { callCommand } from '@shared/lib/api-client';

import type {
  CancelOrReservationInput,
  CreateOperatingRoomInput,
  CreateOrReservationInput,
  OperatingRoom,
  OrReservation,
  UpdateOrReservationInput,
} from '../types/or-schemas';

export function useCreateOperatingRoom(): UseMutationResult<
  OperatingRoom,
  unknown,
  CreateOperatingRoomInput
> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateOperatingRoomInput) =>
      callCommand<OperatingRoom>('operating_rooms_create', { input }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['operating-rooms'] });
      void queryClient.invalidateQueries({ queryKey: ['hospital-map'] });
    },
  });
}

export function useReserveOr(): UseMutationResult<
  OrReservation,
  unknown,
  CreateOrReservationInput
> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateOrReservationInput) =>
      callCommand<OrReservation>('operating_rooms_reserve', { input }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['operating-rooms'] });
      void queryClient.invalidateQueries({ queryKey: ['hospital-map'] });
    },
  });
}

export function useUpdateOrReservation(): UseMutationResult<
  OrReservation,
  unknown,
  UpdateOrReservationInput
> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: UpdateOrReservationInput) =>
      callCommand<OrReservation>('operating_rooms_update_reservation', { input }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['operating-rooms'] });
      void queryClient.invalidateQueries({ queryKey: ['hospital-map'] });
    },
  });
}

export function useCancelOrReservation(): UseMutationResult<
  OrReservation,
  unknown,
  CancelOrReservationInput
> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: CancelOrReservationInput) =>
      callCommand<OrReservation>('operating_rooms_cancel_reservation', { input }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['operating-rooms'] });
      void queryClient.invalidateQueries({ queryKey: ['hospital-map'] });
    },
  });
}
