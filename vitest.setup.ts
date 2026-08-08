import '@testing-library/jest-dom/vitest';

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
