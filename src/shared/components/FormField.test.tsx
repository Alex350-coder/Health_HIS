import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { FormField } from './FormField';

describe('FormField', () => {
  it('renders the label and children', () => {
    render(
      <FormField id="username" label="Username">
        <input id="username" />
      </FormField>,
    );

    expect(screen.getByLabelText('Username')).toBeInTheDocument();
  });

  it('renders an error message when provided', () => {
    render(
      <FormField id="username" label="Username" error="required">
        <input id="username" />
      </FormField>,
    );

    expect(screen.getByRole('alert')).toHaveTextContent('required');
  });

  it('renders no error message when not provided', () => {
    render(
      <FormField id="username" label="Username">
        <input id="username" />
      </FormField>,
    );

    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  });
});
