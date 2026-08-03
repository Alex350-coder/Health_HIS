import { ErrorBoundary } from '@shared/components/ErrorBoundary';

export default function DashboardPage(): JSX.Element {
  return (
    <ErrorBoundary>
      <main>
        <h1>Hospital Information System</h1>
      </main>
    </ErrorBoundary>
  );
}
