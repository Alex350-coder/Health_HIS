import { cva, type VariantProps } from 'class-variance-authority';
import { type ButtonHTMLAttributes } from 'react';

import { cn } from '@shared/lib/cn';

const buttonVariants = cva(
  'inline-flex items-center justify-center gap-2 rounded-md font-semibold transition-colors focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50',
  {
    variants: {
      intent: {
        primary: 'bg-accent text-accent-foreground hover:bg-accent/90',
        secondary:
          'bg-surface-raised text-text-primary border border-border-default hover:bg-surface',
        danger: 'bg-danger text-white hover:bg-danger/90',
        ghost: 'bg-transparent text-text-primary hover:bg-surface',
      },
      size: {
        sm: 'h-9 px-3 text-sm',
        md: 'h-11 px-4 text-base',
        lg: 'h-12 px-6 text-lg',
      },
    },
    defaultVariants: {
      intent: 'primary',
      size: 'md',
    },
  },
);

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & VariantProps<typeof buttonVariants>;

export function Button({
  className,
  intent,
  size,
  type = 'button',
  ...rest
}: ButtonProps): JSX.Element {
  return (
    <button type={type} className={cn(buttonVariants({ intent, size }), className)} {...rest} />
  );
}
