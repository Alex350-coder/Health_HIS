import { ErrorBoundary } from '@shared/components/ErrorBoundary';

export default function BedListPage(): JSX.Element {
  return (
    <ErrorBoundary>
      <main>
        <h1>Beds</h1>
      </main>
    </ErrorBoundary>
  );
}
