import { ErrorBoundary } from '@shared/components/ErrorBoundary';

export default function PatientCreatePage(): JSX.Element {
  return (
    <ErrorBoundary>
      <main>
        <h1>Register Patient</h1>
      </main>
    </ErrorBoundary>
  );
}
