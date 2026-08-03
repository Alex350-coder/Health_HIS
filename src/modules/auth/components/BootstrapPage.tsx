import { ErrorBoundary } from '@shared/components/ErrorBoundary';

import { BootstrapForm } from './BootstrapForm';

export default function BootstrapPage(): JSX.Element {
  return (
    <ErrorBoundary>
      <main>
        <h1>Set up the first administrator account</h1>
        <BootstrapForm />
      </main>
    </ErrorBoundary>
  );
}
