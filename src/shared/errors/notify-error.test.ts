import { beforeEach, describe, expect, it } from 'vitest';

import { useToastStore } from '@shared/lib/toast-store';

import { AppErrorException } from './app-error';
import { notifyMutationError } from './notify-error';

describe('notifyMutationError', () => {
  beforeEach(() => {
    useToastStore.setState({ toasts: [] });
  });

  it('pushes a toast for a Conflict error', () => {
    notifyMutationError(new AppErrorException({ type: 'Conflict', message: 'duplicate username' }));

    expect(useToastStore.getState().toasts).toMatchObject([
      { title: 'duplicate username', status: 'danger' },
    ]);
  });

  it('pushes a toast for a Database error', () => {
    notifyMutationError(
      new AppErrorException({ type: 'Database', message: 'x', correlationId: 'a1b2c3' }),
    );

    expect(useToastStore.getState().toasts).toMatchObject([
      {
        title: 'Something went wrong saving your changes. (Error reference: a1b2c3)',
        status: 'danger',
      },
    ]);
  });

  it('does not push a toast for AccountLocked', () => {
    notifyMutationError(new AppErrorException({ type: 'AccountLocked', retryAfterSecs: 60 }));

    expect(useToastStore.getState().toasts).toHaveLength(0);
  });

  it('does not push a toast for a non-AppError value', () => {
    notifyMutationError(new Error('boom'));

    expect(useToastStore.getState().toasts).toHaveLength(0);
  });
});
