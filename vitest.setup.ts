import { vi } from 'vitest';

import '@testing-library/jest-dom/vitest';

// No real Tauri IPC bridge exists in jsdom; every test that mounts `App` (and therefore
// `useEventSubscription`) needs `listen` to resolve rather than hang/reject. Test files that
// need to assert on `listen` calls override this with their own `vi.mock` (e.g.
// use-event-subscription.test.tsx), which takes precedence per-file.
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => undefined)),
}));

// jsdom has no ResizeObserver; @tanstack/react-virtual (VirtualizedTable) requires one to exist.
class ResizeObserverStub {
  observe(): void {
    // No layout engine to observe in jsdom — intentionally a no-op.
  }

  unobserve(): void {
    // No layout engine to observe in jsdom — intentionally a no-op.
  }

  disconnect(): void {
    // No layout engine to observe in jsdom — intentionally a no-op.
  }
}

globalThis.ResizeObserver ??= ResizeObserverStub;
