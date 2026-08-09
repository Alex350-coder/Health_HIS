import { Link } from '@tanstack/react-router';

import { ErrorBoundary } from '@shared/components/ErrorBoundary';
import { Button } from '@shared/ui/Button';

import { PatientList } from './PatientList';

export default function PatientListPage(): JSX.Element {
  return (
    <ErrorBoundary>
      <main className="flex flex-col gap-4 p-6">
        <div className="flex items-center justify-between">
          <h1 className="text-xl font-semibold text-text-primary">Patients</h1>
          <Link to="/patients/new">
            <Button>Register Patient</Button>
          </Link>
        </div>
        <PatientList />
      </main>
    </ErrorBoundary>
  );
}
