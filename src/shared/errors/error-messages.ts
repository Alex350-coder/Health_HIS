import { AppErrorException } from './app-error';

import type { AppError } from './app-error';

/** Extracts a display message from a mutation/query `error`, or `undefined` if there is none. */
export function mutationErrorMessage(error: unknown): string | undefined {
  return error instanceof AppErrorException ? toUserMessage(error.appError) : undefined;
}

const NON_INLINE_ERROR_TYPES = new Set(['Conflict', 'Database', 'AccountLocked']);

/**
 * Like {@link mutationErrorMessage}, but omits the categories that have their own dedicated
 * presentation (ErrorHandling.md Section 3): Conflict/Database render as a toast, AccountLocked
 * as a countdown modal. Forms use this for their inline error banner.
 */
export function formErrorMessage(error: unknown): string | undefined {
  if (!(error instanceof AppErrorException) || NON_INLINE_ERROR_TYPES.has(error.appError.type)) {
    return undefined;
  }
  return toUserMessage(error.appError);
}

/** Extracts `retryAfterSecs` from an `AccountLocked` error, or `undefined` otherwise. */
export function accountLockedRetrySecs(error: unknown): number | undefined {
  if (error instanceof AppErrorException && error.appError.type === 'AccountLocked') {
    return error.appError.retryAfterSecs;
  }
  return undefined;
}

/**
 * The single place `AppError` values are turned into user-facing text (ErrorHandling.md
 * Section 3). No component formats an `AppError` message itself.
 */
export function toUserMessage(error: AppError): string {
  switch (error.type) {
    case 'Validation':
      return error.message;
    case 'NotFound':
      return 'The requested item could not be found.';
    case 'Conflict':
      return error.message;
    case 'Unauthorized':
      return 'Your session has expired. Please log in again.';
    case 'AccountLocked':
      return `Too many failed attempts. Try again in ${String(Math.ceil(error.retryAfterSecs / 60))} minute(s).`;
    case 'Database':
      return `Something went wrong saving your changes. (Error reference: ${error.correlationId})`;
    case 'Filesystem':
      return `Something went wrong accessing a file. (Error reference: ${error.correlationId})`;
    case 'Unexpected':
      return `Something went wrong. (Error reference: ${error.correlationId})`;
  }
}
