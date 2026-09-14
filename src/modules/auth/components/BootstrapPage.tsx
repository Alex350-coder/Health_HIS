import { ErrorBoundary } from '@shared/components/ErrorBoundary';
import { Card, CardContent, CardHeader, CardTitle } from '@shared/ui/Card';

import { BootstrapForm } from './BootstrapForm';

export default function BootstrapPage(): JSX.Element {
  return (
    <ErrorBoundary>
      <main className="flex min-h-dvh items-center justify-center bg-surface p-4">
        <Card className="w-full max-w-sm">
          <CardHeader className="items-center text-center">
            <CardTitle>Hospital Information System</CardTitle>
            <p className="text-sm text-text-secondary">Set up the first administrator account</p>
          </CardHeader>
          <CardContent>
            <BootstrapForm />
          </CardContent>
        </Card>
      </main>
    </ErrorBoundary>
  );
}
