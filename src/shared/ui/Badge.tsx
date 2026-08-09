import { cva, type VariantProps } from 'class-variance-authority';
import { type HTMLAttributes, type ReactNode } from 'react';

import { cn } from '@shared/lib/cn';

const badgeVariants = cva(
  'inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-sm font-medium',
  {
    variants: {
      status: {
        success: 'bg-success/10 text-success',
        warning: 'bg-warning/10 text-warning',
        danger: 'bg-danger/10 text-danger',
        info: 'bg-info/10 text-info',
        neutral: 'bg-surface text-text-secondary',
      },
    },
    defaultVariants: {
      status: 'neutral',
    },
  },
);

type BadgeProps = HTMLAttributes<HTMLSpanElement> &
  VariantProps<typeof badgeVariants> & {
    /** Rendered before the label — Rule 19.2 requires a non-color status indicator alongside every status color. */
    icon?: ReactNode;
  };

export function Badge({ className, status, icon, children, ...rest }: BadgeProps): JSX.Element {
  return (
    <span className={cn(badgeVariants({ status }), className)} {...rest}>
      {icon}
      {children}
    </span>
  );
}
