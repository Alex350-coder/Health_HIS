import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { ErrorBoundary } from './ErrorBoundary';

function Bomb(): JSX.Element {
  throw new Error('boom');
}

describe('ErrorBoundary', () => {
  it('renders its children when there is no error', () => {
    render(
      <ErrorBoundary>
        <p>all good</p>
      </ErrorBoundary>,
    );

    expect(screen.getByText('all good')).toBeInTheDocument();
  });

  it('renders a fallback and reload button when a child throws', () => {
    vi.spyOn(console, 'error').mockImplementation(() => {
      /* React logs the caught error to console; suppress it for this test */
    });

    render(
      <ErrorBoundary>
        <Bomb />
      </ErrorBoundary>,
    );

    expect(screen.getByRole('alert')).toHaveTextContent('Something went wrong.');
    expect(screen.getByRole('button', { name: 'Reload' })).toBeInTheDocument();

    vi.restoreAllMocks();
  });

  it('clears the error state when reload is clicked, re-rendering children on the next pass', async () => {
    vi.spyOn(console, 'error').mockImplementation(() => {
      /* suppress expected React error boundary logging */
    });
    const user = userEvent.setup();
    let shouldThrow = true;
    function MaybeBomb(): JSX.Element {
      if (shouldThrow) {
        throw new Error('boom');
      }
      return <p>recovered</p>;
    }

    const { rerender } = render(
      <ErrorBoundary>
        <MaybeBomb />
      </ErrorBoundary>,
    );
    shouldThrow = false;

    await user.click(screen.getByRole('button', { name: 'Reload' }));
    rerender(
      <ErrorBoundary>
        <MaybeBomb />
      </ErrorBoundary>,
    );

    expect(screen.getByText('recovered')).toBeInTheDocument();
    vi.restoreAllMocks();
  });
});
