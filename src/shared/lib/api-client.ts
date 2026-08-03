import { invoke } from '@tauri-apps/api/core';

import { AppErrorException, isAppError, type AppError } from '../errors/app-error';

/**
 * Fires exactly once, in one place (ErrorHandling.md Section 2 "Authorization" row), whenever a
 * command reports `Unauthorized`. `app/App.tsx` wires this to clearing the session slice and
 * navigating to `/login` — kept as a callback (not a direct import) so `shared/` never depends on
 * `modules/auth` (Architecture.md — cross-module imports are forbidden; shared code has no module
 * of its own to import from).
 */
let onUnauthorized: (() => void) | undefined;

export function setUnauthorizedHandler(handler: () => void): void {
  onUnauthorized = handler;
}

/**
 * The one place every IPC call in the app goes through. Normalizes whatever `invoke` rejects
 * with into an `AppError`, and centrally handles `Unauthorized` (Rule: no ad hoc per-component
 * handling of session expiry).
 */
export async function callCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    const appError = normalizeError(error);
    if (appError.type === 'Unauthorized') {
      onUnauthorized?.();
    }
    throw new AppErrorException(appError);
  }
}

function normalizeError(error: unknown): AppError {
  if (isAppError(error)) {
    return error;
  }
  return {
    type: 'Unexpected',
    message: 'an unexpected error occurred',
    correlationId: 'unknown',
  };
}
