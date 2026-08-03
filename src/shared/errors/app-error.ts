/**
 * Mirrors `src-tauri/src/errors/app_error.rs`'s `AppError` enum (ErrorHandling.md Section 1) —
 * the one documented, intentionally-duplicated contract (Rule 17.4). Every field here must stay
 * in lockstep with the Rust enum; a mismatch is a schema-drift bug.
 */
export type AppError =
  | { type: 'Validation'; field: string; message: string }
  | { type: 'NotFound'; entity: string; id: number }
  | { type: 'Conflict'; message: string }
  | { type: 'Unauthorized' }
  | { type: 'AccountLocked'; retryAfterSecs: number }
  | { type: 'Database'; message: string; correlationId: string }
  | { type: 'Filesystem'; message: string; correlationId: string }
  | { type: 'Unexpected'; message: string; correlationId: string };

export function isAppError(value: unknown): value is AppError {
  return (
    typeof value === 'object' && value !== null && 'type' in value && typeof value.type === 'string'
  );
}

/** Wraps an `AppError` so it can be `throw`n (plain objects aren't valid throw targets per lint rule). */
export class AppErrorException extends Error {
  readonly appError: AppError;

  constructor(appError: AppError) {
    super(
      appError.type === 'Validation' || appError.type === 'Conflict'
        ? appError.message
        : appError.type,
    );
    this.name = 'AppErrorException';
    this.appError = appError;
  }
}
