import { ErrorBoundary } from '@shared/components/ErrorBoundary';

export default function NotificationListPage(): JSX.Element {
  return (
    <ErrorBoundary>
      <main>
        <h1>Notifications</h1>
      </main>
    </ErrorBoundary>
  );
}
