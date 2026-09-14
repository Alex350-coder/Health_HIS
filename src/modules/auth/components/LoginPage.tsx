import { ErrorBoundary } from '@shared/components/ErrorBoundary';
import { Card, CardContent, CardHeader, CardTitle } from '@shared/ui/Card';

import { LoginForm } from './LoginForm';

export default function LoginPage(): JSX.Element {
  return (
    <ErrorBoundary>
      <main className="flex min-h-dvh items-center justify-center bg-surface p-4">
        <Card className="w-full max-w-sm">
          <CardHeader className="items-center text-center">
            <CardTitle>Hospital Information System</CardTitle>
            <p className="text-sm text-text-secondary">Log in to continue</p>
          </CardHeader>
          <CardContent>
            <LoginForm />
          </CardContent>
        </Card>
      </main>
    </ErrorBoundary>
  );
}
