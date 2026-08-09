import { ErrorBoundary } from '@shared/components/ErrorBoundary';

/** Read-only visualization over Beds/Operating Rooms data — never writes (CLAUDE.md Section 3.3). */
export default function HospitalMapPage(): JSX.Element {
  return (
    <ErrorBoundary>
      <main>
        <h1>Hospital Map</h1>
      </main>
    </ErrorBoundary>
  );
}
