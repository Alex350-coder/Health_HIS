import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';

import { useToastStore } from '@shared/lib/toast-store';

import { ToastRegion } from './ToastRegion';

describe('ToastRegion', () => {
  beforeEach(() => {
    useToastStore.setState({ toasts: [] });
  });

  it('renders nothing when the toast queue is empty', () => {
    render(<ToastRegion />);

    expect(screen.queryByText(/./)).not.toBeInTheDocument();
  });

  it('renders a toast pushed to the store', async () => {
    useToastStore.getState().pushToast({ title: 'Username already taken', status: 'danger' });
    render(<ToastRegion />);

    expect(await screen.findByText('Username already taken')).toBeInTheDocument();
  });
});
