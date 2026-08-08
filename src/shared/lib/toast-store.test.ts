import { beforeEach, describe, expect, it } from 'vitest';

import { useToastStore } from './toast-store';

describe('useToastStore', () => {
  beforeEach(() => {
    useToastStore.setState({ toasts: [] });
  });

  it('adds a toast with a generated id', () => {
    useToastStore.getState().pushToast({ title: 'Saved', status: 'success' });

    const { toasts } = useToastStore.getState();
    expect(toasts).toHaveLength(1);
    expect(toasts[0]).toMatchObject({ title: 'Saved', status: 'success' });
    expect(toasts[0]?.id).toBeTruthy();
  });

  it('removes a toast by id', () => {
    useToastStore.getState().pushToast({ title: 'Saved', status: 'success' });
    const id = useToastStore.getState().toasts[0]?.id;

    useToastStore.getState().dismissToast(id ?? '');

    expect(useToastStore.getState().toasts).toHaveLength(0);
  });
});
