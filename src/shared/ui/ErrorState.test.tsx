import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { ErrorState } from './ErrorState';

describe('ErrorState', () => {
  it('renders a default title as an alert', () => {
    render(<ErrorState />);

    expect(screen.getByRole('alert')).toHaveTextContent('Something went wrong');
  });

  it('calls onRetry when the retry button is clicked', async () => {
    const user = userEvent.setup();
    const onRetry = vi.fn();
    render(<ErrorState description="Could not load patients." onRetry={onRetry} />);

    await user.click(screen.getByRole('button', { name: 'Retry' }));

    expect(onRetry).toHaveBeenCalledOnce();
  });
});
