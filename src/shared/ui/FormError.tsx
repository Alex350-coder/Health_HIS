import { type HTMLAttributes } from 'react';

import { cn } from '@shared/lib/cn';

type FormErrorProps = HTMLAttributes<HTMLParagraphElement>;

/** Rendered next to its field, never only in a top-of-form summary (ErrorHandling.md Section 3). */
export function FormError({ className, children, ...rest }: FormErrorProps): JSX.Element | null {
  if (!children) {
    return null;
  }

  return (
    <p role="alert" className={cn('text-sm text-danger', className)} {...rest}>
      {children}
    </p>
  );
}
