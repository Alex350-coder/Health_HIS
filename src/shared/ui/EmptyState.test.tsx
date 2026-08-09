import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { EmptyState } from './EmptyState';

describe('EmptyState', () => {
  it('renders its title and description', () => {
    render(
      <EmptyState title="No patients yet" description="Register the first patient to begin." />,
    );

    expect(screen.getByText('No patients yet')).toBeInTheDocument();
    expect(screen.getByText('Register the first patient to begin.')).toBeInTheDocument();
  });

  it('renders an optional action', () => {
    render(
      <EmptyState title="No patients yet" action={<button type="button">Add patient</button>} />,
    );

    expect(screen.getByRole('button', { name: 'Add patient' })).toBeInTheDocument();
  });
});
