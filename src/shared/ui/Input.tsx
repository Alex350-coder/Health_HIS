import { forwardRef, type InputHTMLAttributes } from 'react';

import { cn } from '@shared/lib/cn';

type InputProps = InputHTMLAttributes<HTMLInputElement>;

/** Styled native input (UI.md Section 8) — visible focus ring, error state via `aria-invalid`. */
export const Input = forwardRef<HTMLInputElement, InputProps>(function Input(
  { className, ...rest },
  ref,
) {
  return (
    <input
      ref={ref}
      className={cn(
        'h-11 w-full rounded-md border border-border-default bg-surface-raised px-3 text-base text-text-primary placeholder:text-text-secondary',
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent',
        'aria-invalid:border-danger aria-invalid:focus-visible:ring-danger',
        'disabled:cursor-not-allowed disabled:opacity-50',
        className,
      )}
      {...rest}
    />
  );
});
