import { useMutation, useQueryClient, type UseMutationResult } from '@tanstack/react-query';

import { callCommand } from '@shared/lib/api-client';

import type {
  Bed,
  CreateBedInput,
  CreateFloorInput,
  CreateRoomInput,
  Floor,
  Room,
  SetBedStatusInput,
} from '../types/bed-schemas';

export function useCreateFloor(): UseMutationResult<Floor, unknown, CreateFloorInput> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateFloorInput) => callCommand<Floor>('beds_create_floor', { input }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['beds'] });
      void queryClient.invalidateQueries({ queryKey: ['hospital-map', 'layout'] });
    },
  });
}

export function useCreateRoom(): UseMutationResult<Room, unknown, CreateRoomInput> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateRoomInput) => callCommand<Room>('beds_create_room', { input }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['beds'] });
      void queryClient.invalidateQueries({ queryKey: ['hospital-map', 'layout'] });
    },
  });
}

export function useCreateBed(): UseMutationResult<Bed, unknown, CreateBedInput> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateBedInput) => callCommand<Bed>('beds_create', { input }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['beds'] });
      void queryClient.invalidateQueries({ queryKey: ['hospital-map', 'layout'] });
    },
  });
}

export function useSetBedStatus(): UseMutationResult<Bed, unknown, SetBedStatusInput> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: SetBedStatusInput) => callCommand<Bed>('beds_set_status', { input }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['beds'] });
      void queryClient.invalidateQueries({ queryKey: ['hospital-map', 'layout'] });
    },
  });
}
