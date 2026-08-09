import * as DialogPrimitive from '@radix-ui/react-dialog';
import { X } from 'lucide-react';

import { cn } from '@shared/lib/cn';

import type { ComponentPropsWithoutRef, ReactNode } from 'react';

// Re-exporting Radix's own components under this module's name is the documented composition
// pattern (UI.md Section 8) — fast refresh only affects the dev server, not correctness.
/* eslint-disable react-refresh/only-export-components */
export const Dialog = DialogPrimitive.Root;
export const DialogTrigger = DialogPrimitive.Trigger;
/* eslint-enable react-refresh/only-export-components */

type DialogContentProps = ComponentPropsWithoutRef<typeof DialogPrimitive.Content> & {
  title: string;
  description?: string;
};

/**
 * Radix-backed modal — never hand-rolled, per UI.md Section 8 (focus trapping, Escape-to-close,
 * and return-focus-to-trigger are handled by Radix, not reimplemented here).
 */
export function DialogContent({
  className,
  title,
  description,
  children,
  ...rest
}: DialogContentProps): JSX.Element {
  return (
    <DialogPrimitive.Portal>
      <DialogPrimitive.Overlay className="fixed inset-0 z-40 bg-black/40" />
      <DialogPrimitive.Content
        className={cn(
          'fixed left-1/2 top-1/2 z-50 w-full max-w-md -translate-x-1/2 -translate-y-1/2',
          'rounded-lg border border-border-default bg-surface-raised p-6 shadow-lg',
          className,
        )}
        {...rest}
      >
        <DialogPrimitive.Title className="text-lg font-semibold text-text-primary">
          {title}
        </DialogPrimitive.Title>
        {description !== undefined ? (
          <DialogPrimitive.Description className="mt-1 text-sm text-text-secondary">
            {description}
          </DialogPrimitive.Description>
        ) : null}
        <div className="mt-4">{children}</div>
        <DialogPrimitive.Close
          aria-label="Close"
          className="absolute right-4 top-4 rounded-md text-text-secondary hover:text-text-primary focus-visible:outline-none"
        >
          <X className="size-4" aria-hidden="true" />
        </DialogPrimitive.Close>
      </DialogPrimitive.Content>
    </DialogPrimitive.Portal>
  );
}

export function DialogFooter({ children }: { children: ReactNode }): JSX.Element {
  return <div className="mt-6 flex justify-end gap-2">{children}</div>;
}
