import { ErrorBoundary } from '@shared/components/ErrorBoundary';

export default function OrSchedulePage(): JSX.Element {
  return (
    <ErrorBoundary>
      <main>
        <h1>Operating Rooms</h1>
      </main>
    </ErrorBoundary>
  );
}
