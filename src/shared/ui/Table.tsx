import { type HTMLAttributes, type TdHTMLAttributes, type ThHTMLAttributes } from 'react';

import { cn } from '@shared/lib/cn';

/**
 * Semantic `<table>` primitives, per UI.md Section 5. Reach for `VirtualizedTable` instead once a
 * dataset can exceed 100 rows (Rule 18.1) — this primitive renders every row it is given.
 */
export function Table({ className, ...rest }: HTMLAttributes<HTMLTableElement>): JSX.Element {
  return (
    <div className="w-full overflow-x-auto">
      <table className={cn('w-full border-collapse text-sm', className)} {...rest} />
    </div>
  );
}

export function TableHeader({
  className,
  ...rest
}: HTMLAttributes<HTMLTableSectionElement>): JSX.Element {
  return <thead className={cn('border-b border-border-default', className)} {...rest} />;
}

export function TableBody({
  className,
  ...rest
}: HTMLAttributes<HTMLTableSectionElement>): JSX.Element {
  return <tbody className={className} {...rest} />;
}

export function TableRow({ className, ...rest }: HTMLAttributes<HTMLTableRowElement>): JSX.Element {
  return <tr className={cn('border-b border-border-default last:border-0', className)} {...rest} />;
}

export function TableHead({
  className,
  ...rest
}: ThHTMLAttributes<HTMLTableCellElement>): JSX.Element {
  return (
    <th
      scope="col"
      className={cn('px-3 py-2 text-left font-semibold text-text-secondary', className)}
      {...rest}
    />
  );
}

export function TableCell({
  className,
  ...rest
}: TdHTMLAttributes<HTMLTableCellElement>): JSX.Element {
  return <td className={cn('px-3 py-2 text-text-primary', className)} {...rest} />;
}
