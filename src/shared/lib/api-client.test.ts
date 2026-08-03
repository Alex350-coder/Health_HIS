import { invoke } from '@tauri-apps/api/core';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { AppErrorException } from '../errors/app-error';

import { callCommand, setUnauthorizedHandler } from './api-client';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

describe('callCommand', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
    setUnauthorizedHandler(() => {
      /* no-op default so leftover handlers from other tests don't fire */
    });
  });

  it('resolves with the invoke result on success', async () => {
    mockedInvoke.mockResolvedValueOnce({ ok: true });

    await expect(callCommand('auth_current_user')).resolves.toEqual({ ok: true });
    expect(mockedInvoke).toHaveBeenCalledWith('auth_current_user', undefined);
  });

  it('passes args through to invoke', async () => {
    mockedInvoke.mockResolvedValueOnce(null);

    await callCommand('auth_login', { input: { username: 'a', password: 'b' } });

    expect(mockedInvoke).toHaveBeenCalledWith('auth_login', {
      input: { username: 'a', password: 'b' },
    });
  });

  it('wraps a rejected AppError in an AppErrorException', async () => {
    mockedInvoke.mockRejectedValueOnce({ type: 'Conflict', message: 'duplicate username' });

    await expect(callCommand('auth_create_user')).rejects.toMatchObject({
      appError: { type: 'Conflict', message: 'duplicate username' },
    });
  });

  it('the wrapped rejection is an AppErrorException instance', async () => {
    mockedInvoke.mockRejectedValueOnce({ type: 'Conflict', message: 'duplicate username' });

    await expect(callCommand('auth_create_user')).rejects.toBeInstanceOf(AppErrorException);
  });

  it('normalizes a non-AppError rejection to Unexpected', async () => {
    mockedInvoke.mockRejectedValueOnce(new Error('network down'));

    await expect(callCommand('auth_login')).rejects.toMatchObject({
      appError: {
        type: 'Unexpected',
        message: 'an unexpected error occurred',
        correlationId: 'unknown',
      },
    });
  });

  it('invokes the unauthorized handler exactly once when the error is Unauthorized', async () => {
    const handler = vi.fn();
    setUnauthorizedHandler(handler);
    mockedInvoke.mockRejectedValueOnce({ type: 'Unauthorized' });

    await expect(callCommand('audit_list')).rejects.toBeInstanceOf(AppErrorException);
    expect(handler).toHaveBeenCalledTimes(1);
  });

  it('does not invoke the unauthorized handler for other error types', async () => {
    const handler = vi.fn();
    setUnauthorizedHandler(handler);
    mockedInvoke.mockRejectedValueOnce({ type: 'Conflict', message: 'x' });

    await expect(callCommand('auth_create_user')).rejects.toBeInstanceOf(AppErrorException);
    expect(handler).not.toHaveBeenCalled();
  });
});

afterEach(() => {
  vi.restoreAllMocks();
});
