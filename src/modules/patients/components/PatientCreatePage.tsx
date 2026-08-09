import { ErrorBoundary } from '@shared/components/ErrorBoundary';

import { PatientCreateForm } from './PatientCreateForm';

export default function PatientCreatePage(): JSX.Element {
  return (
    <ErrorBoundary>
      <main className="flex flex-col gap-4 p-6">
        <h1 className="text-xl font-semibold text-text-primary">Register Patient</h1>
        <PatientCreateForm />
      </main>
    </ErrorBoundary>
  );
}
