import { create } from 'zustand';

import type { User } from '../types/auth-schemas';

/**
 * In-memory only, never persisted to disk/localStorage (Security.md Section 4 — no "remember
 * me"). Restarting the app requires logging in again.
 */
interface SessionState {
  token: string | null;
  user: User | null;
  setSession: (token: string, user: User) => void;
  clearSession: () => void;
}

export const useSessionStore = create<SessionState>((set) => ({
  token: null,
  user: null,
  setSession: (token, user) => {
    set({ token, user });
  },
  clearSession: () => {
    set({ token: null, user: null });
  },
}));
