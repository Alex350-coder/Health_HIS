import { describe, expect, it } from 'vitest';

import { AppErrorException, isAppError } from './app-error';

describe('isAppError', () => {
  it('accepts an object with a string type field', () => {
    expect(isAppError({ type: 'Unauthorized' })).toBe(true);
  });

  it('rejects null', () => {
    expect(isAppError(null)).toBe(false);
  });

  it('rejects primitives', () => {
    expect(isAppError('Unauthorized')).toBe(false);
    expect(isAppError(42)).toBe(false);
  });

  it('rejects an object without a type field', () => {
    expect(isAppError({ message: 'oops' })).toBe(false);
  });

  it('rejects an object whose type field is not a string', () => {
    expect(isAppError({ type: 1 })).toBe(false);
  });
});

describe('AppErrorException', () => {
  it('uses the message for Validation errors', () => {
    const exception = new AppErrorException({
      type: 'Validation',
      field: 'username',
      message: 'required',
    });

    expect(exception.message).toBe('required');
    expect(exception.appError).toEqual({
      type: 'Validation',
      field: 'username',
      message: 'required',
    });
    expect(exception.name).toBe('AppErrorException');
  });

  it('uses the message for Conflict errors', () => {
    const exception = new AppErrorException({ type: 'Conflict', message: 'duplicate' });

    expect(exception.message).toBe('duplicate');
  });

  it('falls back to the type discriminant for other variants', () => {
    const exception = new AppErrorException({ type: 'Unauthorized' });

    expect(exception.message).toBe('Unauthorized');
  });

  it('is a real Error instance', () => {
    const exception = new AppErrorException({ type: 'Unauthorized' });

    expect(exception).toBeInstanceOf(Error);
  });
});
