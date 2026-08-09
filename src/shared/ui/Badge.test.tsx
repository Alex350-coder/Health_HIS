import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { Badge } from './Badge';

describe('Badge', () => {
  it('renders its label text', () => {
    render(<Badge status="success">Available</Badge>);

    expect(screen.getByText('Available')).toBeInTheDocument();
  });

  it('renders a non-color icon alongside the status color, per Rule 19.2', () => {
    render(
      <Badge status="danger" icon={<span data-testid="icon">!</span>}>
        Critical
      </Badge>,
    );

    expect(screen.getByTestId('icon')).toBeInTheDocument();
    expect(screen.getByText('Critical')).toBeInTheDocument();
  });
});
