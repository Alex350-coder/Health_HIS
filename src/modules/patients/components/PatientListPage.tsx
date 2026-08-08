import { ErrorBoundary } from '@shared/components/ErrorBoundary';

export default function PatientListPage(): JSX.Element {
  return (
    <ErrorBoundary>
      <main>
        <h1>Patients</h1>
      </main>
    </ErrorBoundary>
  );
}
