import { ErrorBoundary } from '@shared/components/ErrorBoundary';

export default function MedicalHistoryTab(): JSX.Element {
  return (
    <ErrorBoundary>
      <section>
        <h2>Medical History</h2>
      </section>
    </ErrorBoundary>
  );
}
