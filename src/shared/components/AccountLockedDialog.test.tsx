import { act, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { AccountLockedDialog } from './AccountLockedDialog';

describe('AccountLockedDialog', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('is closed when retryAfterSecs is undefined', () => {
    render(<AccountLockedDialog retryAfterSecs={undefined} onOpenChange={vi.fn()} />);

    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('shows the initial countdown when opened', () => {
    render(<AccountLockedDialog retryAfterSecs={60} onOpenChange={vi.fn()} />);

    expect(screen.getByRole('dialog')).toHaveTextContent('Try again in 60 seconds.');
  });

  it('counts down every second', () => {
    render(<AccountLockedDialog retryAfterSecs={2} onOpenChange={vi.fn()} />);

    act(() => {
      vi.advanceTimersByTime(1000);
    });

    expect(screen.getByRole('dialog')).toHaveTextContent('Try again in 1 second.');
  });

  it('shows a retry-now message once the countdown reaches zero', () => {
    render(<AccountLockedDialog retryAfterSecs={1} onOpenChange={vi.fn()} />);

    act(() => {
      vi.advanceTimersByTime(1000);
    });

    expect(screen.getByRole('dialog')).toHaveTextContent('You can try again now.');
  });
});
