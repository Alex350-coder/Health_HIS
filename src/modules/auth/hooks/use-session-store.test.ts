import { beforeEach, describe, expect, it } from 'vitest';

import { useSessionStore } from './use-session-store';

const USER = {
  id: 1,
  fullName: 'Ada Lovelace',
  username: 'ada',
  role: 'admin',
  isActive: true,
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: null,
};

describe('useSessionStore', () => {
  beforeEach(() => {
    useSessionStore.getState().clearSession();
  });

  it('starts with no session', () => {
    const state = useSessionStore.getState();

    expect(state.token).toBeNull();
    expect(state.user).toBeNull();
  });

  it('setSession stores the token and user', () => {
    useSessionStore.getState().setSession('token-123', USER);

    const state = useSessionStore.getState();
    expect(state.token).toBe('token-123');
    expect(state.user).toEqual(USER);
  });

  it('clearSession resets the token and user to null', () => {
    useSessionStore.getState().setSession('token-123', USER);

    useSessionStore.getState().clearSession();

    const state = useSessionStore.getState();
    expect(state.token).toBeNull();
    expect(state.user).toBeNull();
  });
});
