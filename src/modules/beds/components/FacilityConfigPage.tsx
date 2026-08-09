import { ErrorBoundary } from '@shared/components/ErrorBoundary';

/** The only write path for structural facility data — floors, rooms, beds (IPC.md Section 2.1). */
export default function FacilityConfigPage(): JSX.Element {
  return (
    <ErrorBoundary>
      <main>
        <h1>Facility Configuration</h1>
      </main>
    </ErrorBoundary>
  );
}
