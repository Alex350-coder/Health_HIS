import { type HTMLAttributes } from 'react';

import { cn } from '@shared/lib/cn';

export function Card({ className, ...rest }: HTMLAttributes<HTMLDivElement>): JSX.Element {
  return (
    <div
      className={cn('rounded-lg border border-border-default bg-surface-raised', className)}
      {...rest}
    />
  );
}

export function CardHeader({ className, ...rest }: HTMLAttributes<HTMLDivElement>): JSX.Element {
  return <div className={cn('flex flex-col gap-1 p-4', className)} {...rest} />;
}

export function CardTitle({
  className,
  children,
  ...rest
}: HTMLAttributes<HTMLHeadingElement>): JSX.Element {
  return (
    <h3 className={cn('text-lg font-semibold text-text-primary', className)} {...rest}>
      {children}
    </h3>
  );
}

export function CardContent({ className, ...rest }: HTMLAttributes<HTMLDivElement>): JSX.Element {
  return <div className={cn('p-4 pt-0', className)} {...rest} />;
}

export function CardFooter({ className, ...rest }: HTMLAttributes<HTMLDivElement>): JSX.Element {
  return (
    <div
      className={cn('flex items-center gap-2 border-t border-border-default p-4', className)}
      {...rest}
    />
  );
}
