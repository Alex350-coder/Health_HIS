import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { ToastMessage, ToastProvider, ToastViewport } from './Toast';

describe('ToastMessage', () => {
  it('renders its title and description when open', () => {
    render(
      <ToastProvider>
        <ToastMessage
          title="Saved"
          description="Your changes were saved."
          status="success"
          open
          onOpenChange={() => {
            // no-op for this test
          }}
        />
        <ToastViewport />
      </ToastProvider>,
    );

    expect(screen.getByText('Saved')).toBeInTheDocument();
    expect(screen.getByText('Your changes were saved.')).toBeInTheDocument();
  });

  it('does not render its content when closed', () => {
    render(
      <ToastProvider>
        <ToastMessage
          title="Saved"
          status="success"
          open={false}
          onOpenChange={() => {
            // no-op for this test
          }}
        />
        <ToastViewport />
      </ToastProvider>,
    );

    expect(screen.queryByText('Saved')).not.toBeInTheDocument();
  });
});
