import { useQuery, type UseQueryResult } from '@tanstack/react-query';

import { callCommand } from '@shared/lib/api-client';

import type { InventoryCategory, InventoryItem } from '../types/inventory-schemas';

export function useInventoryCategoriesList(): UseQueryResult<InventoryCategory[]> {
  return useQuery({
    queryKey: ['inventory', 'categories'],
    queryFn: () => callCommand<InventoryCategory[]>('inventory_list_categories', {}),
  });
}

export interface InventoryItemsListParams {
  categoryId?: number | null;
  lowStockOnly?: boolean;
}

export function useInventoryItemsList(
  params: InventoryItemsListParams = {},
): UseQueryResult<InventoryItem[]> {
  const categoryId = params.categoryId ?? null;
  const lowStockOnly = params.lowStockOnly ?? false;
  return useQuery({
    queryKey: ['inventory', 'items', categoryId, lowStockOnly],
    queryFn: () =>
      callCommand<InventoryItem[]>('inventory_list_items', {
        categoryId,
        lowStockOnly,
      }),
  });
}
