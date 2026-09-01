import { useNavigate } from '@tanstack/react-router';
import { useState } from 'react';

import { ErrorBoundary } from '@shared/components/ErrorBoundary';
import { mutationErrorMessage } from '@shared/errors/error-messages';
import { EmptyState } from '@shared/ui/EmptyState';
import { ErrorState } from '@shared/ui/ErrorState';
import { Skeleton } from '@shared/ui/Skeleton';
import { VirtualizedTable } from '@shared/ui/VirtualizedTable';

import { useInventoryCategoriesList, useInventoryItemsList } from '../api/inventory-queries';

import { CreateInventoryCategoryForm } from './CreateInventoryCategoryForm';
import { CreateInventoryItemForm } from './CreateInventoryItemForm';
import { InventoryItemRow } from './InventoryItemRow';

import type { InventoryCategory, InventoryItem } from '../types/inventory-schemas';
import type { UseQueryResult } from '@tanstack/react-query';

function renderHeader(): JSX.Element {
  return (
    <tr>
      <th className="p-3 text-left text-sm font-semibold text-text-secondary">Name</th>
      <th className="p-3 text-left text-sm font-semibold text-text-secondary">Quantity</th>
      <th className="p-3 text-left text-sm font-semibold text-text-secondary">Reorder threshold</th>
      <th className="p-3 text-left text-sm font-semibold text-text-secondary">Expiration</th>
      <th className="p-3 text-left text-sm font-semibold text-text-secondary">Status</th>
    </tr>
  );
}

function InventoryItemsResults({ query }: { query: UseQueryResult<InventoryItem[]> }): JSX.Element {
  const navigate = useNavigate();

  if (query.isLoading) {
    return <Skeleton className="h-64 w-full" />;
  }
  if (query.isError) {
    return (
      <ErrorState
        description={mutationErrorMessage(query.error) ?? 'Could not load the inventory items.'}
        onRetry={() => void query.refetch()}
      />
    );
  }
  if (!query.isSuccess || query.data.length === 0) {
    return <EmptyState title="No inventory items yet" description="Add an item below to begin." />;
  }
  return (
    <VirtualizedTable<InventoryItem>
      items={query.data}
      getRowKey={(item) => item.id}
      renderHeader={renderHeader}
      renderRow={(item) => (
        <InventoryItemRow
          item={item}
          onSelect={() => void navigate({ to: `/inventory/${String(item.id)}` })}
        />
      )}
    />
  );
}

function InventoryFilters({
  lowStockOnly,
  onLowStockOnlyChange,
}: {
  lowStockOnly: boolean;
  onLowStockOnlyChange: (value: boolean) => void;
}): JSX.Element {
  return (
    <label className="flex items-center gap-2 text-sm font-medium text-text-primary">
      <input
        type="checkbox"
        checked={lowStockOnly}
        onChange={(event) => onLowStockOnlyChange(event.target.checked)}
      />
      Show low-stock items only
    </label>
  );
}

function InventoryCreateForms({ categories }: { categories: InventoryCategory[] }): JSX.Element {
  return (
    <section className="grid grid-cols-1 gap-6 md:grid-cols-2">
      <div>
        <h2 className="text-base font-semibold text-text-primary">Add category</h2>
        <CreateInventoryCategoryForm />
      </div>
      <div>
        <h2 className="text-base font-semibold text-text-primary">Add item</h2>
        {categories.length === 0 ? (
          <p className="text-sm text-text-secondary">Add a category first.</p>
        ) : (
          <CreateInventoryItemForm categories={categories} />
        )}
      </div>
    </section>
  );
}

/** Inventory's own list + write path — this module owns its tables, unlike OR's cross-module case. */
export default function InventoryListPage(): JSX.Element {
  const categoriesQuery = useInventoryCategoriesList();
  const [lowStockOnly, setLowStockOnly] = useState(false);
  const itemsQuery = useInventoryItemsList({ lowStockOnly });

  return (
    <ErrorBoundary>
      <main className="flex flex-col gap-6 p-6">
        <h1 className="text-xl font-semibold text-text-primary">Inventory</h1>
        <InventoryCreateForms categories={categoriesQuery.data ?? []} />
        <section className="flex flex-col gap-4">
          <div className="flex items-center justify-between">
            <h2 className="text-base font-semibold text-text-primary">Items</h2>
            <InventoryFilters lowStockOnly={lowStockOnly} onLowStockOnlyChange={setLowStockOnly} />
          </div>
          <InventoryItemsResults query={itemsQuery} />
        </section>
      </main>
    </ErrorBoundary>
  );
}
