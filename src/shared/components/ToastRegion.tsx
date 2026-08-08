import { useToastStore } from '@shared/lib/toast-store';
import { ToastMessage, ToastProvider, ToastViewport } from '@shared/ui/Toast';

/** Renders the global toast queue (`shared/lib/toast-store.ts`). Mounted once at the app root. */
export function ToastRegion(): JSX.Element {
  const toasts = useToastStore((state) => state.toasts);
  const dismissToast = useToastStore((state) => state.dismissToast);

  return (
    <ToastProvider>
      {toasts.map((toast) => (
        <ToastMessage
          key={toast.id}
          title={toast.title}
          status={toast.status}
          open
          onOpenChange={(open) => {
            if (!open) {
              dismissToast(toast.id);
            }
          }}
        />
      ))}
      <ToastViewport />
    </ToastProvider>
  );
}
