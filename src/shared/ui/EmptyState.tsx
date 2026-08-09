import { type ReactNode } from 'react';

interface EmptyStateProps {
  icon?: ReactNode;
  title: string;
  description?: string;
  action?: ReactNode;
}

/** Shown when a list/table has no rows yet — always pairs a message with an optional next action. */
export function EmptyState({ icon, title, description, action }: EmptyStateProps): JSX.Element {
  return (
    <div className="flex flex-col items-center gap-2 rounded-lg border border-dashed border-border-default p-10 text-center">
      {icon}
      <p className="text-base font-semibold text-text-primary">{title}</p>
      {description !== undefined ? (
        <p className="text-sm text-text-secondary">{description}</p>
      ) : null}
      {action}
    </div>
  );
}
