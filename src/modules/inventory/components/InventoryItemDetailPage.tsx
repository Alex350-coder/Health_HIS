import { useParams } from '@tanstack/react-router';

import { ErrorBoundary } from '@shared/components/ErrorBoundary';

export default function InventoryItemDetailPage(): JSX.Element {
  const { itemId } = useParams({ strict: false });

  return (
    <ErrorBoundary>
      <main>
        <h1>Inventory Item {itemId}</h1>
      </main>
    </ErrorBoundary>
  );
}
