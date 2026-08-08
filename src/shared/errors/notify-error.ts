import { useToastStore } from '@shared/lib/toast-store';

import { AppErrorException } from './app-error';
import { toUserMessage } from './error-messages';

const TOAST_ERROR_TYPES = new Set(['Conflict', 'Database']);

/**
 * Global `MutationCache.onError` hook (wired in `App.tsx`) — the one place Conflict and
 * transient Database errors surface as a toast (ErrorHandling.md Section 3). Every other
 * category is handled elsewhere (Validation inline, Unauthorized redirect, AccountLocked modal,
 * Unexpected/Filesystem via `ErrorBoundary`), so this is intentionally narrow.
 */
export function notifyMutationError(error: unknown): void {
  if (!(error instanceof AppErrorException)) {
    return;
  }
  if (!TOAST_ERROR_TYPES.has(error.appError.type)) {
    return;
  }
  useToastStore.getState().pushToast({ title: toUserMessage(error.appError), status: 'danger' });
}
