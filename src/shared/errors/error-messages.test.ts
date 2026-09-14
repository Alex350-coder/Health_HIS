import { describe, expect, it } from 'vitest';

import { AppErrorException } from './app-error';
import { loginErrorMessage, mutationErrorMessage, toUserMessage } from './error-messages';

describe('toUserMessage', () => {
  it('returns the message for a Validation error', () => {
    expect(toUserMessage({ type: 'Validation', field: 'username', message: 'too short' })).toBe(
      'too short',
    );
  });

  it('returns a generic message for NotFound', () => {
    expect(toUserMessage({ type: 'NotFound', entity: 'user', id: 1 })).toBe(
      'The requested item could not be found.',
    );
  });

  it('returns the message for a Conflict error', () => {
    expect(toUserMessage({ type: 'Conflict', message: 'already exists' })).toBe('already exists');
  });

  it('returns a session-expired message for Unauthorized', () => {
    expect(toUserMessage({ type: 'Unauthorized' })).toBe(
      'Your session has expired. Please log in again.',
    );
  });

  it('formats the retry window for AccountLocked', () => {
    expect(toUserMessage({ type: 'AccountLocked', retryAfterSecs: 125 })).toBe(
      'Too many failed attempts. Try again in 3 minute(s).',
    );
  });

  it('includes the correlation id for Database errors', () => {
    expect(toUserMessage({ type: 'Database', message: 'boom', correlationId: 'abc-123' })).toBe(
      'Something went wrong saving your changes. (Error reference: abc-123)',
    );
  });

  it('includes the correlation id for Filesystem errors', () => {
    expect(toUserMessage({ type: 'Filesystem', message: 'boom', correlationId: 'abc-123' })).toBe(
      'Something went wrong accessing a file. (Error reference: abc-123)',
    );
  });

  it('includes the correlation id for Unexpected errors', () => {
    expect(toUserMessage({ type: 'Unexpected', message: 'boom', correlationId: 'abc-123' })).toBe(
      'Something went wrong. (Error reference: abc-123)',
    );
  });
});

describe('mutationErrorMessage', () => {
  it('extracts a user-facing message from an AppErrorException', () => {
    const error = new AppErrorException({ type: 'Conflict', message: 'username already taken' });

    expect(mutationErrorMessage(error)).toBe('username already taken');
  });

  it('returns undefined for a non-AppErrorException value', () => {
    expect(mutationErrorMessage(new Error('plain error'))).toBeUndefined();
  });

  it('returns undefined when there is no error', () => {
    expect(mutationErrorMessage(undefined)).toBeUndefined();
  });
});

describe('loginErrorMessage', () => {
  it('returns an invalid-credentials message for Unauthorized, not the session-expiry text', () => {
    const error = new AppErrorException({ type: 'Unauthorized' });

    expect(loginErrorMessage(error)).toBe('Invalid username or password.');
  });

  it('falls back to formErrorMessage for other inline-eligible error types', () => {
    const error = new AppErrorException({
      type: 'Validation',
      field: 'username',
      message: 'Username is required.',
    });

    expect(loginErrorMessage(error)).toBe('Username is required.');
  });

  it('returns undefined for a non-AppErrorException value', () => {
    expect(loginErrorMessage(new Error('plain error'))).toBeUndefined();
  });
});
