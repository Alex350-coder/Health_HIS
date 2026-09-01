import { useMutation, useQueryClient, type UseMutationResult } from '@tanstack/react-query';

import { callCommand } from '@shared/lib/api-client';

import type {
  CreateInventoryCategoryInput,
  CreateInventoryItemInput,
  CreateInventoryTransactionInput,
  InventoryCategory,
  InventoryItem,
  InventoryTransaction,
  MaintenanceSchedule,
  ScheduleMaintenanceInput,
} from '../types/inventory-schemas';

export function useCreateInventoryCategory(): UseMutationResult<
  InventoryCategory,
  unknown,
  CreateInventoryCategoryInput
> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateInventoryCategoryInput) =>
      callCommand<InventoryCategory>('inventory_create_category', { input }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['inventory', 'categories'] });
    },
  });
}

export function useCreateInventoryItem(): UseMutationResult<
  InventoryItem,
  unknown,
  CreateInventoryItemInput
> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateInventoryItemInput) =>
      callCommand<InventoryItem>('inventory_create_item', { input }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['inventory', 'items'] });
    },
  });
}

export function useRecordInventoryTransaction(): UseMutationResult<
  InventoryTransaction,
  unknown,
  CreateInventoryTransactionInput
> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateInventoryTransactionInput) =>
      callCommand<InventoryTransaction>('inventory_record_transaction', { input }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['inventory', 'items'] });
      void queryClient.invalidateQueries({ queryKey: ['inventory', 'transactions'] });
    },
  });
}

export function useScheduleMaintenance(): UseMutationResult<
  MaintenanceSchedule,
  unknown,
  ScheduleMaintenanceInput
> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: ScheduleMaintenanceInput) =>
      callCommand<MaintenanceSchedule>('inventory_schedule_maintenance', { input }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['inventory', 'maintenance'] });
    },
  });
}
