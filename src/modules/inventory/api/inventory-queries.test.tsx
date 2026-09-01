import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import {
  useInventoryCategoriesList,
  useInventoryItemsList,
  useInventoryMaintenanceSchedulesList,
  useInventoryTransactionsList,
} from './inventory-queries';

import type {
  InventoryCategory,
  InventoryItem,
  InventoryTransaction,
  MaintenanceSchedule,
} from '../types/inventory-schemas';
import type { ReactNode } from 'react';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

function wrapper({ children }: { children: ReactNode }): JSX.Element {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

const CATEGORY: InventoryCategory = {
  id: 1,
  name: 'Analgesics',
  kind: 'medicine',
};

const ITEM: InventoryItem = {
  id: 1,
  categoryId: 1,
  name: 'Ibuprofen 400mg',
  quantity: 50,
  unit: 'box',
  reorderThreshold: 10,
  expirationDate: null,
  location: 'Pharmacy Shelf B2',
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: null,
};

describe('useInventoryCategoriesList', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('calls inventory_list_categories with no arguments', async () => {
    mockedInvoke.mockResolvedValueOnce([CATEGORY]);
    const { result } = renderHook(() => useInventoryCategoriesList(), { wrapper });

    await waitFor(() => {
      expect(result.current.data).toEqual([CATEGORY]);
    });
    expect(mockedInvoke).toHaveBeenCalledWith('inventory_list_categories', {});
  });
});

describe('useInventoryItemsList', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('calls inventory_list_items with default filters when none are given', async () => {
    mockedInvoke.mockResolvedValueOnce([ITEM]);
    const { result } = renderHook(() => useInventoryItemsList(), { wrapper });

    await waitFor(() => {
      expect(result.current.data).toEqual([ITEM]);
    });
    expect(mockedInvoke).toHaveBeenCalledWith('inventory_list_items', {
      categoryId: null,
      lowStockOnly: false,
    });
  });

  it('calls inventory_list_items with the given filters', async () => {
    mockedInvoke.mockResolvedValueOnce([ITEM]);
    const { result } = renderHook(
      () => useInventoryItemsList({ categoryId: 1, lowStockOnly: true }),
      { wrapper },
    );

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });
    expect(mockedInvoke).toHaveBeenCalledWith('inventory_list_items', {
      categoryId: 1,
      lowStockOnly: true,
    });
  });
});

describe('useInventoryTransactionsList', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('calls inventory_list_transactions with the item id', async () => {
    const transaction: InventoryTransaction = {
      id: 1,
      itemId: 1,
      quantityDelta: 50,
      reason: 'restock',
      encounterId: null,
      treatmentId: null,
      performedByUserId: 1,
      createdAt: '2026-01-01T00:00:00Z',
    };
    mockedInvoke.mockResolvedValueOnce([transaction]);
    const { result } = renderHook(() => useInventoryTransactionsList(1), { wrapper });

    await waitFor(() => {
      expect(result.current.data).toEqual([transaction]);
    });
    expect(mockedInvoke).toHaveBeenCalledWith('inventory_list_transactions', { itemId: 1 });
  });
});

describe('useInventoryMaintenanceSchedulesList', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('calls inventory_list_maintenance_schedules with the item id', async () => {
    const schedule: MaintenanceSchedule = {
      id: 1,
      inventoryItemId: 1,
      scheduledDate: '2026-03-01',
      completedDate: null,
      notes: null,
      createdAt: '2026-01-01T00:00:00Z',
    };
    mockedInvoke.mockResolvedValueOnce([schedule]);
    const { result } = renderHook(() => useInventoryMaintenanceSchedulesList(1), { wrapper });

    await waitFor(() => {
      expect(result.current.data).toEqual([schedule]);
    });
    expect(mockedInvoke).toHaveBeenCalledWith('inventory_list_maintenance_schedules', {
      itemId: 1,
    });
  });
});
