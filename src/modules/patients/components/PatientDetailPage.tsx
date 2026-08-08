import { Outlet, useParams } from '@tanstack/react-router';

import { ErrorBoundary } from '@shared/components/ErrorBoundary';

/** Hub page — tabs into Medical History and Billing for this patient via nested routes. */
export default function PatientDetailPage(): JSX.Element {
  const { patientId } = useParams({ strict: false });

  return (
    <ErrorBoundary>
      <main>
        <h1>Patient {patientId}</h1>
        <Outlet />
      </main>
    </ErrorBoundary>
  );
}
