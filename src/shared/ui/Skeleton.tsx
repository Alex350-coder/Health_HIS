import { type HTMLAttributes } from 'react';

import { cn } from '@shared/lib/cn';

type SkeletonProps = HTMLAttributes<HTMLDivElement>;

/** Loading placeholder — the "Loading" state of the mandatory loading/empty/error triad (UI.md Section 6). */
export function Skeleton({ className, ...rest }: SkeletonProps): JSX.Element {
  return (
    <div
      role="status"
      aria-label="Loading"
      className={cn('animate-pulse rounded-md bg-border-default', className)}
      {...rest}
    />
  );
}
