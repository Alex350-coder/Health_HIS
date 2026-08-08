import { AlertTriangle } from 'lucide-react';

import { Button } from './Button';

interface ErrorStateProps {
  title?: string;
  description?: string;
  onRetry?: () => void;
}

/** Renders the `Unexpected`/`Filesystem` AppError categories (ErrorHandling.md Section 2). */
export function ErrorState({
  title = 'Something went wrong',
  description,
  onRetry,
}: ErrorStateProps): JSX.Element {
  return (
    <div
      role="alert"
      className="flex flex-col items-center gap-2 rounded-lg border border-danger/30 p-10 text-center"
    >
      <AlertTriangle className="size-6 text-danger" aria-hidden="true" />
      <p className="text-base font-semibold text-text-primary">{title}</p>
      {description !== undefined ? (
        <p className="text-sm text-text-secondary">{description}</p>
      ) : null}
      {onRetry !== undefined ? (
        <Button intent="secondary" size="sm" onClick={onRetry}>
          Retry
        </Button>
      ) : null}
    </div>
  );
}
