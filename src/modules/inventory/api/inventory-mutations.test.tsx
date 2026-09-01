import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { renderHook, act } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import {
  useCreateInventoryCategory,
  useCreateInventoryItem,
  useRecordInventoryTransaction,
  useScheduleMaintenance,
} from './inventory-mutations';

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
import type { ReactNode } from 'react';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

function wrapper({ children }: { children: ReactNode }): JSX.Element {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

const CATEGORY: InventoryCategory = { id: 1, name: 'Analgesics', kind: 'medicine' };

const ITEM: InventoryItem = {
  id: 1,
  categoryId: 1,
  name: 'Ibuprofen 400mg',
  quantity: 0,
  unit: 'box',
  reorderThreshold: 10,
  expirationDate: null,
  location: null,
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: null,
};

const TRANSACTION: InventoryTransaction = {
  id: 1,
  itemId: 1,
  quantityDelta: 50,
  reason: 'restock',
  encounterId: null,
  treatmentId: null,
  performedByUserId: 1,
  createdAt: '2026-01-01T00:00:00Z',
};

const SCHEDULE: MaintenanceSchedule = {
  id: 1,
  inventoryItemId: 1,
  scheduledDate: '2026-03-01',
  completedDate: null,
  notes: null,
  createdAt: '2026-01-01T00:00:00Z',
};

describe('inventory-mutations', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('useCreateInventoryCategory calls inventory_create_category with the input', async () => {
    const input: CreateInventoryCategoryInput = { name: 'Analgesics', kind: 'medicine' };
    mockedInvoke.mockResolvedValueOnce(CATEGORY);
    const { result } = renderHook(() => useCreateInventoryCategory(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockedInvoke).toHaveBeenCalledWith('inventory_create_category', { input });
  });

  it('useCreateInventoryItem calls inventory_create_item with the input', async () => {
    const input: CreateInventoryItemInput = {
      categoryId: 1,
      name: 'Ibuprofen 400mg',
      unit: 'box',
      reorderThreshold: 10,
    };
    mockedInvoke.mockResolvedValueOnce(ITEM);
    const { result } = renderHook(() => useCreateInventoryItem(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockedInvoke).toHaveBeenCalledWith('inventory_create_item', { input });
  });

  it('useRecordInventoryTransaction calls inventory_record_transaction with the input', async () => {
    const input: CreateInventoryTransactionInput = {
      itemId: 1,
      quantityDelta: 50,
      reason: 'restock',
    };
    mockedInvoke.mockResolvedValueOnce(TRANSACTION);
    const { result } = renderHook(() => useRecordInventoryTransaction(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockedInvoke).toHaveBeenCalledWith('inventory_record_transaction', { input });
  });

  it('useScheduleMaintenance calls inventory_schedule_maintenance with the input', async () => {
    const input: ScheduleMaintenanceInput = {
      inventoryItemId: 1,
      scheduledDate: '2026-03-01',
    };
    mockedInvoke.mockResolvedValueOnce(SCHEDULE);
    const { result } = renderHook(() => useScheduleMaintenance(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync(input);
    });

    expect(mockedInvoke).toHaveBeenCalledWith('inventory_schedule_maintenance', { input });
  });
});
