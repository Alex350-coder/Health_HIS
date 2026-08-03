import type { AppError } from './app-error';

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
