import { ErrorBoundary } from '@shared/components/ErrorBoundary';

export default function BillingTab(): JSX.Element {
  return (
    <ErrorBoundary>
      <section>
        <h2>Billing</h2>
      </section>
    </ErrorBoundary>
  );
}
