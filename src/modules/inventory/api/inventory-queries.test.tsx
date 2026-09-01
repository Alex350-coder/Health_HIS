import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { useInventoryCategoriesList, useInventoryItemsList } from './inventory-queries';

import type { InventoryCategory, InventoryItem } from '../types/inventory-schemas';
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
