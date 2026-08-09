import { ErrorBoundary } from '@shared/components/ErrorBoundary';

export default function InventoryListPage(): JSX.Element {
  return (
    <ErrorBoundary>
      <main>
        <h1>Inventory</h1>
      </main>
    </ErrorBoundary>
  );
}
