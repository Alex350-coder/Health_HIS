import { create } from 'zustand';

export interface ToastEntry {
  id: string;
  title: string;
  status: 'success' | 'danger' | 'info';
}

interface ToastState {
  toasts: ToastEntry[];
  pushToast: (toast: Omit<ToastEntry, 'id'>) => void;
  dismissToast: (id: string) => void;
}

/**
 * Single global toast queue (ErrorHandling.md Section 3 — Conflict / transient Database errors
 * surface here). Rendered by `shared/components/ToastRegion.tsx`, mounted once at the app root.
 */
export const useToastStore = create<ToastState>((set) => ({
  toasts: [],
  pushToast: (toast) => {
    const id = crypto.randomUUID();
    set((state) => ({ toasts: [...state.toasts, { ...toast, id }] }));
  },
  dismissToast: (id) => {
    set((state) => ({ toasts: state.toasts.filter((toast) => toast.id !== id) }));
  },
}));
