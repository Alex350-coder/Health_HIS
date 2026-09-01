import { useParams } from '@tanstack/react-router';

import { ErrorBoundary } from '@shared/components/ErrorBoundary';
import { mutationErrorMessage } from '@shared/errors/error-messages';
import { Badge } from '@shared/ui/Badge';
import { EmptyState } from '@shared/ui/EmptyState';
import { ErrorState } from '@shared/ui/ErrorState';
import { Skeleton } from '@shared/ui/Skeleton';

import {
  useInventoryItemsList,
  useInventoryMaintenanceSchedulesList,
  useInventoryTransactionsList,
} from '../api/inventory-queries';

import { RecordInventoryTransactionDialog } from './RecordInventoryTransactionDialog';
import { ScheduleMaintenanceDialog } from './ScheduleMaintenanceDialog';

import type {
  InventoryItem,
  InventoryTransaction,
  MaintenanceSchedule,
} from '../types/inventory-schemas';
import type { UseQueryResult } from '@tanstack/react-query';

function ItemHeader({ item }: { item: InventoryItem }): JSX.Element {
  const isLowStock = item.quantity <= item.reorderThreshold;

  return (
    <header className="flex flex-col gap-2">
      <div className="flex items-center gap-3">
        <h1 className="text-xl font-semibold text-text-primary">{item.name}</h1>
        {isLowStock ? (
          <Badge status="warning">Low stock</Badge>
        ) : (
          <Badge status="success">OK</Badge>
        )}
      </div>
      <p className="text-sm text-text-secondary">
        {item.quantity} {item.unit} on hand · reorder threshold {item.reorderThreshold}
        {item.location ? ` · ${item.location}` : ''}
      </p>
      <p className="text-sm text-text-secondary">Expires: {item.expirationDate ?? '—'}</p>
      <div className="flex gap-2">
        <RecordInventoryTransactionDialog itemId={item.id} />
        <ScheduleMaintenanceDialog inventoryItemId={item.id} />
      </div>
    </header>
  );
}

function TransactionHistoryTable({
  query,
}: {
  query: UseQueryResult<InventoryTransaction[]>;
}): JSX.Element {
  if (query.isLoading) {
    return <Skeleton className="h-48 w-full" />;
  }
  if (query.isError) {
    return (
      <ErrorState
        description={mutationErrorMessage(query.error) ?? 'Could not load the transaction history.'}
        onRetry={() => void query.refetch()}
      />
    );
  }
  if (!query.isSuccess || query.data.length === 0) {
    return <EmptyState title="No transactions yet" description="Record one above to begin." />;
  }
  return (
    <table className="w-full">
      <thead>
        <tr>
          <th className="p-3 text-left text-sm font-semibold text-text-secondary">Date</th>
          <th className="p-3 text-left text-sm font-semibold text-text-secondary">Change</th>
          <th className="p-3 text-left text-sm font-semibold text-text-secondary">Reason</th>
        </tr>
      </thead>
      <tbody>
        {query.data.map((transaction) => (
          <tr key={transaction.id}>
            <td className="p-3 align-middle">{transaction.createdAt}</td>
            <td className="p-3 align-middle">{transaction.quantityDelta}</td>
            <td className="p-3 align-middle">{transaction.reason}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function MaintenanceScheduleList({
  query,
}: {
  query: UseQueryResult<MaintenanceSchedule[]>;
}): JSX.Element {
  if (query.isLoading) {
    return <Skeleton className="h-48 w-full" />;
  }
  if (query.isError) {
    return (
      <ErrorState
        description={
          mutationErrorMessage(query.error) ?? 'Could not load the maintenance schedule.'
        }
        onRetry={() => void query.refetch()}
      />
    );
  }
  if (!query.isSuccess || query.data.length === 0) {
    return <EmptyState title="No maintenance scheduled" description="Schedule one above." />;
  }
  return (
    <ul className="flex flex-col gap-2">
      {query.data.map((schedule) => (
        <li key={schedule.id} className="text-sm text-text-secondary">
          {schedule.scheduledDate} — {schedule.completedDate ? 'Completed' : 'Pending'}
          {schedule.notes ? ` — ${schedule.notes}` : ''}
        </li>
      ))}
    </ul>
  );
}

export default function InventoryItemDetailPage(): JSX.Element {
  const { itemId } = useParams({ strict: false });
  const parsedItemId = Number(itemId);

  const itemsQuery = useInventoryItemsList();
  const transactionsQuery = useInventoryTransactionsList(parsedItemId);
  const maintenanceQuery = useInventoryMaintenanceSchedulesList(parsedItemId);

  const item = itemsQuery.data?.find((candidate) => candidate.id === parsedItemId);

  return (
    <ErrorBoundary>
      <main className="flex flex-col gap-6 p-6">
        {itemsQuery.isLoading ? <Skeleton className="h-24 w-full" /> : null}
        {itemsQuery.isSuccess && !item ? (
          <EmptyState title="Item not found" description="This inventory item does not exist." />
        ) : null}
        {item ? <ItemHeader item={item} /> : null}
        <section className="flex flex-col gap-4">
          <h2 className="text-base font-semibold text-text-primary">Transaction history</h2>
          <TransactionHistoryTable query={transactionsQuery} />
        </section>
        <section className="flex flex-col gap-4">
          <h2 className="text-base font-semibold text-text-primary">Maintenance schedule</h2>
          <MaintenanceScheduleList query={maintenanceQuery} />
        </section>
      </main>
    </ErrorBoundary>
  );
}
