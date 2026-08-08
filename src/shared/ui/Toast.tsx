import * as ToastPrimitive from '@radix-ui/react-toast';
import { cva, type VariantProps } from 'class-variance-authority';

import { cn } from '@shared/lib/cn';

const TOAST_DURATION_MS = 5000;

// Re-exporting Radix's own component under this module's name is the documented composition
// pattern (UI.md Section 8) — fast refresh only affects the dev server, not correctness.
// eslint-disable-next-line react-refresh/only-export-components
export const ToastProvider = ToastPrimitive.Provider;

export function ToastViewport(): JSX.Element {
  return (
    <ToastPrimitive.Viewport className="fixed bottom-4 right-4 z-50 flex w-full max-w-sm flex-col gap-2" />
  );
}

const toastVariants = cva('rounded-md border p-4 shadow-md', {
  variants: {
    status: {
      success: 'border-success/30 bg-surface-raised text-success',
      danger: 'border-danger/30 bg-surface-raised text-danger',
      info: 'border-info/30 bg-surface-raised text-info',
    },
  },
  defaultVariants: {
    status: 'info',
  },
});

interface ToastMessageProps extends VariantProps<typeof toastVariants> {
  title: string;
  description?: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

/** Transient, auto-dismissing notification for conflict/transient-error feedback (ErrorHandling.md Section 3). */
export function ToastMessage({
  title,
  description,
  status,
  open,
  onOpenChange,
}: ToastMessageProps): JSX.Element {
  return (
    <ToastPrimitive.Root
      className={cn(toastVariants({ status }))}
      duration={TOAST_DURATION_MS}
      open={open}
      onOpenChange={onOpenChange}
    >
      <ToastPrimitive.Title className="font-semibold">{title}</ToastPrimitive.Title>
      {description !== undefined ? (
        <ToastPrimitive.Description className="mt-1 text-sm text-text-secondary">
          {description}
        </ToastPrimitive.Description>
      ) : null}
    </ToastPrimitive.Root>
  );
}
