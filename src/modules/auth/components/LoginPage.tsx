import { ErrorBoundary } from '@shared/components/ErrorBoundary';

import { LoginForm } from './LoginForm';

export default function LoginPage(): JSX.Element {
  return (
    <ErrorBoundary>
      <main>
        <h1>Hospital Information System</h1>
        <LoginForm />
      </main>
    </ErrorBoundary>
  );
}
